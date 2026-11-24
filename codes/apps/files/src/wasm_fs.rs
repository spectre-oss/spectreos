//! In-memory virtual SpectreOS filesystem compiled ONLY on `wasm32` (`#[cfg(target_arch = "wasm32")]`).
//!
//! Zero bytes are compiled into native Linux / ARM64 phone builds. Allows debugging the real
//! `spectre-files` UI, navigation, selection, search, sorting, grid/list views, image previews,
//! create/rename/copy/move/trash/restore workflows, and file-picker modes directly in the browser.

use crate::browser::{Entry, Filter, Location, PickerRequest, file_name};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BASE_SECS: u64 = 1_795_400_000; // 2026-11-23 UTC

#[derive(Clone)]
enum Node {
    Dir {
        modified_secs: u64,
    },
    File {
        bytes: u64,
        modified_secs: u64,
        image_seed: Option<u8>,
    },
}

struct Vfs {
    nodes: BTreeMap<PathBuf, Node>,
    trash_origin: BTreeMap<PathBuf, PathBuf>,
    tick: u64,
}

impl Vfs {
    fn seeded() -> Self {
        let mut vfs = Self {
            nodes: BTreeMap::new(),
            trash_origin: BTreeMap::new(),
            tick: BASE_SECS,
        };
        for dir in [
            "/",
            "/home",
            "/home/spectre",
            "/home/spectre/Downloads",
            "/home/spectre/Documents",
            "/home/spectre/Documents/SpectreOS",
            "/home/spectre/Pictures",
            "/home/spectre/Pictures/Wallpapers",
            "/home/spectre/Music",
            "/home/spectre/Videos",
            "/home/spectre/Desktop",
            "/home/spectre/.config",
            "/home/spectre/.local",
            "/home/spectre/.local/share",
            "/home/spectre/.local/share/Trash",
            "/home/spectre/.local/share/Trash/files",
        ] {
            vfs.nodes.insert(
                PathBuf::from(dir),
                Node::Dir {
                    modified_secs: BASE_SECS - 3600,
                },
            );
        }

        let files: &[(&str, u64, u64, Option<u8>)] = &[
            ("/home/spectre/Pictures/Wallpapers/aurora-ridge.png", 1_842_112, 120, Some(1)),
            ("/home/spectre/Pictures/Wallpapers/desert-dusk.jpg", 2_310_400, 900, Some(2)),
            ("/home/spectre/Pictures/poco-f1-wisp.png", 945_200, 1_800, Some(3)),
            ("/home/spectre/Pictures/persian-keyboard.webp", 412_880, 3_600, Some(4)),
            ("/home/spectre/Downloads/spectre-release-notes.pdf", 648_192, 600, None),
            ("/home/spectre/Downloads/shiraz-sample.apk", 14_820_110, 4_200, None),
            ("/home/spectre/Downloads/backup-2026-11.zip", 8_940_032, 7_200, None),
            ("/home/spectre/Documents/todo.md", 3_420, 300, None),
            ("/home/spectre/Documents/budget-2026.csv", 18_432, 86_400, None),
            ("/home/spectre/Documents/SpectreOS/architecture.txt", 24_576, 12_400, None),
            ("/home/spectre/Documents/SpectreOS/keyboard-spec.md", 11_264, 15_000, None),
            ("/home/spectre/Music/night-drive.flac", 31_457_280, 172_800, None),
            ("/home/spectre/Music/voice-memo.ogg", 1_240_512, 25_000, None),
            ("/home/spectre/Videos/wisp-60fps-capture.mp4", 48_234_496, 5_400, None),
            ("/home/spectre/Desktop/quick-note.txt", 640, 200, None),
            ("/home/spectre/.config/wisp.conf", 1_024, 9_000, None),
            ("/home/spectre/.bashrc", 2_048, 200_000, None),
        ];
        for &(path, bytes, age, seed) in files {
            vfs.nodes.insert(
                PathBuf::from(path),
                Node::File {
                    bytes,
                    modified_secs: BASE_SECS.saturating_sub(age),
                    image_seed: seed,
                },
            );
        }

        // Seed one trashed file so Trash + Restore can be debugged immediately.
        let trashed = PathBuf::from("/home/spectre/.local/share/Trash/files/old-draft.txt");
        vfs.nodes.insert(
            trashed.clone(),
            Node::File {
                bytes: 4_096,
                modified_secs: BASE_SECS - 43_200,
                image_seed: None,
            },
        );
        vfs.trash_origin
            .insert(trashed, PathBuf::from("/home/spectre/Documents/old-draft.txt"));

        vfs
    }

    fn next_time(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }

    fn to_system_time(secs: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn is_dir(&self, path: &Path) -> bool {
        matches!(self.nodes.get(path), Some(Node::Dir { .. }))
    }

    fn is_file(&self, path: &Path) -> bool {
        matches!(self.nodes.get(path), Some(Node::File { .. }))
    }

    fn exists(&self, path: &Path) -> bool {
        self.nodes.contains_key(path)
    }

    fn ensure_parents(&mut self, path: &Path) {
        let mut ancestors = Vec::new();
        let mut cur = path.parent();
        while let Some(p) = cur {
            if self.nodes.contains_key(p) {
                break;
            }
            ancestors.push(p.to_path_buf());
            cur = p.parent();
        }
        let now = self.next_time();
        for p in ancestors.into_iter().rev() {
            self.nodes.insert(p, Node::Dir { modified_secs: now });
        }
    }

    fn unique_path(&self, path: PathBuf) -> PathBuf {
        if !self.exists(&path) {
            return path;
        }
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let ext = path.extension().map(|e| e.to_string_lossy().into_owned());
        for count in 2.. {
            let name = match &ext {
                Some(e) => format!("{stem} ({count}).{e}"),
                None => format!("{stem} ({count})"),
            };
            let candidate = path.with_file_name(name);
            if !self.exists(&candidate) {
                return candidate;
            }
        }
        path
    }
}

thread_local! {
    static VFS: RefCell<Vfs> = RefCell::new(Vfs::seeded());
}

pub(crate) fn reset_default() {
    VFS.with_borrow_mut(|v| *v = Vfs::seeded());
}

pub(crate) fn default_home() -> PathBuf {
    PathBuf::from("/home/spectre")
}

pub(crate) fn is_dir(path: &Path) -> bool {
    VFS.with_borrow(|v| v.is_dir(path))
}

pub(crate) fn is_file(path: &Path) -> bool {
    VFS.with_borrow(|v| v.is_file(path))
}

pub(crate) fn exists(path: &Path) -> bool {
    VFS.with_borrow(|v| v.exists(path))
}

pub(crate) fn create_dir(path: &Path) -> io::Result<()> {
    VFS.with_borrow_mut(|v| {
        if v.exists(path) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "An item already has that name",
            ));
        }
        let Some(parent) = path.parent() else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid folder"));
        };
        if !v.is_dir(parent) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Parent folder does not exist",
            ));
        }
        let modified_secs = v.next_time();
        v.nodes.insert(path.to_path_buf(), Node::Dir { modified_secs });
        Ok(())
    })
}

pub(crate) fn create_new_file(path: &Path) -> io::Result<bool> {
    VFS.with_borrow_mut(|v| {
        if let Some(node) = v.nodes.get(path) {
            return match node {
                Node::File { .. } => Ok(true), // already exists as file -> prompt Replace
                Node::Dir { .. } => Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "A folder already has that name",
                )),
            };
        }
        let modified_secs = v.next_time();
        v.ensure_parents(path);
        v.nodes.insert(
            path.to_path_buf(),
            Node::File {
                bytes: 0,
                modified_secs,
                image_seed: None,
            },
        );
        Ok(false)
    })
}

pub(crate) fn rename(source: &Path, dest: &Path) -> io::Result<()> {
    transfer(source, dest, true)
}

pub(crate) fn transfer(source: &Path, dest: &Path, moving: bool) -> io::Result<()> {
    VFS.with_borrow_mut(|v| {
        if !v.exists(source) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Item not found"));
        }
        if v.exists(dest) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "An item already has that name",
            ));
        }
        if dest.starts_with(source) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "A folder cannot contain itself",
            ));
        }
        let to_copy: Vec<(PathBuf, Node)> = v
            .nodes
            .iter()
            .filter(|(p, _)| p.as_path() == source || p.starts_with(source))
            .map(|(p, n)| (p.clone(), n.clone()))
            .collect();
        let now = v.next_time();
        for (old_path, mut node) in to_copy {
            let rel = old_path.strip_prefix(source).unwrap_or(Path::new(""));
            let new_path = if rel.as_os_str().is_empty() {
                dest.to_path_buf()
            } else {
                dest.join(rel)
            };
            match &mut node {
                Node::Dir { modified_secs } | Node::File { modified_secs, .. } => {
                    *modified_secs = now;
                }
            }
            v.nodes.insert(new_path, node);
            if moving {
                v.nodes.remove(&old_path);
            }
        }
        Ok(())
    })
}

pub(crate) fn transfer_into(source: &Path, dest_dir: &Path, moving: bool) -> io::Result<()> {
    let name = source
        .file_name()
        .ok_or_else(|| io::Error::other("Invalid item"))?;
    if moving && source.parent() == Some(dest_dir) {
        return Ok(());
    }
    let dest = VFS.with_borrow(|v| v.unique_path(dest_dir.join(name)));
    transfer(source, &dest, moving)
}

pub(crate) fn trash(path: &Path) -> io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Cannot trash storage root"))?;
    let trash_files = PathBuf::from("/home/spectre/.local/share/Trash/files");
    let dest = VFS.with_borrow(|v| v.unique_path(trash_files.join(name)));
    transfer(path, &dest, true)?;
    VFS.with_borrow_mut(|v| {
        v.trash_origin.insert(dest, path.to_path_buf());
    });
    Ok(())
}

pub(crate) fn restore(path: &Path) -> io::Result<()> {
    let dest = VFS.with_borrow(|v| {
        v.trash_origin.get(path).cloned().unwrap_or_else(|| {
            let name = path.file_name().unwrap_or_default();
            PathBuf::from("/home/spectre").join(name)
        })
    });
    let final_dest = VFS.with_borrow(|v| v.unique_path(dest));
    VFS.with_borrow_mut(|v| v.ensure_parents(&final_dest));
    transfer(path, &final_dest, true)?;
    VFS.with_borrow_mut(|v| {
        v.trash_origin.remove(path);
    });
    Ok(())
}

pub(crate) fn empty_entry(path: &Path) -> io::Result<()> {
    VFS.with_borrow_mut(|v| {
        let keys: Vec<PathBuf> = v
            .nodes
            .keys()
            .filter(|p| p.as_path() == path || p.starts_with(path))
            .cloned()
            .collect();
        if keys.is_empty() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Item not found"));
        }
        for k in keys {
            v.nodes.remove(&k);
            v.trash_origin.remove(&k);
        }
        Ok(())
    })
}

pub(crate) fn compress(paths: &[PathBuf], target: &Path) -> io::Result<()> {
    VFS.with_borrow_mut(|v| {
        if v.exists(target) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "An archive already has that name",
            ));
        }
        let total: u64 = paths
            .iter()
            .filter_map(|p| match v.nodes.get(p) {
                Some(Node::File { bytes, .. }) => Some((*bytes / 2).max(256)),
                Some(Node::Dir { .. }) => Some(4_096),
                None => None,
            })
            .sum();
        let modified_secs = v.next_time();
        v.nodes.insert(
            target.to_path_buf(),
            Node::File {
                bytes: total.max(512),
                modified_secs,
                image_seed: None,
            },
        );
        Ok(())
    })
}

pub(crate) fn extract(archive: &Path) -> io::Result<()> {
    let title = archive
        .file_stem()
        .ok_or_else(|| io::Error::other("Invalid archive name"))?;
    let dest = VFS.with_borrow(|v| v.unique_path(archive.with_file_name(title)));
    create_dir(&dest)?;
    VFS.with_borrow_mut(|v| {
        let now = v.next_time();
        v.nodes.insert(
            dest.join("extracted-readme.txt"),
            Node::File {
                bytes: 1_280,
                modified_secs: now,
                image_seed: None,
            },
        );
    });
    Ok(())
}

pub(crate) fn read_location(
    location: &Location,
    home: &Path,
    query: &str,
    filter: &Filter,
    picker: Option<&PickerRequest>,
    hidden: bool,
) -> io::Result<Vec<Entry>> {
    VFS.with_borrow(|v| {
        let bases: Vec<PathBuf> = match location {
            Location::Directory(path) => {
                if !v.is_dir(path) {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "Folder does not exist",
                    ));
                }
                vec![path.clone()]
            }
            Location::Trash => vec![PathBuf::from("/home/spectre/.local/share/Trash/files")],
            Location::Recent | Location::Category(_) => vec![
                home.join("Downloads"),
                home.join("Documents"),
                home.join("Pictures"),
                home.join("Music"),
                home.join("Videos"),
                home.join("Desktop"),
            ],
        };
        let collection = matches!(location, Location::Recent | Location::Category(_));
        let recurse = collection || (!query.is_empty() && !matches!(location, Location::Trash));

        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for (path, node) in &v.nodes {
            let Some(parent) = path.parent() else {
                continue;
            };
            let inside = if recurse {
                bases.iter().any(|b| path != b && path.starts_with(b))
            } else {
                bases.iter().any(|b| parent == b.as_path())
            };
            if !inside || !seen.insert(path.clone()) {
                continue;
            }
            let name = file_name(path);
            if !hidden && name.starts_with('.') {
                continue;
            }
            if recurse
                && !hidden
                && path
                    .components()
                    .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            {
                continue;
            }
            let (directory, bytes, modified_secs) = match *node {
                Node::Dir { modified_secs } => (true, 0, modified_secs),
                Node::File {
                    bytes,
                    modified_secs,
                    ..
                } => (false, bytes, modified_secs),
            };
            if directory && collection {
                continue;
            }
            let matches_picker = match picker {
                Some(PickerRequest::Folder) => directory,
                Some(PickerRequest::Open { filter, .. }) => directory || filter.accepts(path),
                _ => true,
            };
            let matches_category =
                !matches!(location, Location::Category(kind) if !kind.accepts(path));
            if matches_picker
                && matches_category
                && (directory || filter.accepts(path))
                && (query.is_empty() || name.to_lowercase().contains(query))
            {
                out.push(Entry {
                    path: path.clone(),
                    name,
                    directory,
                    bytes,
                    modified: Some(Vfs::to_system_time(modified_secs)),
                });
            }
        }
        if matches!(location, Location::Recent) && query.is_empty() {
            out.sort_by_key(|e| std::cmp::Reverse(e.modified));
            out.truncate(150);
        }
        Ok(out)
    })
}

/// Generate a crisp procedural RGBA preview thumbnail for virtual image files on `wasm32`.
pub(crate) fn thumbnail(path: &Path) -> Option<bhoo::Image> {
    let seed = VFS.with_borrow(|v| match v.nodes.get(path) {
        Some(Node::File { image_seed, .. }) => *image_seed,
        _ => None,
    })?;
    let w = 160u32;
    let h = 112u32;
    let mut pixels = vec![0u8; (w * h * 4) as usize];
    let (c0, c1, accent) = match seed {
        1 => ([18u8, 36, 72], [38u8, 168, 148], [176u8, 245, 224]),
        2 => ([64u8, 28, 66], [234u8, 128, 68], [255u8, 220, 152]),
        3 => ([24u8, 30, 48], [84u8, 108, 198], [208u8, 188, 255]),
        _ => ([32u8, 42, 48], [76u8, 134, 114], [220u8, 242, 232]),
    };
    for y in 0..h {
        let ty = y as f32 / (h - 1) as f32;
        for x in 0..w {
            let tx = x as f32 / (w - 1) as f32;
            let wave = ((tx * 6.0 + seed as f32).sin() * 0.14 + 0.56).clamp(0.0, 1.0);
            let idx = ((y * w + x) * 4) as usize;
            let mix = |a: u8, b: u8, t: f32| (a as f32 + (b as f32 - a as f32) * t) as u8;
            if ty > wave {
                let mt = ((ty - wave) / (1.0 - wave).max(0.01)).clamp(0.0, 1.0);
                pixels[idx] = mix(c0[0], 14, mt);
                pixels[idx + 1] = mix(c0[1], 18, mt);
                pixels[idx + 2] = mix(c0[2], 26, mt);
            } else {
                let d = ((tx - 0.72).powi(2) + (ty - 0.28).powi(2)).sqrt();
                let glow = (1.0 - d / 0.28).clamp(0.0, 1.0);
                pixels[idx] = mix(mix(c0[0], c1[0], ty), accent[0], glow * 0.85);
                pixels[idx + 1] = mix(mix(c0[1], c1[1], ty), accent[1], glow * 0.85);
                pixels[idx + 2] = mix(mix(c0[2], c1[2], ty), accent[2], glow * 0.85);
            }
            pixels[idx + 3] = 255;
        }
    }
    bhoo::Image::rgba(w, h, pixels)
}
