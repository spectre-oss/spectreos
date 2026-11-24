use crate::{protocol, video::VideoFrame};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    io::{self, Read, Write},
    net::{Shutdown, TcpStream, ToSocketAddrs},
    os::unix::net::UnixStream,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::Duration,
};

#[derive(Clone, Debug)]
pub enum Endpoint {
    Session(PathBuf),
    Tcp(String),
    Unix(PathBuf),
}
impl Endpoint {
    pub fn attached(&self) -> bool {
        matches!(self, Self::Session(_))
    }
}
pub fn session_path(name: &str) -> Result<PathBuf, String> {
    if name.is_empty()
        || name.len() > 40
        || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
    {
        return Err("invalid session name".into());
    }
    let root = if let Some(path) = std::env::var_os("XDG_RUNTIME_DIR") {
        PathBuf::from(path)
    } else {
        PathBuf::from(std::env::var_os("HOME").ok_or("HOME or XDG_RUNTIME_DIR is required")?)
            .join(".cache")
    };
    Ok(root
        .join("wisp_remote_control")
        .join(format!("{name}.sock")))
}
#[derive(Debug)]
pub struct Shared {
    pub frame: Option<Arc<VideoFrame>>,
    pub logical: [f64; 2],
    pub frames: u64,
    pub connected: bool,
    pub finished: bool,
    pub message: String,
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            frame: None,
            logical: [400.0, 832.0],
            frames: 0,
            connected: false,
            finished: false,
            message: "Connecting".into(),
        }
    }
}
#[derive(Debug)]
pub enum Command {
    Input(Value),
    Stop,
}
struct Control {
    running: AtomicBool,
    abort: Mutex<Option<Stream>>,
    writer_started: AtomicBool,
    needs_keyframe: AtomicBool,
    done: Mutex<bool>,
    finished: Condvar,
}
struct WriterDone(Arc<Control>);
impl Drop for WriterDone {
    fn drop(&mut self) {
        *self.0.done.lock().unwrap() = true;
        self.0.finished.notify_all();
    }
}
pub struct Worker {
    sender: SyncSender<Command>,
    control: Arc<Control>,
}
impl Worker {
    pub fn send(&self, command: Command) {
        if self.sender.try_send(command).is_err() {
            self.stop();
        }
    }
    pub fn stop(&self) {
        self.control.running.store(false, Ordering::SeqCst);
        let _ = self.sender.try_send(Command::Stop);
        if let Some(stream) = &*self.control.abort.lock().unwrap() {
            stream.shutdown();
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
        if self.control.writer_started.load(Ordering::SeqCst) {
            let _ = self.control.finished.wait_timeout_while(
                self.control.done.lock().unwrap(),
                Duration::from_secs(6),
                |done| !*done,
            );
        }
    }
}

pub enum Stream {
    Tcp(TcpStream),
    Unix(UnixStream),
}
impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(s) => s.read(buf),
            Self::Unix(s) => s.read(buf),
        }
    }
}
impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(s) => s.write(buf),
            Self::Unix(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Tcp(s) => s.flush(),
            Self::Unix(s) => s.flush(),
        }
    }
}
impl Stream {
    fn open(endpoint: &Endpoint) -> Result<Self, String> {
        let stream = match endpoint {
            Endpoint::Session(path) | Endpoint::Unix(path) => Self::Unix(
                UnixStream::connect(path).map_err(|e| format!("{}: {e}", path.display()))?,
            ),
            Endpoint::Tcp(address) => {
                let mut result = Err(io::Error::other("no address"));
                for address in address.to_socket_addrs().map_err(|e| e.to_string())? {
                    result = TcpStream::connect_timeout(&address, Duration::from_secs(5));
                    if result.is_ok() {
                        break;
                    }
                }
                let stream = result.map_err(|e| e.to_string())?;
                stream.set_nodelay(true).map_err(|e| e.to_string())?;
                Self::Tcp(stream)
            }
        };
        match &stream {
            Self::Tcp(s) => {
                s.set_read_timeout(Some(Duration::from_secs(6)))
                    .map_err(|e| e.to_string())?;
                s.set_write_timeout(Some(Duration::from_secs(3)))
                    .map_err(|e| e.to_string())?;
            }
            Self::Unix(s) => {
                s.set_read_timeout(Some(Duration::from_secs(6)))
                    .map_err(|e| e.to_string())?;
                s.set_write_timeout(Some(Duration::from_secs(3)))
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(stream)
    }
    fn duplicate(&self) -> Result<Self, String> {
        match self {
            Self::Tcp(s) => s.try_clone().map(Self::Tcp),
            Self::Unix(s) => s.try_clone().map(Self::Unix),
        }
        .map_err(|e| e.to_string())
    }
    fn shutdown(&self) {
        match self {
            Self::Tcp(s) => {
                let _ = s.shutdown(Shutdown::Both);
            }
            Self::Unix(s) => {
                let _ = s.shutdown(Shutdown::Both);
            }
        }
    }
}
type State = Arc<Mutex<Shared>>;
type Notify = Arc<dyn Fn() + Send + Sync>;
fn status(state: &State, metadata: &Value) {
    let mut state = state.lock().unwrap();
    for (index, name) in ["logical_width", "logical_height"].iter().enumerate() {
        if let Some(size) = metadata[*name]
            .as_f64()
            .filter(|n| n.is_finite() && *n > 0.0 && *n < 32768.0)
        {
            state.logical[index] = size;
        }
    }
}
fn accept_frame(
    state: &State,
    metadata: Value,
    bytes: Vec<u8>,
    notify: &Notify,
    decoder: &mut crate::h264::FrameDecoder,
    control: &Arc<Control>,
) -> Result<(), String> {
    if protocol::check_reply(&metadata).is_err() {
        state.lock().unwrap().frame = None;
        state.lock().unwrap().message = metadata["error"]
            .as_str()
            .unwrap_or("Stream unavailable")
            .into();
        notify();
        return Ok(());
    }
    let Some(decoded) = decoder.decode(&metadata, bytes)? else {
        if decoder.needs_keyframe {
            control.needs_keyframe.store(true, Ordering::SeqCst);
        }
        return Ok(());
    };
    let frame = Arc::new(decoded);
    status(state, &metadata);
    {
        let mut state = state.lock().unwrap();
        state.frame = Some(frame);
        state.frames += 1;
        state.message = "Connected".into();
    }
    notify();
    Ok(())
}
fn rpc(endpoint: &Endpoint, command: &Value) -> Result<(Value, Vec<u8>), String> {
    let mut stream = Stream::open(endpoint)?;
    protocol::write_frame(&mut stream, command).map_err(|e| e.to_string())?;
    let (meta, bytes) = protocol::read_frame(&mut stream).map_err(|e| e.to_string())?;
    protocol::check_reply(&meta)?;
    Ok((meta, bytes))
}
fn next_command(
    receiver: &Receiver<Command>,
    deferred: &mut Option<Command>,
) -> Result<Command, mpsc::RecvTimeoutError> {
    let mut command = if let Some(command) = deferred.take() {
        command
    } else {
        receiver.recv_timeout(Duration::from_millis(200))?
    };
    // Coalesce consecutive motion only; never move a press/release/key across it.
    if let Command::Input(ref value) = command {
        if value["type"] == "touch_move" {
            let slot = value["slot"].clone();
            while let Ok(next) = receiver.try_recv() {
                if matches!(&next,Command::Input(v) if v["type"]=="touch_move" && v["slot"]==slot) {
                    command = next;
                } else {
                    *deferred = Some(next);
                    break;
                }
            }
        }
    }
    Ok(command)
}

pub fn start(endpoint: Endpoint, state: State, notify: Notify) -> Worker {
    let (sender, receiver) = mpsc::sync_channel(128);
    let control = Arc::new(Control {
        running: AtomicBool::new(true),
        abort: Mutex::new(None),
        writer_started: AtomicBool::new(false),
        needs_keyframe: AtomicBool::new(false),
        done: Mutex::new(false),
        finished: Condvar::new(),
    });
    let inner = control.clone();
    thread::spawn(move || {
        let result = if endpoint.attached() {
            attached(endpoint, &state, &notify, &inner, receiver)
        } else {
            direct(endpoint, &state, &notify, &inner, receiver)
        };
        inner.running.store(false, Ordering::SeqCst);
        if let Some(stream) = &*inner.abort.lock().unwrap() {
            stream.shutdown();
        }
        {
            let mut state = state.lock().unwrap();
            state.connected = false;
            state.finished = true;
            state.message = result.err().unwrap_or_else(|| "Detached".into());
        }
        notify();
    });
    Worker { sender, control }
}
fn attached(
    endpoint: Endpoint,
    state: &State,
    notify: &Notify,
    control: &Arc<Control>,
    receiver: Receiver<Command>,
) -> Result<(), String> {
    let (meta, _) = rpc(&endpoint, &json!({"type":"status"}))?;
    if meta["connected"] != true {
        return Err(meta["error"]
            .as_str()
            .unwrap_or("CLI session disconnected; restart it explicitly")
            .into());
    }
    status(state, &meta);
    state.lock().unwrap().connected = true;
    notify();
    let writer_endpoint = endpoint.clone();
    let writer_control = control.clone();
    let writer_state = state.clone();
    let writer_notify = notify.clone();
    control.writer_started.store(true, Ordering::SeqCst);
    thread::spawn(move || {
        let _done = WriterDone(writer_control.clone());
        let mut keys = HashSet::new();
        let mut contacts = HashSet::new();
        let mut deferred = None;
        while writer_control.running.load(Ordering::SeqCst) {
            match next_command(&receiver, &mut deferred) {
                Ok(Command::Input(value)) => match rpc(&writer_endpoint, &value) {
                    Ok(_) => match value["type"].as_str() {
                        Some("key") => {
                            if let Some(code) = value["code"].as_u64() {
                                if value["down"] == true {
                                    keys.insert(code);
                                } else {
                                    keys.remove(&code);
                                }
                            }
                        }
                        Some("touch_down") => {
                            if let Some(slot) = value["slot"].as_u64() {
                                contacts.insert(slot);
                            }
                        }
                        Some("touch_up") => {
                            if let Some(slot) = value["slot"].as_u64() {
                                contacts.remove(&slot);
                            }
                        }
                        Some("touch_cancel") => contacts.clear(),
                        _ => {}
                    },
                    Err(error) => {
                        writer_state.lock().unwrap().message = error;
                        writer_notify();
                    }
                },
                Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                    writer_control.running.store(false, Ordering::SeqCst);
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if writer_control.needs_keyframe.swap(false, Ordering::SeqCst) {
                        let _ = rpc(&writer_endpoint, &json!({"type":"keyframe"}));
                    }
                }
            }
        }
        // Closing a viewer detaches; it must not leave input held in the persistent
        // controller, even when queued focus-loss releases have not been sent yet.
        if !contacts.is_empty() {
            let _ = rpc(&writer_endpoint, &json!({"type":"touch_cancel"}));
        }
        for code in keys {
            let _ = rpc(
                &writer_endpoint,
                &json!({"type":"key","code":code,"down":false}),
            );
        }
    });
    let mut decoder = crate::h264::FrameDecoder::default();
    let mut after = 0;
    while control.running.load(Ordering::SeqCst) {
        match rpc(&endpoint, &json!({"type":"video","after":after})) {
            Ok((meta, bytes)) => {
                after = meta["client_packet_sequence"]
                    .as_u64()
                    .ok_or("missing streamed frame sequence")?;
                accept_frame(state, meta, bytes, notify, &mut decoder, control)?;
            }
            Err(error) => {
                let (meta, _) = rpc(&endpoint, &json!({"type":"status"}))?;
                if meta["connected"] != true {
                    return Err(
                        "CLI session disconnected; local phone input may have revoked it".into(),
                    );
                }
                state.lock().unwrap().message = error;
                notify();
                thread::sleep(Duration::from_millis(200));
            }
        }
    }
    Ok(())
}
fn direct(
    endpoint: Endpoint,
    state: &State,
    notify: &Notify,
    control: &Arc<Control>,
    receiver: Receiver<Command>,
) -> Result<(), String> {
    let mut stream = Stream::open(&endpoint)?;
    *control.abort.lock().unwrap() = Some(stream.duplicate()?);
    protocol::write_frame(&mut stream, &json!({"id":1,"type":"hello","version":1}))
        .map_err(|e| e.to_string())?;
    let (hello, _) = protocol::read_frame(&mut stream)
        .map_err(|e| format!("Wisp refused connection (another controller may own it): {e}"))?;
    protocol::check_reply(&hello)?;
    if hello["version"] != 1 {
        return Err("unsupported Wisp protocol version".into());
    }
    status(state, &hello);
    protocol::write_frame(
        &mut stream,
        &json!({"id":2,"type":"stream","fps":15,"codec":"h264"}),
    )
    .map_err(|e| e.to_string())?;
    let (ack, _) = protocol::read_frame(&mut stream).map_err(|e| e.to_string())?;
    protocol::check_reply(&ack)?;
    state.lock().unwrap().connected = true;
    notify();
    let mut writer = stream.duplicate()?;
    type Pending = Arc<Mutex<HashMap<u64, SyncSender<Result<(), String>>>>>;
    let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
    let writer_pending = pending.clone();
    let writer_control = control.clone();
    let writer_state = state.clone();
    let writer_notify = notify.clone();
    control.writer_started.store(true, Ordering::SeqCst);
    thread::spawn(move || {
        let _done = WriterDone(writer_control.clone());
        let mut identifier = 2;
        let mut heartbeat = std::time::Instant::now();
        let mut deferred = None;
        while writer_control.running.load(Ordering::SeqCst) {
            let mut value = match next_command(&receiver, &mut deferred) {
                Ok(Command::Input(value)) => value,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if writer_control.needs_keyframe.swap(false, Ordering::SeqCst) {
                        json!({"type":"keyframe"})
                    } else if heartbeat.elapsed() >= Duration::from_secs(2) {
                        json!({"type":"heartbeat"})
                    } else {
                        continue;
                    }
                }
                _ => break,
            };
            heartbeat = std::time::Instant::now();
            identifier += 1;
            value["id"] = identifier.into();
            let (tx, rx) = mpsc::sync_channel(1);
            writer_pending.lock().unwrap().insert(identifier, tx);
            let result = protocol::write_frame(&mut writer, &value)
                .map_err(|e| e.to_string())
                .and_then(|_| {
                    rx.recv_timeout(Duration::from_secs(5))
                        .map_err(|_| "command acknowledgement timed out".to_string())
                });
            writer_pending.lock().unwrap().remove(&identifier);
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    writer_state.lock().unwrap().message = error;
                    writer_notify();
                }
                Err(error) => {
                    writer_state.lock().unwrap().message = error;
                    break;
                }
            }
        }
        writer_control.running.store(false, Ordering::SeqCst);
        writer.shutdown();
    });
    let mut decoder = crate::h264::FrameDecoder::default();
    while control.running.load(Ordering::SeqCst) {
        let (meta, bytes) = protocol::read_frame(&mut stream).map_err(|e| {
            format!("Disconnected: {e}. Reconnect explicitly after local takeover.")
        })?;
        if meta["event"] == "frame" {
            accept_frame(state, meta, bytes, notify, &mut decoder, control)?;
        } else if let Some(id) = meta["id"].as_u64() {
            if let Some(reply) = pending.lock().unwrap().remove(&id) {
                let _ = reply.try_send(protocol::check_reply(&meta));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn motion_coalescing_preserves_release_and_key_order() {
        let (tx, rx) = mpsc::sync_channel(8);
        let mut deferred = None;
        for value in [
            json!({"type":"touch_move","slot":0,"x":1}),
            json!({"type":"touch_move","slot":0,"x":2}),
            json!({"type":"touch_up","slot":0}),
            json!({"type":"key","code":30,"down":true}),
        ] {
            tx.send(Command::Input(value)).unwrap();
        }
        assert!(matches!(next_command(&rx,&mut deferred).unwrap(),Command::Input(v) if v["x"]==2));
        assert!(
            matches!(next_command(&rx,&mut deferred).unwrap(),Command::Input(v) if v["type"]=="touch_up")
        );
        assert!(
            matches!(next_command(&rx,&mut deferred).unwrap(),Command::Input(v) if v["type"]=="key")
        );
    }
    #[test]
    fn detaching_releases_held_keys_without_stopping_controller() {
        use std::{
            os::unix::net::UnixListener,
            time::{Instant, SystemTime, UNIX_EPOCH},
        };
        let path = std::env::temp_dir().join(format!(
            "wisp_rust_viewer_{}_{}.sock",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let calls = Arc::new(Mutex::new(Vec::<Value>::new()));
        let log = calls.clone();
        let alive = Arc::new(AtomicBool::new(true));
        let serving = alive.clone();
        let server = thread::spawn(move || {
            let mut sequence = 0u64;
            let mut png = Vec::new();
            {
                let mut encoder = png::Encoder::new(&mut png, 1, 1);
                encoder.set_color(png::ColorType::Rgb);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&[10, 20, 30])
                    .unwrap();
            }
            while serving.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(1)))
                            .unwrap();
                        let (command, _) = protocol::read_frame(&mut stream).unwrap();
                        log.lock().unwrap().push(command.clone());
                        let mut reply = json!({"ok":true,"connected":true,"logical_width":400,"logical_height":832});
                        let binary = if command["type"] == "frame" || command["type"] == "video" {
                            sequence += 1;
                            reply["client_sequence"] = sequence.into();
                            reply["client_packet_sequence"] = sequence.into();
                            &png[..]
                        } else {
                            &[][..]
                        };
                        let bytes = serde_json::to_vec(&reply).unwrap();
                        let _ = stream.write_all(&(bytes.len() as u32).to_be_bytes());
                        let _ = stream.write_all(&(binary.len() as u32).to_be_bytes());
                        let _ = stream.write_all(&bytes);
                        let _ = stream.write_all(binary);
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2))
                    }
                    Err(e) => panic!("{e}"),
                }
            }
        });
        let state = Arc::new(Mutex::new(Shared::default()));
        let worker = start(
            Endpoint::Session(path.clone()),
            state.clone(),
            Arc::new(|| {}),
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        while !state.lock().unwrap().connected && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(state.lock().unwrap().connected);
        worker.send(Command::Input(json!({"type":"key","code":42,"down":true})));
        while !calls
            .lock()
            .unwrap()
            .iter()
            .any(|c| c["type"] == "key" && c["down"] == true)
            && Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(5));
        }
        drop(worker);
        let result = rpc(&Endpoint::Session(path.clone()), &json!({"type":"status"})).unwrap();
        assert_eq!(result.0["connected"], true);
        let log = calls.lock().unwrap();
        assert!(
            log.iter()
                .any(|c| c["type"] == "key" && c["code"] == 42 && c["down"] == false)
        );
        assert!(!log.iter().any(|c| c["type"] == "stop"));
        drop(log);
        alive.store(false, Ordering::SeqCst);
        server.join().unwrap();
        let _ = std::fs::remove_file(path);
    }
}
