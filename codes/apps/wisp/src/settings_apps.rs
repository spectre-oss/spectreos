//! Settings uses one catalog per desktop registry generation.
use super::{AppId, InstalledAppInfo, freedesktop::DesktopRegistry, settings::ShirazInstalledApp};
use bhoo::Image;
use std::{collections::HashSet, sync::mpsc};

pub(super) struct SettingsApps {
    pub apps: Vec<InstalledAppInfo>,
    pub desktop_indices: Vec<usize>,
    pub icons: Vec<Option<Image>>,
    versions: Vec<(String, String)>,
    pending: Option<mpsc::Receiver<Vec<Option<Image>>>>,
}

impl SettingsApps {
    pub fn matches(&self, installed: &[ShirazInstalledApp]) -> bool {
        self.versions.len() == installed.len()
            && self
                .versions
                .iter()
                .zip(installed)
                .all(|((pkg, ver), app)| pkg == &app.package && ver == &app.version)
    }

    pub fn new(registry: &DesktopRegistry, installed: &[ShirazInstalledApp]) -> Self {
        let mut apps: Vec<_> = AppId::ALL
            .iter()
            .enumerate()
            .map(|(i, app)| InstalledAppInfo {
                id: app.package().into(),
                name: app.name().into(),
                package: app.package().into(),
                version: app.version().into(),
                storage_mb: app.storage_mb(),
                builtin_idx: Some(i),
                desktop_id: None,
                is_android: false,
            })
            .collect();
        let builtin: HashSet<_> = AppId::ALL
            .iter()
            .filter_map(|&app| {
                registry
                    .find_for_builtin(app)
                    .map(|entry| entry.id.as_str())
            })
            .collect();
        let mut desktop_indices = Vec::new();
        let mut names = Vec::new();
        for (i, entry) in registry.apps.iter().enumerate() {
            let android = entry.shiraz_package.is_some();
            if !android && builtin.contains(entry.id.as_str()) {
                continue;
            }
            let package = entry.shiraz_package.as_ref().unwrap_or(&entry.id);
            let version = installed
                .iter()
                .find(|app| &app.package == package)
                .map(|app| app.version.as_str())
                .unwrap_or(if android { "Android" } else { "Desktop" });
            apps.push(InstalledAppInfo {
                id: entry.id.clone(),
                name: entry.name.clone(),
                package: package.clone(),
                version: version.into(),
                storage_mb: 0,
                builtin_idx: None,
                desktop_id: Some(entry.id.clone()),
                is_android: android,
            });
            desktop_indices.push(i);
            names.push(entry.icon.clone());
        }
        let mut pending = None;
        if !cfg!(target_arch = "wasm32") && names.iter().any(Option::is_some) {
            let mut resolver = registry.icon_resolver.clone();
            let (tx, rx) = mpsc::channel();
            if std::thread::Builder::new()
                .name("settings-icons".into())
                .spawn(move || {
                    let images = names
                        .iter()
                        .map(|name| {
                            name.as_deref()
                                .and_then(|name| resolver.load_icon_image(name))
                        })
                        .collect();
                    let _ = tx.send(images);
                })
                .is_ok()
            {
                pending = Some(rx);
            }
        }
        Self {
            apps,
            desktop_indices,
            icons: Vec::new(),
            pending,
            versions: installed
                .iter()
                .map(|a| (a.package.clone(), a.version.clone()))
                .collect(),
        }
    }

    pub fn poll(&mut self) -> bool {
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(images) => self.icons = images,
                Err(mpsc::TryRecvError::Empty) => return true,
                Err(mpsc::TryRecvError::Disconnected) => {}
            }
            self.pending = None;
        }
        false
    }
}
