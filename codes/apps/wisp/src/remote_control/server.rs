//! Owner-only Unix socket, one connection, bounded compositor mailbox.
use super::protocol::{self, Command, Reply, Request};
use std::{
    fs,
    io::{self},
    net::Shutdown,
    os::unix::{
        fs::{FileTypeExt, MetadataExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

pub enum Event {
    Begin {
        session: u64,
        alive: Arc<AtomicBool>,
        reply: SyncSender<Reply>,
        id: u64,
    },
    Request {
        session: u64,
        request: Request,
        reply: SyncSender<Reply>,
    },
}

pub fn start() -> io::Result<Receiver<Event>> {
    let runtime = PathBuf::from(
        std::env::var_os("XDG_RUNTIME_DIR")
            .ok_or_else(|| io::Error::other("XDG_RUNTIME_DIR required"))?,
    );
    let meta = fs::metadata(&runtime)?;
    if meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
        return Err(io::Error::other(
            "runtime directory must be private and owned by Wisp user",
        ));
    }
    let path = runtime.join("wisp_remote_control.sock");
    if let Ok(meta) = fs::symlink_metadata(&path) {
        if !meta.file_type().is_socket() || meta.uid() != unsafe { libc::geteuid() } {
            return Err(io::Error::other("refusing to replace non-owned socket"));
        }
        match UnixStream::connect(&path) {
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "remote_control socket in use",
                ));
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => fs::remove_file(&path)?,
            Err(e) => return Err(e),
        }
    }
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    let (tx, rx) = mpsc::sync_channel(16);
    let busy = Arc::new(AtomicBool::new(false));
    let next = AtomicU64::new(1);
    thread::Builder::new()
        .name("remote_control_accept".into())
        .spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { break };
                if busy.swap(true, Ordering::SeqCst) {
                    let _ = stream.shutdown(Shutdown::Both);
                    continue;
                }
                let tx = tx.clone();
                let busy = busy.clone();
                let session = next.fetch_add(1, Ordering::Relaxed);
                if let Err(e) = thread::Builder::new()
                    .name("remote_control_session".into())
                    .spawn(move || {
                        let alive = Arc::new(AtomicBool::new(true));
                        let result = serve(&stream, &tx, session, alive.clone());
                        alive.store(false, Ordering::SeqCst);
                        let _ = stream.shutdown(Shutdown::Both);
                        busy.store(false, Ordering::SeqCst);
                        if let Err(e) = result {
                            tracing::debug!("remote_control disconnected: {e}");
                        }
                    })
                {
                    tracing::error!("remote_control worker: {e}");
                    break;
                }
            }
        })?;
    tracing::info!("remote_control development socket: {}", path.display());
    Ok(rx)
}

fn transact(tx: &SyncSender<Event>, event: Event, rx: Receiver<Reply>) -> io::Result<Reply> {
    tx.try_send(event)
        .map_err(|_| io::Error::other("compositor mailbox full"))?;
    rx.recv_timeout(Duration::from_secs(3))
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "compositor timeout"))
}
fn send(writer: &mut &UnixStream, reply: Reply) -> io::Result<()> {
    let mut json = reply.json;
    let bytes = if let Some(p) = reply.pixels {
        match super::capture::encode(p) {
            Ok(data) => data,
            Err(error) => {
                json["ok"] = false.into();
                json["error"] = error.into();
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    protocol::write_frame(writer, &json, &bytes)
}

fn serve(
    stream: &UnixStream,
    tx: &SyncSender<Event>,
    session: u64,
    alive: Arc<AtomicBool>,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut writer = stream;
    let mut reader = stream.try_clone()?;
    let hello = protocol::read_request(&mut reader)?;
    if !matches!(
        hello.command,
        Command::Hello {
            version: protocol::VERSION
        }
    ) {
        send(
            &mut writer,
            Reply::error(hello.id, "first command must be hello version 1"),
        )?;
        return Ok(());
    }
    let (reply, rx) = mpsc::sync_channel(1);
    send(
        &mut writer,
        transact(
            tx,
            Event::Begin {
                session,
                alive: alive.clone(),
                reply,
                id: hello.id,
            },
            rx,
        )?,
    )?;
    let (requests, incoming) = mpsc::sync_channel(16);
    thread::Builder::new()
        .name("remote_control_read".into())
        .spawn(move || {
            loop {
                let request = protocol::read_request(&mut reader);
                let failed = request.is_err();
                if requests.try_send(request).is_err() || failed {
                    break;
                }
            }
        })?;
    let mut fps = 0u8;
    let mut codec = super::video::Codec::H264;
    let mut video: Option<super::video::VideoWorker> = None;
    let mut capture: Option<(Receiver<Reply>, Instant)> = None;
    let mut discard_capture = false;
    let started = Instant::now();
    let mut next_frame = Instant::now();
    let mut heartbeat = Instant::now();
    while alive.load(Ordering::SeqCst) && heartbeat.elapsed() < Duration::from_secs(10) {
        match incoming.recv_timeout(Duration::from_millis(5)) {
            Ok(Ok(request)) => {
                heartbeat = Instant::now();
                let id = request.id;
                match request.command {
                    Command::Stream {
                        fps: requested,
                        codec: requested_codec,
                    } => match super::video::Codec::parse(requested_codec.as_deref()) {
                        Err(error) => send(&mut writer, Reply::error(id, error))?,
                        Ok(selected) if requested <= 30 => {
                            fps = requested;
                            codec = selected;
                            video = None;
                            if fps > 0 {
                                video = Some(super::video::VideoWorker::new(codec, fps));
                            }
                            next_frame = Instant::now();
                            let mut reply = Reply::ok(id);
                            reply.json["codec"] = codec.name().into();
                            reply.json["fps"] = fps.into();
                            send(&mut writer, reply)?;
                        }
                        _ => send(&mut writer, Reply::error(id, "fps must be 0..30"))?,
                    },
                    Command::Keyframe => {
                        if let Some(video) = &video {
                            video.keyframe();
                            send(&mut writer, Reply::ok(id))?;
                        } else if fps > 0 {
                            // A paused screen has no encoder; wake starts with IDR.
                            send(&mut writer, Reply::ok(id))?;
                        } else {
                            send(&mut writer, Reply::error(id, "stream is stopped"))?;
                        }
                    }
                    Command::Disconnect => {
                        send(&mut writer, Reply::ok(id))?;
                        break;
                    }
                    _ => {
                        let is_status = matches!(request.command, Command::Status);
                        let sleep = matches!(request.command, Command::Sleep);
                        let wake = matches!(request.command, Command::Wake);
                        let (reply, rx) = mpsc::sync_channel(1);
                        let mut response = transact(
                            tx,
                            Event::Request {
                                session,
                                request,
                                reply,
                            },
                            rx,
                        )?;
                        if is_status {
                            response.json["stream_codec"] = codec.name().into();
                            response.json["stream_fps"] = fps.into();
                        }
                        send(&mut writer, response)?;
                        if sleep || wake {
                            video = None;
                            discard_capture = capture.is_some();
                            next_frame = Instant::now();
                            if sleep {
                                let mut paused = Reply::error(0, "screen is off; wake first");
                                paused.json["event"] = "frame".into();
                                send(&mut writer, paused)?;
                            }
                        }
                    }
                }
            }
            Ok(Err(e)) => return Err(e),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        // Command replies are sent first. Encoding and backpressure cannot block
        // dispatch while a capture is waiting on the compositor or CPU encoder.
        if let Some(worker) = &video {
            match worker.output.as_ref().unwrap().try_recv() {
                Ok(Ok(packet)) => {
                    protocol::write_frame(&mut writer, &packet.metadata, &packet.bytes)?
                }
                Ok(Err(error)) => {
                    let mut response = Reply::error(0, error);
                    response.json["event"] = "frame".into();
                    send(&mut writer, response)?;
                    video = None;
                    fps = 0;
                }
                _ => {}
            }
        }
        let reply = if let Some((receiver, deadline)) = &capture {
            match receiver.try_recv() {
                Ok(reply) => Some(reply),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(io::Error::other("capture channel closed"));
                }
                Err(mpsc::TryRecvError::Empty) if Instant::now() > *deadline => {
                    return Err(io::Error::other("capture timed out"));
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(mut reply) = reply {
            capture = None;
            if discard_capture {
                discard_capture = false;
                continue;
            }
            if let Some(pixels) = reply.pixels.take() {
                if video.is_none() && fps > 0 {
                    video = Some(super::video::VideoWorker::new(codec, fps));
                }
                if let Some(worker) = &video {
                    worker.submit(pixels, reply.json, started.elapsed().as_micros() as u64);
                }
            } else {
                reply.json["event"] = "frame".into();
                send(&mut writer, reply)?;
                // The next successful frame after wake starts a fresh IDR stream.
                video = None;
                next_frame = Instant::now() + Duration::from_millis(500);
            }
        }
        if fps > 0
            && capture.is_none()
            && Instant::now() >= next_frame
            && alive.load(Ordering::SeqCst)
        {
            let (reply, receiver) = mpsc::sync_channel(1);
            tx.try_send(Event::Request {
                session,
                request: Request {
                    id: 0,
                    command: Command::Screenshot,
                },
                reply,
            })
            .map_err(|_| io::Error::other("capture mailbox full"))?;
            capture = Some((receiver, Instant::now() + Duration::from_secs(3)));
            next_frame = Instant::now() + Duration::from_secs_f64(1.0 / fps as f64);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::io::Read;

    fn read_frame(stream: &mut UnixStream) -> Value {
        let mut header = [0; 8];
        stream.read_exact(&mut header).unwrap();
        assert_eq!(&header[4..], &[0; 4]);
        let mut body = vec![0; u32::from_be_bytes(header[..4].try_into().unwrap()) as usize];
        stream.read_exact(&mut body).unwrap();
        serde_json::from_slice(&body).unwrap()
    }
    #[test]
    fn handshake_status_and_disconnect_share_one_connection() {
        let (mut client, server) = UnixStream::pair().unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let (tx, rx) = mpsc::sync_channel(16);
        let compositor = thread::spawn(move || {
            match rx.recv().unwrap() {
                Event::Begin {
                    session, id, reply, ..
                } => {
                    assert_eq!(session, 7);
                    reply.send(Reply::ok(id)).unwrap();
                }
                _ => panic!("expected begin"),
            }
            match rx.recv().unwrap() {
                Event::Request {
                    session,
                    request,
                    reply,
                } => {
                    assert_eq!(session, 7);
                    assert!(matches!(request.command, Command::Status));
                    reply.send(Reply::ok(request.id)).unwrap();
                }
                _ => panic!("expected status"),
            }
        });
        let worker =
            thread::spawn(move || serve(&server, &tx, 7, Arc::new(AtomicBool::new(true))).unwrap());
        protocol::write_frame(
            &mut client,
            &json!({"id":1,"type":"hello","version":1}),
            &[],
        )
        .unwrap();
        assert_eq!(read_frame(&mut client)["id"], 1);
        protocol::write_frame(&mut client, &json!({"id":2,"type":"status"}), &[]).unwrap();
        assert_eq!(read_frame(&mut client)["id"], 2);
        protocol::write_frame(&mut client, &json!({"id":3,"type":"disconnect"}), &[]).unwrap();
        assert_eq!(read_frame(&mut client)["id"], 3);
        worker.join().unwrap();
        compositor.join().unwrap();
    }
    #[test]
    fn rejects_protocol_version_before_compositor_session() {
        let (mut client, server) = UnixStream::pair().unwrap();
        let (tx, rx) = mpsc::sync_channel(1);
        let worker =
            thread::spawn(move || serve(&server, &tx, 1, Arc::new(AtomicBool::new(true))).unwrap());
        protocol::write_frame(
            &mut client,
            &json!({"id":1,"type":"hello","version":2}),
            &[],
        )
        .unwrap();
        assert_eq!(read_frame(&mut client)["ok"], false);
        worker.join().unwrap();
        assert!(rx.try_recv().is_err());
    }
}
