use crate::operations as fs_ops;
use std::collections::{BTreeSet, HashMap};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{self, OpenOptions};
#[cfg(target_arch = "wasm32")]
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::SystemTime;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    Any,
    Images,
    Videos,
    Audio,
    Documents,
    Archives,
    Extensions(Vec<String>),
}

impl Filter {
    pub fn accepts(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let known: &[&str] = match self {
            Self::Any => return true,
            Self::Images => &["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"],
            Self::Videos => &[
                "mp4", "mkv", "mov", "webm", "avi", "m4v", "3gp", "mpeg", "ts",
            ],
            Self::Audio => &["mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "wma"],
            Self::Documents => &[
                "pdf", "txt", "md", "odt", "ods", "doc", "docx", "xlsx", "pptx", "csv", "rtf",
                "epub", "json", "xml",
            ],
            Self::Archives => &["zip", "tar", "gz", "xz", "7z", "rar", "bz2"],
            Self::Extensions(values) => {
                return values
                    .iter()
                    .any(|value| value.trim_start_matches('.').eq_ignore_ascii_case(&ext));
            }
        };
        known.contains(&ext.as_str())
    }
}

#[derive(Clone, Debug)]
pub enum PickerRequest {
    Open { filter: Filter, multiple: bool },
    Folder,
    Save { suggested_name: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Open(PathBuf),
    Share(Vec<PathBuf>),
    NewWindow(Option<PathBuf>),
    Selected(Vec<PathBuf>),
    Cancelled,
    Focus(u64),
    Blur,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Location {
    Recent,
    Category(Filter),
    Directory(PathBuf),
    Trash,
}

#[derive(Clone, Debug)]
pub(crate) struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub directory: bool,
    pub bytes: u64,
    pub modified: Option<SystemTime>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Order {
    Name,
    Modified,
    Size,
    Kind,
}

#[derive(Clone)]
pub(crate) enum Modal {
    NewFolder,
    Rename(PathBuf),
    Compress(Vec<PathBuf>),
    Delete(Vec<PathBuf>),
    Details(Entry),
    Preview(Entry),
    Replace(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClipboardMode {
    Copy,
    Move,
}

#[derive(Clone, Debug)]
pub(crate) struct JobReport {
    done: usize,
    errors: Vec<String>,
    failed: Vec<PathBuf>,
}

pub struct Browser {
    pub(crate) home: PathBuf,
    pub(crate) roots: Vec<(String, Location)>,
    pub(crate) location: Location,
    history: Vec<Location>,
    pub(crate) entries: Vec<Entry>,
    pub(crate) selection: BTreeSet<PathBuf>,
    clipboard: Option<(ClipboardMode, Vec<PathBuf>)>,
    pub(crate) request: Option<PickerRequest>,
    pub(crate) filter: Filter,
    pub(crate) order: Order,
    pub(crate) descending: bool,
    pub(crate) grid: bool,
    pub(crate) hidden: bool,
    pub(crate) nav_open: bool,
    pub(crate) menu_open: bool,
    pub(crate) select_mode: bool,
    pub(crate) search_open: bool,
    pub(crate) search: String,
    pub(crate) edit: String,
    pub(crate) save_name: String,
    pub(crate) scroll: f32,
    pub(crate) modal: Option<Modal>,
    pub(crate) message: Option<String>,
    scan: Option<Receiver<io::Result<Vec<Entry>>>>,
    scan_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    job: Option<Receiver<JobReport>>,
    pub(crate) pictures: HashMap<PathBuf, bhoo::Image>,
    thumbnails: Option<Receiver<(PathBuf, Option<bhoo::Image>)>>,
    thumbnail_requests: BTreeSet<PathBuf>,
    initialized: bool,
}

impl Default for Browser {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let home = crate::wasm_fs::default_home();
        #[cfg(not(target_arch = "wasm32"))]
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| PathBuf::from("/"));
        Self::new(home)
    }
}

impl Browser {
    pub fn new(home: PathBuf) -> Self {
        Self {
            roots: discover_roots(&home),
            home,
            location: Location::Recent,
            history: Vec::new(),
            entries: Vec::new(),
            selection: BTreeSet::new(),
            clipboard: None,
            request: None,
            filter: Filter::Any,
            order: Order::Modified,
            descending: true,
            grid: false,
            hidden: false,
            nav_open: false,
            menu_open: false,
            select_mode: false,
            search_open: false,
            search: String::new(),
            edit: String::new(),
            save_name: String::new(),
            scroll: 0.0,
            modal: None,
            message: None,
            scan: None,
            scan_cancel: None,
            job: None,
            pictures: HashMap::new(),
            thumbnails: None,
            thumbnail_requests: BTreeSet::new(),
            initialized: false,
        }
    }

    pub fn picker(request: PickerRequest) -> Self {
        let mut browser = Self::default();
        browser.set_picker(request);
        browser
    }

    pub fn set_picker(&mut self, request: PickerRequest) {
        let location = match &request {
            PickerRequest::Save { suggested_name } => {
                self.save_name = suggested_name.clone();
                let downloads = user_directory(&self.home, "DOWNLOAD", "Downloads");
                #[cfg(not(target_arch = "wasm32"))]
                let has_downloads = downloads.is_dir();
                #[cfg(target_arch = "wasm32")]
                let has_downloads = crate::wasm_fs::is_dir(&downloads);
                Location::Directory(if has_downloads {
                    downloads
                } else {
                    self.home.clone()
                })
            }
            PickerRequest::Folder => Location::Directory(self.home.clone()),
            PickerRequest::Open {
                filter: Filter::Images,
                ..
            } => Location::Category(Filter::Images),
            _ => Location::Recent,
        };
        self.grid = matches!(
            request,
            PickerRequest::Open {
                filter: Filter::Images,
                ..
            }
        );
        self.request = Some(request);
        self.history.clear();
        self.filter = Filter::Any;
        self.change_location(location);
    }

    pub fn set_start_folder(&mut self, path: PathBuf) -> io::Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        let ok = path.is_dir();
        #[cfg(target_arch = "wasm32")]
        let ok = crate::wasm_fs::is_dir(&path);
        if !ok {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "The starting folder does not exist",
            ));
        }
        self.change_location(Location::Directory(path));
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn reset_wasm_demo_fs() {
        crate::wasm_fs::reset_default();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn debug_state_json(&self) -> String {
        let loc = match &self.location {
            Location::Recent => "Recent".to_string(),
            Location::Category(f) => format!("Category({f:?})"),
            Location::Directory(p) => p.display().to_string(),
            Location::Trash => "Trash".to_string(),
        };
        let modal = match &self.modal {
            None => "null".to_string(),
            Some(Modal::NewFolder) => "\"NewFolder\"".to_string(),
            Some(Modal::Rename(p)) => format!("\"Rename({})\"", p.display()),
            Some(Modal::Compress(ps)) => format!("\"Compress({})\"", ps.len()),
            Some(Modal::Delete(ps)) => format!("\"Delete({})\"", ps.len()),
            Some(Modal::Details(e)) => format!("\"Details({})\"", e.name),
            Some(Modal::Preview(e)) => format!("\"Preview({})\"", e.name),
            Some(Modal::Replace(p)) => format!("\"Replace({})\"", p.display()),
        };
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "{{\"location\":\"{}\",\"picker\":{},\"entries\":{},\"selected\":{},\"grid\":{},\"hidden\":{},\"searchOpen\":{},\"search\":\"{}\",\"navOpen\":{},\"menuOpen\":{},\"selectMode\":{},\"modal\":{},\"scroll\":{:.1},\"message\":{}}}",
            esc(&loc),
            self.request.is_some(),
            self.entries.len(),
            self.selection.len(),
            self.grid,
            self.hidden,
            self.search_open,
            esc(&self.search),
            self.nav_open,
            self.menu_open,
            self.select_mode,
            modal,
            self.scroll,
            match &self.message {
                Some(m) => format!("\"{}\"", esc(m)),
                None => "null".to_string(),
            }
        )
    }

    pub fn report(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
    }

    pub fn edit_text(&self, id: u64) -> Option<&str> {
        match id {
            crate::SEARCH_FIELD_ID => Some(&self.search),
            crate::EDIT_FIELD_ID if self.modal.is_some() => Some(&self.edit),
            crate::EDIT_FIELD_ID => Some(&self.save_name),
            _ => None,
        }
    }

    pub fn set_edit_text(&mut self, id: u64, value: &str) -> bool {
        match id {
            crate::SEARCH_FIELD_ID => {
                if self.search != value {
                    self.search = value.to_owned();
                    self.scroll = 0.0;
                    self.reload();
                }
            }
            crate::EDIT_FIELD_ID if self.modal.is_some() => self.edit = value.to_owned(),
            crate::EDIT_FIELD_ID => self.save_name = value.to_owned(),
            _ => return false,
        }
        true
    }

    pub fn back(&mut self) -> bool {
        if self.modal.take().is_some() {
            return true;
        }
        if self.menu_open {
            self.menu_open = false;
            return true;
        }
        if self.nav_open {
            self.nav_open = false;
            return true;
        }
        if self.select_mode || !self.selection.is_empty() {
            self.select_mode = false;
            self.selection.clear();
            return true;
        }
        if self.search_open {
            self.search_open = false;
            self.search.clear();
            self.reload();
            return true;
        }
        if let Some(location) = self.history.pop() {
            self.change_location(location);
            return true;
        }
        false
    }

    pub(crate) fn go(&mut self, location: Location) {
        if self.location != location {
            self.history.push(self.location.clone());
        }
        self.change_location(location);
    }

    pub(crate) fn root(&mut self, location: Location) {
        self.history.clear();
        self.change_location(location);
    }

    fn change_location(&mut self, location: Location) {
        self.location = location;
        self.selection.clear();
        self.select_mode = false;
        self.search.clear();
        self.search_open = false;
        self.nav_open = false;
        self.menu_open = false;
        self.scroll = 0.0;
        if matches!(self.location, Location::Recent) {
            self.order = Order::Modified;
            self.descending = true;
        }
        self.reload();
    }

    pub(crate) fn directory(&self) -> Option<&Path> {
        match &self.location {
            Location::Directory(path) => Some(path),
            _ => None,
        }
    }

    pub(crate) fn allowed(&self, entry: &Entry) -> bool {
        if entry.directory {
            return true;
        }
        match &self.request {
            Some(PickerRequest::Open { filter, .. }) => filter.accepts(&entry.path),
            Some(PickerRequest::Folder) => false,
            _ => true,
        }
    }

    pub(crate) fn multiple(&self) -> bool {
        matches!(
            self.request,
            None | Some(PickerRequest::Open { multiple: true, .. })
        )
    }

    pub(crate) fn pick(&mut self, entry: Entry) -> Action {
        if self.select_mode || !self.selection.is_empty() {
            self.toggle(entry.path);
            return Action::None;
        }
        if entry.directory && self.location != Location::Trash {
            self.go(Location::Directory(entry.path));
            return Action::None;
        }
        if self.location == Location::Trash {
            self.selection.insert(entry.path);
            return Action::None;
        }
        match self.request {
            Some(PickerRequest::Open { .. }) if self.allowed(&entry) => {
                Action::Selected(vec![entry.path])
            }
            Some(PickerRequest::Save { .. }) => {
                self.save_name = entry.name;
                Action::None
            }
            Some(_) => Action::None,
            None if Filter::Images.accepts(&entry.path) => {
                self.modal = Some(Modal::Preview(entry));
                Action::None
            }
            None => Action::Open(entry.path),
        }
    }

    pub(crate) fn confirm_picker(&mut self) -> Action {
        match self.request {
            Some(PickerRequest::Folder) => self
                .directory()
                .map_or(Action::None, |dir| Action::Selected(vec![dir.to_owned()])),
            Some(PickerRequest::Save { .. }) => {
                let result = self
                    .directory()
                    .ok_or_else(|| io::Error::other("Choose a folder"))
                    .and_then(|dir| fs_ops::child(dir, &self.save_name))
                    .and_then(|path| {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let exists = crate::wasm_fs::create_new_file(&path)?;
                            Ok((path, exists))
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        match OpenOptions::new().write(true).create_new(true).open(&path) {
                            Ok(_) => Ok((path, false)),
                            Err(err)
                                if err.kind() == io::ErrorKind::AlreadyExists
                                    && fs::symlink_metadata(&path)?.is_file() =>
                            {
                                Ok((path, true))
                            }
                            Err(err) => Err(err),
                        }
                    });
                match result {
                    Ok((path, true)) => {
                        self.modal = Some(Modal::Replace(path));
                        Action::None
                    }
                    Ok((path, false)) => Action::Selected(vec![path]),
                    Err(err) => {
                        self.message = Some(err.to_string());
                        Action::None
                    }
                }
            }
            Some(PickerRequest::Open { .. }) if !self.selection.is_empty() => {
                Action::Selected(self.selection.iter().cloned().collect())
            }
            _ => Action::None,
        }
    }

    pub(crate) fn toggle(&mut self, path: PathBuf) {
        if !self.selection.remove(&path) {
            if !self.multiple() {
                self.selection.clear();
            }
            self.selection.insert(path);
        }
    }

    pub(crate) fn select_all(&mut self) {
        self.selection = self
            .entries
            .iter()
            .filter(|entry| self.request.is_none() || (!entry.directory && self.allowed(entry)))
            .map(|entry| entry.path.clone())
            .collect();
        self.select_mode = true;
    }

    pub(crate) fn selected(&self) -> Vec<PathBuf> {
        self.selection.iter().cloned().collect()
    }

    pub(crate) fn set_clipboard(&mut self, moving: bool) {
        let selected = self.selected();
        if !selected.is_empty() {
            self.message = Some(format!("{} item(s) ready to paste", selected.len()));
            self.clipboard = Some((
                if moving {
                    ClipboardMode::Move
                } else {
                    ClipboardMode::Copy
                },
                selected,
            ));
            self.selection.clear();
            self.select_mode = false;
        }
    }

    pub(crate) fn can_paste(&self) -> bool {
        self.request.is_none() && self.clipboard.is_some() && self.directory().is_some()
    }

    pub(crate) fn paste(&mut self) {
        let Some(dest) = self.directory().map(Path::to_owned) else {
            return;
        };
        let Some((mode, paths)) = self.clipboard.clone() else {
            return;
        };
        self.start_job(paths, move |path| {
            #[cfg(target_arch = "wasm32")]
            {
                crate::wasm_fs::transfer_into(path, &dest, mode == ClipboardMode::Move)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let name = path
                    .file_name()
                    .ok_or_else(|| io::Error::other("Invalid item"))?;
                if mode == ClipboardMode::Move && path.parent() == Some(dest.as_path()) {
                    return Ok(());
                }
                fs_ops::transfer(
                    path,
                    &fs_ops::unique_path(dest.join(name)),
                    mode == ClipboardMode::Move,
                )
            }
        });
    }

    pub(crate) fn restore(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            self.start_job(self.selected(), crate::wasm_fs::restore);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(root) = fs_ops::trash_root() else {
                return;
            };
            self.start_job(self.selected(), move |path| fs_ops::restore(&root, path));
        }
    }

    pub(crate) fn extract(&mut self) {
        let paths = self.selected();
        self.start_job(paths, |path| {
            #[cfg(target_arch = "wasm32")]
            {
                crate::wasm_fs::extract(path)
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let title = path
                    .file_stem()
                    .ok_or_else(|| io::Error::other("Invalid archive name"))?;
                fs_ops::extract(path, &fs_ops::unique_path(path.with_file_name(title)))
            }
        });
    }

    pub(crate) fn begin_modal(&mut self, modal: Modal) {
        self.edit = match &modal {
            Modal::Rename(path) => path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            Modal::Compress(_) => "Archive.zip".into(),
            _ => String::new(),
        };
        self.modal = Some(modal);
        self.menu_open = false;
    }

    pub(crate) fn finish_modal(&mut self) {
        let Some(modal) = self.modal.clone() else {
            return;
        };
        let name = self.edit.trim().to_owned();
        match modal {
            Modal::NewFolder => {
                let res = self
                    .directory()
                    .ok_or_else(|| io::Error::other("Choose a folder"))
                    .and_then(|dir| fs_ops::child(dir, &name))
                    .and_then(|p| {
                        #[cfg(target_arch = "wasm32")]
                        {
                            crate::wasm_fs::create_dir(&p)
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            fs::create_dir(p)
                        }
                    });
                self.complete_immediate(res);
            }
            Modal::Rename(path) => {
                let res = path
                    .parent()
                    .ok_or_else(|| io::Error::other("No parent directory"))
                    .and_then(|dir| fs_ops::child(dir, &name))
                    .and_then(|dest| {
                        #[cfg(target_arch = "wasm32")]
                        {
                            crate::wasm_fs::rename(&path, &dest)
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            fs_ops::rename(&path, &dest)
                        }
                    });
                self.complete_immediate(res);
            }
            Modal::Compress(paths) => {
                let folder = self.directory().map(Path::to_owned).or_else(|| {
                    paths
                        .first()
                        .and_then(|path| path.parent().map(Path::to_owned))
                });
                let name = if name.to_ascii_lowercase().ends_with(".zip") {
                    name
                } else {
                    format!("{name}.zip")
                };
                let res = folder
                    .ok_or_else(|| io::Error::other("Choose a folder"))
                    .and_then(|dir| fs_ops::child(&dir, &name));
                match res {
                    Ok(target) => {
                        self.modal = None;
                        self.start_job(vec![target], move |target| {
                            #[cfg(target_arch = "wasm32")]
                            {
                                crate::wasm_fs::compress(&paths, target)
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                fs_ops::compress(&paths, target)
                            }
                        });
                    }
                    Err(err) => self.message = Some(err.to_string()),
                }
            }
            Modal::Delete(paths) => {
                let permanently = self.location == Location::Trash;
                #[cfg(target_arch = "wasm32")]
                {
                    self.modal = None;
                    self.start_job(paths, move |path| {
                        if permanently {
                            crate::wasm_fs::empty_entry(path)
                        } else {
                            crate::wasm_fs::trash(path)
                        }
                    });
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let Some(root) = fs_ops::trash_root() else {
                        self.message = Some("Trash storage is unavailable".into());
                        return;
                    };
                    self.modal = None;
                    self.start_job(paths, move |path| {
                        if permanently {
                            fs_ops::empty_entry(&root, path)
                        } else {
                            fs_ops::trash(&root, path)
                        }
                    });
                }
            }
            _ => self.modal = None,
        }
    }

    fn complete_immediate(&mut self, result: io::Result<()>) {
        match result {
            Ok(()) => {
                self.modal = None;
                self.selection.clear();
                self.select_mode = false;
                self.reload();
            }
            Err(err) => self.message = Some(err.to_string()),
        }
    }

    fn start_job(
        &mut self,
        paths: Vec<PathBuf>,
        op: impl Fn(&Path) -> io::Result<()> + Send + 'static,
    ) {
        if paths.is_empty() || self.job.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.job = Some(receiver);
        self.message = Some("Working…".into());
        let run = move || {
            let mut report = JobReport {
                done: 0,
                errors: Vec::new(),
                failed: Vec::new(),
            };
            for path in paths {
                match op(&path) {
                    Ok(()) => report.done += 1,
                    Err(err) => {
                        report.errors.push(format!("{}: {err}", file_name(&path)));
                        report.failed.push(path);
                    }
                }
            }
            let _ = sender.send(report);
        };
        #[cfg(target_arch = "wasm32")]
        run();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(run);
    }

    pub(crate) fn idle(&self) -> bool {
        self.job.is_none() && self.scan.is_none() && self.thumbnails.is_none()
    }

    pub(crate) fn is_loading(&self) -> bool {
        self.scan.is_some()
    }

    pub fn reload(&mut self) {
        self.initialized = true;
        if let Some(cancel) = self.scan_cancel.take() {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.scan_cancel = Some(cancel.clone());
        let (sender, receiver) = mpsc::channel();
        self.scan = Some(receiver);
        self.entries.clear();
        self.pictures.clear();
        self.thumbnail_requests.clear();
        self.thumbnails = None;
        let location = self.location.clone();
        let home = self.home.clone();
        let query = self.search.trim().to_lowercase();
        let filter = self.filter.clone();
        let request = self.request.clone();
        let hidden = self.hidden;
        #[cfg(target_arch = "wasm32")]
        {
            let _ = cancel;
            let _ = sender.send(crate::wasm_fs::read_location(
                &location,
                &home,
                &query,
                &filter,
                request.as_ref(),
                hidden,
            ));
        }
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let _ = sender.send(read_location(
                &location,
                &home,
                &query,
                &filter,
                request.as_ref(),
                hidden,
                &cancel,
            ));
        });
    }

    pub(crate) fn poll(&mut self) {
        if !self.initialized {
            self.reload();
        }
        if let Some(rx) = &self.scan {
            match rx.try_recv() {
                Ok(result) => {
                    self.scan = None;
                    match result {
                        Ok(entries) => {
                            self.entries = entries;
                            self.sort();
                        }
                        Err(err) => self.message = Some(err.to_string()),
                    }
                }
                Err(TryRecvError::Disconnected) => self.scan = None,
                Err(TryRecvError::Empty) => {}
            }
        }
        if let Some(rx) = &self.job {
            match rx.try_recv() {
                Ok(report) => {
                    self.job = None;
                    if let Some((ClipboardMode::Move, paths)) = &mut self.clipboard {
                        #[cfg(target_arch = "wasm32")]
                        paths.retain(|path| {
                            report.failed.contains(path) || crate::wasm_fs::exists(path)
                        });
                        #[cfg(not(target_arch = "wasm32"))]
                        paths.retain(|path| report.failed.contains(path) || path.exists());
                        if paths.is_empty() {
                            self.clipboard = None;
                        }
                    }
                    self.selection.clear();
                    self.select_mode = false;
                    self.message = Some(match report.errors.first() {
                        Some(error) => format!(
                            "{} succeeded, {} failed. {error}",
                            report.done,
                            report.errors.len()
                        ),
                        None => format!("Done · {} item(s)", report.done),
                    });
                    self.reload();
                }
                Err(TryRecvError::Disconnected) => {
                    self.job = None;
                    self.message = Some("The operation stopped unexpectedly".into());
                    self.reload();
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        if let Some(rx) = &self.thumbnails {
            loop {
                match rx.try_recv() {
                    Ok((path, Some(image))) if self.pictures.len() < 72 => {
                        self.pictures.insert(path, image);
                    }
                    Ok(_) => {}
                    Err(TryRecvError::Disconnected) => {
                        self.thumbnails = None;
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                }
            }
        }
    }

    pub(crate) fn thumbnails(&mut self, paths: Vec<PathBuf>) {
        if self.thumbnails.is_some() {
            return;
        }
        let paths: Vec<_> = paths
            .into_iter()
            .filter(|path| self.thumbnail_requests.insert(path.clone()))
            .take(16)
            .collect();
        if paths.is_empty() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.thumbnails = Some(receiver);
        #[cfg(target_arch = "wasm32")]
        {
            for path in paths {
                let image = crate::wasm_fs::thumbnail(&path);
                if sender.send((path, image)).is_err() {
                    break;
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            for path in paths {
                let image = crate::read_image(&path, 224).ok().and_then(|image| {
                    bhoo::Image::rgba(image.width(), image.height(), image.into_raw())
                });
                if sender.send((path, image)).is_err() {
                    break;
                }
            }
        });
    }

    pub(crate) fn sort(&mut self) {
        let order = self.order;
        let descending = self.descending;
        self.entries.sort_by(|a, b| {
            b.directory.cmp(&a.directory).then_with(|| {
                let by_name = || a.name.to_lowercase().cmp(&b.name.to_lowercase());
                let result = match order {
                    Order::Name => by_name(),
                    Order::Size => a.bytes.cmp(&b.bytes).then_with(by_name),
                    Order::Modified => a.modified.cmp(&b.modified).then_with(by_name),
                    Order::Kind => a
                        .path
                        .extension()
                        .cmp(&b.path.extension())
                        .then_with(by_name),
                };
                if descending { result.reverse() } else { result }
            })
        });
    }
}

pub(crate) fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

fn user_directory(home: &Path, key: &str, fallback: &str) -> PathBuf {
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let wanted = format!("XDG_{key}_DIR=");
    if let Ok(config) = fs::read_to_string(config.join("user-dirs.dirs"))
        && let Some(value) = config
            .lines()
            .find_map(|line| line.trim().strip_prefix(&wanted))
    {
        let value = value.trim_matches('"');
        if let Some(relative) = value.strip_prefix("$HOME/") {
            return home.join(relative);
        }
        if Path::new(value).is_absolute() {
            return PathBuf::from(value);
        }
    }
    home.join(fallback)
}

fn discover_roots(home: &Path) -> Vec<(String, Location)> {
    #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
    let mut roots = vec![
        ("Recent".into(), Location::Recent),
        ("Images".into(), Location::Category(Filter::Images)),
        ("Videos".into(), Location::Category(Filter::Videos)),
        ("Audio".into(), Location::Category(Filter::Audio)),
        ("Documents".into(), Location::Category(Filter::Documents)),
        (
            "Downloads".into(),
            Location::Directory(user_directory(home, "DOWNLOAD", "Downloads")),
        ),
        (
            "Internal storage".into(),
            Location::Directory(home.to_owned()),
        ),
        (
            "File system".into(),
            Location::Directory(PathBuf::from("/")),
        ),
        ("Trash".into(), Location::Trash),
    ];
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut media = vec![PathBuf::from("/media"), PathBuf::from("/mnt")];
        if let Some(user) = std::env::var_os("USER") {
            media.push(PathBuf::from("/run/media").join(&user));
            media.push(PathBuf::from("/media").join(user));
        }
        for directory in media {
            if let Ok(entries) = fs::read_dir(directory) {
                for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
                    roots.push((file_name(&entry.path()), Location::Directory(entry.path())));
                }
            }
        }
    }
    roots
}

#[cfg(not(target_arch = "wasm32"))]
fn read_location(
    location: &Location,
    home: &Path,
    query: &str,
    filter: &Filter,
    picker: Option<&PickerRequest>,
    hidden: bool,
    cancel: &std::sync::atomic::AtomicBool,
) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut visited = BTreeSet::new();
    let bases = match location {
        Location::Directory(path) => {
            fs::read_dir(path)?;
            vec![path.clone()]
        }
        Location::Trash => {
            let Some(root) = fs_ops::trash_root() else {
                return Ok(entries);
            };
            vec![root.join("files")]
        }
        Location::Recent | Location::Category(_) => vec![
            user_directory(home, "DOWNLOAD", "Downloads"),
            user_directory(home, "DOCUMENTS", "Documents"),
            user_directory(home, "PICTURES", "Pictures"),
            user_directory(home, "MUSIC", "Music"),
            user_directory(home, "VIDEOS", "Videos"),
            user_directory(home, "DESKTOP", "Desktop"),
        ],
    };
    let collection = matches!(location, Location::Recent | Location::Category(_));
    let recurse = collection || !query.is_empty() && !matches!(location, Location::Trash);
    let mut pending: Vec<_> = bases.into_iter().map(|path| (path, 0)).collect();
    let mut scanned = 0;
    while let Some((current, depth)) = pending.pop() {
        if cancel.load(std::sync::atomic::Ordering::Relaxed) || scanned >= 30_000 {
            break;
        }
        if !visited.insert(current.clone()) {
            continue;
        }
        let Ok(children) = fs::read_dir(current) else {
            continue;
        };
        for child in children.flatten() {
            if cancel.load(std::sync::atomic::Ordering::Relaxed) || scanned >= 30_000 {
                break;
            }
            scanned += 1;
            let path = child.path();
            let name = file_name(&path);
            if !hidden && name.starts_with('.') {
                continue;
            }
            let Ok(metadata) = child.metadata() else {
                continue;
            };
            let directory = if metadata.file_type().is_symlink() {
                path.is_dir()
            } else {
                metadata.is_dir()
            };
            if directory && recurse && depth < 12 && !metadata.file_type().is_symlink() {
                pending.push((path.clone(), depth + 1));
            }
            if directory && collection {
                continue;
            }
            let matches_picker = match picker {
                Some(PickerRequest::Folder) => directory,
                Some(PickerRequest::Open { filter, .. }) => directory || filter.accepts(&path),
                _ => true,
            };
            let matches_category =
                !matches!(location, Location::Category(kind) if !kind.accepts(&path));
            if matches_picker
                && matches_category
                && (directory || filter.accepts(&path))
                && (query.is_empty() || name.to_lowercase().contains(query))
            {
                entries.push(Entry {
                    path,
                    name,
                    directory,
                    bytes: metadata.len(),
                    modified: metadata.modified().ok(),
                });
            }
        }
    }
    if matches!(location, Location::Recent) && query.is_empty() {
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.modified));
        entries.truncate(150);
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_picker_filters_files_and_preserves_multiple_selection() {
        let mut browser = Browser::default();
        browser.request = Some(PickerRequest::Open {
            filter: Filter::Images,
            multiple: true,
        });
        let entry = |name: &str| Entry {
            path: Path::new("/tmp").join(name),
            name: name.into(),
            directory: false,
            bytes: 0,
            modified: None,
        };
        assert_eq!(browser.pick(entry("notes.txt")), Action::None);
        assert!(browser.allowed(&entry("photo.PNG")));
        browser.toggle(entry("photo.PNG").path);
        browser.toggle(entry("other.webp").path);
        assert_eq!(
            browser.confirm_picker(),
            Action::Selected(vec![
                PathBuf::from("/tmp/other.webp"),
                PathBuf::from("/tmp/photo.PNG"),
            ])
        );
    }

    #[test]
    fn save_picker_requests_confirmation_before_returning_an_existing_file() {
        let dir =
            std::env::temp_dir().join(format!("spectre-files-save-test-{}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        let existing = dir.join("report.txt");
        fs::write(&existing, b"original").unwrap();
        let mut browser = Browser::new(dir.clone());
        browser.request = Some(PickerRequest::Save {
            suggested_name: "report.txt".into(),
        });
        browser.location = Location::Directory(dir.clone());
        browser.save_name = "report.txt".into();
        assert_eq!(browser.confirm_picker(), Action::None);
        assert!(matches!(browser.modal, Some(Modal::Replace(ref path)) if path == &existing));
        assert_eq!(fs::read(&existing).unwrap(), b"original");

        assert!(browser.back());
        browser.save_name = "new report.txt".into();
        let created = dir.join("new report.txt");
        assert_eq!(
            browser.confirm_picker(),
            Action::Selected(vec![created.clone()])
        );
        assert!(created.is_file());
        fs::remove_dir_all(dir).unwrap();
    }
}
