use bhoo::window::{self, FrameAction, WindowOptions};
use spectre_files::{Action, Appearance, Browser, Filter, PickerRequest};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let mut request = None;
    let mut start = None;
    let mut multiple = false;
    let mut filter = Filter::Any;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--help") => {
                println!(
                    "spectre-files [--start FOLDER] [--open [--multiple] [--images] | --folder | --save NAME]"
                );
                println!(
                    "A chooser prints one encoded file URI per line. Cancel returns status 1."
                );
                return Ok(());
            }
            Some("--start") => {
                start = Some(PathBuf::from(
                    args.next().ok_or("Provide a starting folder")?,
                ))
            }
            Some("--open") if request.is_none() => {
                request = Some(PickerRequest::Open {
                    filter: Filter::Any,
                    multiple: false,
                })
            }
            Some("--folder") if request.is_none() => request = Some(PickerRequest::Folder),
            Some("--save") if request.is_none() => {
                request = Some(PickerRequest::Save {
                    suggested_name: args
                        .next()
                        .ok_or("Provide a file name")?
                        .into_string()
                        .map_err(|_| "The proposed file name must be UTF-8")?,
                })
            }
            Some("--multiple") => multiple = true,
            Some("--images") => filter = Filter::Images,
            _ => return Err(format!("Unexpected argument {}", arg.to_string_lossy()).into()),
        }
    }
    if let Some(PickerRequest::Open {
        filter: target,
        multiple: allow,
    }) = &mut request
    {
        *target = filter;
        *allow = multiple;
    } else if multiple || filter != Filter::Any {
        return Err("--multiple and --images require --open".into());
    }
    let picker = request.is_some();
    let mut browser = request.map(Browser::picker).unwrap_or_default();
    if let Some(start) = start {
        browser.set_start_folder(std::path::absolute(start)?)?;
    }
    let mut selected = false;
    let result = window::run(
        WindowOptions {
            title: if picker {
                "Choose files".into()
            } else {
                "Files".into()
            },
            app_id: Some("org.spectreos.Files".into()),
            size: [480, 750],
            decorations: false,
            ..WindowOptions::default()
        },
        |f| match browser.show(f, f.available(), Appearance::default()) {
            Action::Selected(paths) => {
                for path in paths {
                    println!("file://{}", spectre_files::uri_path(&path));
                }
                selected = true;
                FrameAction::Exit
            }
            Action::Cancelled => FrameAction::Exit,
            Action::NewWindow(folder) => {
                let result = std::env::current_exe().and_then(|path| {
                    let launcher = std::env::var_os("SPECTRE_FILES_LAUNCHER")
                        .map(PathBuf::from)
                        .unwrap_or(path);
                    let mut command = Command::new(launcher);
                    if let Some(folder) = folder {
                        command.arg("--start").arg(folder);
                    }
                    command.spawn()
                });
                if let Err(error) = result {
                    browser.report(format!("Could not open window: {error}"));
                }
                FrameAction::Redraw
            }
            Action::Open(path) => {
                if let Err(error) = open(&path) {
                    browser.report(format!("Could not open file: {error}"));
                }
                FrameAction::Redraw
            }
            Action::Share(paths) => {
                if let Err(error) = spectre_files::share_to_clipboard(&paths, None) {
                    browser.report(format!("Could not copy file links: {error}"));
                } else {
                    browser.report("File links copied");
                }
                FrameAction::Redraw
            }
            Action::Focus(_) | Action::Blur | Action::None => FrameAction::Wait,
        },
    );
    result?;
    if picker && !selected {
        std::process::exit(1);
    }
    Ok(())
}

fn open(path: &Path) -> std::io::Result<()> {
    Command::new("xdg-open")
        .arg(path)
        .stdout(Stdio::null())
        .spawn()
        .map(|_| ())
}
