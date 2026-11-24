//! Files and its reusable document chooser.
mod browser;
mod operations;
mod view;
#[cfg(target_arch = "wasm32")]
mod wasm_fs;

pub use browser::{Action, Browser, Filter, PickerRequest};
pub use operations::uri_path;
pub use view::{Appearance, EDIT_FIELD_ID, SEARCH_FIELD_ID};

use std::{io, path::Path};

/// Decode a local image into a bounded RGBA image suitable for a picker preview or wallpaper.
pub fn read_image(path: &Path, max_edge: u32) -> io::Result<image::RgbaImage> {
    let mut reader = image::ImageReader::open(path)?.with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12000);
    limits.max_image_height = Some(12000);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().map_err(io::Error::other)?;
    let max_edge = max_edge.max(1);
    if image.width().max(image.height()) > max_edge {
        Ok(image
            .resize(max_edge, max_edge, image::imageops::FilterType::Triangle)
            .into_rgba8())
    } else {
        Ok(image.into_rgba8())
    }
}

/// Offer encoded file links to other Wayland applications using the standard URI-list MIME type.
pub fn share_to_clipboard(paths: &[std::path::PathBuf], display: Option<&str>) -> io::Result<()> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut command = Command::new("wl-copy");
    if let Some(display) = display {
        command.env("WAYLAND_DISPLAY", display);
    }
    let mut child = command
        .args(["--type", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        for path in paths {
            writeln!(stdin, "file://{}\r", uri_path(path))?;
        }
    }
    if !child.wait()?.success() {
        return Err(io::Error::other("Clipboard owner failed"));
    }
    Ok(())
}
