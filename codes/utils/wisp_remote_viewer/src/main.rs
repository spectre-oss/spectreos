mod h264;
mod input;
mod network;
mod protocol;
mod video;
mod viewer;

use network::{Command, Endpoint, Shared};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Options {
    pub endpoint: Endpoint,
    pub headless: bool,
    pub frames: u64,
    pub output: Option<PathBuf>,
    pub exit_after: Option<u64>,
}
fn options() -> Result<Option<Options>, String> {
    let mut args = std::env::args().skip(1);
    let mut result = Options {
        endpoint: Endpoint::Session(network::session_path("poco")?),
        headless: false,
        frames: 5,
        output: None,
        exit_after: None,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "wisp_remote_viewer [--session NAME | --control-socket PATH | --connect HOST:PORT | --socket PATH]\n  --session poco       Attach to the persistent CLI (default); closing only detaches\n  --connect HOST:PORT  Own a direct SSH-forwarded Wisp connection\n  --socket PATH        Own a direct Wisp Unix socket connection\n  --headless --frames N --output FILE.png  Validate N streamed frames without a display\n  --exit-after-frames N  Exit the graphical viewer after N frames (smoke test)\nMouse left/right and up to ten touch contacts; keyboard forwards physical keys.\nToolbar: Wake, Sleep, Back, Home, Recents, Detach/Stop. Local phone input revokes control."
                );
                return Ok(None);
            }
            "--session" => {
                result.endpoint = Endpoint::Session(network::session_path(
                    &args.next().ok_or("missing session name")?,
                )?)
            }
            "--control-socket" => {
                result.endpoint =
                    Endpoint::Session(args.next().ok_or("missing socket path")?.into())
            }
            "--connect" => result.endpoint = Endpoint::Tcp(args.next().ok_or("missing HOST:PORT")?),
            "--socket" => {
                result.endpoint = Endpoint::Unix(args.next().ok_or("missing socket path")?.into())
            }
            "--headless" => result.headless = true,
            "--frames" => {
                result.frames = args
                    .next()
                    .ok_or("missing frame count")?
                    .parse()
                    .map_err(|_| "invalid frame count")?
            }
            "--output" => result.output = Some(args.next().ok_or("missing output path")?.into()),
            "--exit-after-frames" => {
                result.exit_after = Some(
                    args.next()
                        .ok_or("missing frame count")?
                        .parse()
                        .map_err(|_| "invalid frame count")?,
                )
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
    }
    if result.frames == 0 || result.exit_after == Some(0) {
        return Err("frame count must be positive".into());
    }
    Ok(Some(result))
}
fn main() {
    if let Err(error) = run() {
        eprintln!("wisp_remote_viewer: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    if std::env::args().any(|arg| arg == "--decode-video") {
        return h264::decode_pipe();
    }
    let Some(options) = options()? else {
        return Ok(());
    };
    if options.headless {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let worker = network::start(options.endpoint, shared.clone(), Arc::new(|| {}));
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            {
                let state = shared.lock().unwrap();
                if state.frames >= options.frames && state.frame.is_some() {
                    if let Some(path) = &options.output {
                        state
                            .frame
                            .as_ref()
                            .ok_or("missing decoded frame")?
                            .save(path)
                            .map_err(|e| e.to_string())?;
                    }
                    println!(
                        "{}",
                        serde_json::json!({"ok":true,"frames":state.frames,"width":state.frame.as_ref().map(|f|f.width),"height":state.frame.as_ref().map(|f|f.height),"source":"stream"})
                    );
                    break;
                }
                if state.finished {
                    return Err(state.message.clone());
                }
            }
            if Instant::now() >= deadline {
                return Err("stream timed out".into());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        worker.send(Command::Stop);
        worker.stop();
        Ok(())
    } else {
        viewer::run(options)
    }
}
