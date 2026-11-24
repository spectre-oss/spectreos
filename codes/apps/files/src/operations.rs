#![cfg_attr(target_arch = "wasm32", allow(dead_code))]
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

fn error(kind: io::ErrorKind, message: &str) -> io::Error {
    io::Error::new(kind, message.to_owned())
}

pub(crate) fn child(dir: &Path, name: &str) -> io::Result<PathBuf> {
    let path = Path::new(name);
    let mut parts = path.components();
    if !matches!(parts.next(), Some(Component::Normal(_)))
        || parts.next().is_some()
        || name.ends_with('/')
        || name.contains('\0')
    {
        return Err(error(
            io::ErrorKind::InvalidInput,
            "Enter one file or folder name",
        ));
    }
    Ok(dir.join(path))
}

pub(crate) fn remove(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.file_type().is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn rename(source: &Path, dest: &Path) -> io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let source = CString::new(source.as_os_str().as_bytes())?;
    let dest = CString::new(dest.as_os_str().as_bytes())?;
    // SAFETY: both C strings live through the call. Neither pointer is retained.
    if unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            dest.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    } == 0
    {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn rename(source: &Path, dest: &Path) -> io::Result<()> {
    if fs::symlink_metadata(dest).is_ok() {
        return Err(error(
            io::ErrorKind::AlreadyExists,
            "An item already has that name",
        ));
    }
    fs::rename(source, dest)
}

fn copy(source: &Path, dest: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.is_dir() {
        fs::create_dir(dest)?;
        let result = (|| {
            for item in fs::read_dir(source)? {
                let item = item?;
                copy(&item.path(), &dest.join(item.file_name()))?;
            }
            fs::set_permissions(dest, metadata.permissions())
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(dest);
        }
        result
    } else if metadata.is_file() {
        let mut input = File::open(source)?;
        let mut output = OpenOptions::new().write(true).create_new(true).open(dest)?;
        let result = io::copy(&mut input, &mut output)
            .and_then(|_| output.set_permissions(metadata.permissions()))
            .and_then(|_| output.sync_all());
        if result.is_err() {
            let _ = fs::remove_file(dest);
        }
        result
    } else {
        #[cfg(unix)]
        if metadata.file_type().is_symlink() {
            return std::os::unix::fs::symlink(fs::read_link(source)?, dest);
        }
        Err(error(
            io::ErrorKind::Unsupported,
            "This file type cannot be copied",
        ))
    }
}

pub(crate) fn transfer(source: &Path, dest: &Path, moving: bool) -> io::Result<()> {
    let parent = dest
        .parent()
        .ok_or_else(|| error(io::ErrorKind::InvalidInput, "Choose a destination folder"))?;
    if fs::symlink_metadata(source)?.is_dir()
        && parent.canonicalize()?.starts_with(source.canonicalize()?)
    {
        return Err(error(
            io::ErrorKind::InvalidInput,
            "A folder cannot contain itself",
        ));
    }
    if moving {
        match rename(source, dest) {
            Ok(()) => return Ok(()),
            Err(err) if err.raw_os_error() == Some(18) => {}
            Err(err) => return Err(err),
        }
    }
    copy(source, dest)?;
    if moving {
        remove(source)?;
    }
    Ok(())
}

fn candidate_name(path: &Path, count: usize) -> PathBuf {
    let mut name = path.file_stem().unwrap_or_default().to_os_string();
    name.push(format!(" ({count})"));
    if let Some(ext) = path.extension() {
        name.push(".");
        name.push(ext);
    }
    path.with_file_name(name)
}

pub(crate) fn unique_path(path: PathBuf) -> PathBuf {
    if fs::symlink_metadata(&path).is_err() {
        return path;
    }
    (2..)
        .map(|count| candidate_name(&path, count))
        .find(|candidate| fs::symlink_metadata(candidate).is_err())
        .unwrap()
}

pub(crate) fn trash_root() -> Option<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
        })?;
    Some(data.join("Trash"))
}

fn trash_info(root: &Path, entry: &Path) -> io::Result<PathBuf> {
    let name = entry
        .file_name()
        .ok_or_else(|| error(io::ErrorKind::InvalidInput, "Invalid trash entry"))?;
    let mut info_name = name.to_os_string();
    info_name.push(".trashinfo");
    Ok(root.join("info").join(info_name))
}

pub fn uri_path(path: &Path) -> String {
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes()
    };
    #[cfg(not(unix))]
    let display = path.to_string_lossy();
    #[cfg(not(unix))]
    let bytes = display.as_bytes();
    bytes
        .iter()
        .map(|&b| match b {
            b'/' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            b if b.is_ascii_alphanumeric() => (b as char).to_string(),
            b => format!("%{b:02X}"),
        })
        .collect()
}

fn decode_uri(value: &str) -> io::Result<PathBuf> {
    let mut bytes = Vec::new();
    let mut chars = value.bytes();
    while let Some(byte) = chars.next() {
        if byte != b'%' {
            bytes.push(byte);
            continue;
        }
        let nibble = |byte: Option<u8>| byte.and_then(|b| (b as char).to_digit(16));
        let decoded = nibble(chars.next())
            .zip(nibble(chars.next()))
            .map(|(a, b)| (a * 16 + b) as u8)
            .ok_or_else(|| error(io::ErrorKind::InvalidData, "Invalid trash metadata"))?;
        bytes.push(decoded);
    }
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(std::ffi::OsString::from_vec(bytes))
    };
    #[cfg(not(unix))]
    let path = PathBuf::from(String::from_utf8_lossy(&bytes).into_owned());
    if !path.is_absolute() {
        return Err(error(
            io::ErrorKind::InvalidData,
            "Trash origin must be an absolute path",
        ));
    }
    Ok(path)
}

fn deletion_date() -> String {
    #[cfg(unix)]
    {
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs()) as libc::time_t;
        let mut time = std::mem::MaybeUninit::<libc::tm>::uninit();
        // SAFETY: localtime_r initializes the provided storage on success.
        let time = unsafe {
            if libc::localtime_r(&seconds, time.as_mut_ptr()).is_null() {
                return "1970-01-01T00:00:00".into();
            }
            time.assume_init()
        };
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            time.tm_year + 1900,
            time.tm_mon + 1,
            time.tm_mday,
            time.tm_hour,
            time.tm_min,
            time.tm_sec
        )
    }
    #[cfg(not(unix))]
    "1970-01-01T00:00:00".into()
}

pub(crate) fn trash(root: &Path, path: &Path) -> io::Result<()> {
    let files = root.join("files");
    fs::create_dir_all(&files)?;
    fs::create_dir_all(root.join("info"))?;
    let name = path
        .file_name()
        .ok_or_else(|| error(io::ErrorKind::InvalidInput, "Cannot trash the storage root"))?;
    let proposed = files.join(name);
    for count in 1.. {
        let dest = if count == 1 {
            proposed.clone()
        } else {
            candidate_name(&proposed, count)
        };
        if fs::symlink_metadata(&dest).is_ok() {
            continue;
        }
        let info = trash_info(root, &dest)?;
        match OpenOptions::new().write(true).create_new(true).open(&info) {
            Ok(mut file) => {
                let result = writeln!(
                    file,
                    "[Trash Info]\nPath={}\nDeletionDate={}",
                    uri_path(path),
                    deletion_date()
                )
                .and_then(|_| file.sync_all())
                .and_then(|_| transfer(path, &dest, true));
                if result.is_err() && fs::symlink_metadata(&dest).is_err() {
                    let _ = fs::remove_file(&info);
                }
                return result;
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    unreachable!()
}

pub(crate) fn restore(root: &Path, path: &Path) -> io::Result<()> {
    let info = trash_info(root, path)?;
    let data = fs::read_to_string(&info)?;
    let dest = decode_uri(
        data.lines()
            .find_map(|line| line.strip_prefix("Path="))
            .ok_or_else(|| error(io::ErrorKind::InvalidData, "Trash origin is missing"))?,
    )?;
    transfer(path, &dest, true)?;
    fs::remove_file(info)
}

pub(crate) fn empty_entry(root: &Path, path: &Path) -> io::Result<()> {
    remove(path)?;
    let info = trash_info(root, path)?;
    match fs::remove_file(info) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

pub(crate) fn compress(paths: &[PathBuf], target: &Path) -> io::Result<()> {
    use zip::write::SimpleFileOptions;
    fn append(zip: &mut zip::ZipWriter<File>, path: &Path, name: &str) -> io::Result<()> {
        let metadata = fs::symlink_metadata(path)?;
        let options = SimpleFileOptions::default();
        if metadata.is_dir() {
            zip.add_directory(format!("{name}/"), options)?;
            for child in fs::read_dir(path)? {
                let child = child?;
                append(
                    zip,
                    &child.path(),
                    &format!("{name}/{}", child.file_name().to_string_lossy()),
                )?;
            }
        } else if metadata.is_file() {
            let options = options.compression_method(zip::CompressionMethod::Deflated);
            zip.start_file(name, options)?;
            io::copy(&mut File::open(path)?, zip)?;
        } else {
            return Err(error(
                io::ErrorKind::Unsupported,
                "Archive items must be files or folders",
            ));
        }
        Ok(())
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)?;
    let result = (|| {
        let mut zip = zip::ZipWriter::new(file);
        for path in paths {
            let name = path.file_name().ok_or_else(|| {
                error(io::ErrorKind::InvalidInput, "Cannot archive a storage root")
            })?;
            append(&mut zip, path, &name.to_string_lossy())?;
        }
        zip.finish()?.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(target);
    }
    result
}

pub(crate) fn extract(archive: &Path, dest: &Path) -> io::Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    fs::create_dir(dest)?;
    let result = (|| {
        for index in 0..zip.len() {
            let mut item = zip.by_index(index)?;
            let relative = item.enclosed_name().ok_or_else(|| {
                error(
                    io::ErrorKind::InvalidData,
                    "Archive contains an invalid path",
                )
            })?;
            let target = dest.join(relative);
            if item.is_dir() {
                fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(target)?;
                io::copy(&mut item, &mut output)?;
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(dest);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);

    impl Fixture {
        fn new(label: &str) -> Self {
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "spectre-files-{label}-{}-{time}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn item(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn copy_move_and_rename_preserve_contents_and_refuse_replacement() {
        let fixture = Fixture::new("transfer");
        let source = fixture.item("source");
        let copy = fixture.item("copy");
        let renamed = fixture.item("renamed");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("text.txt"), b"keep me").unwrap();
        assert!(transfer(&source, &source.join("inside"), false).is_err());
        transfer(&source, &copy, false).unwrap();
        assert_eq!(fs::read(copy.join("text.txt")).unwrap(), b"keep me");
        assert_eq!(
            transfer(&source, &copy, true).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        rename(&source, &renamed).unwrap();
        assert!(!source.exists());
        assert_eq!(fs::read(renamed.join("text.txt")).unwrap(), b"keep me");
        assert!(child(&fixture.0, "../unrelated").is_err());
    }

    #[test]
    fn trash_and_restore_round_trip_escaped_names() {
        let fixture = Fixture::new("trash");
        let trash_root = fixture.item("Trash");
        let source = fixture.item("demo % #.txt");
        fs::write(&source, "test").unwrap();
        trash(&trash_root, &source).unwrap();
        assert!(!source.exists());
        let trashed = trash_root.join("files").join("demo % #.txt");
        assert_eq!(fs::read_to_string(&trashed).unwrap(), "test");
        restore(&trash_root, &trashed).unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "test");
        assert!(!trashed.exists());
    }

    #[test]
    fn archive_round_trip_and_extract_rejects_path_traversal() {
        let fixture = Fixture::new("archive");
        let file = fixture.item("demo.txt");
        fs::write(&file, "hello").unwrap();
        let zip = fixture.item("demo.zip");
        let directory = fixture.item("extracted");
        compress(&[file], &zip).unwrap();
        extract(&zip, &directory).unwrap();
        assert_eq!(
            fs::read_to_string(directory.join("demo.txt")).unwrap(),
            "hello"
        );

        let zip_path = fixture.item("invalid.zip");
        let mut zip = zip::ZipWriter::new(File::create(&zip_path).unwrap());
        zip.start_file("../unrelated.txt", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"must stay in zip").unwrap();
        zip.finish().unwrap();
        assert!(extract(&zip_path, &fixture.item("invalid")).is_err());
        assert!(!fixture.item("unrelated.txt").exists());
        assert!(!fixture.item("invalid").exists());
    }
}
