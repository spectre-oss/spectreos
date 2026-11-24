//! Linux / NixOS / Fontconfig System Font Backend.
//!
//! Discovers installed system TrueType/OpenType font directories across NixOS
//! (`/etc/fonts/conf.d/00-nixos-cache.conf`, `/run/current-system/sw/share/fonts`),
//! standard FHS paths, user `~/.local/share/fonts`, and `BHOO_FONT_DIRS`.
//!
//! Resolves the primary Latin UI face (`Roboto` by default) and the Persian/Farsi
//! UI face (**Shabnam** by default — preferring `Shabnam-FD.ttf` / `Shabnam.ttf` /
//! `Shabnam-WOL.ttf`), and persists user Fontconfig rules into
//! `~/.config/fontconfig/conf.d/60-spectre-fonts.conf`.

use super::{find_in_path, run_cmd_capture};
use crate::services::fonts::{
    FontBackend, FontCatalog, FontFaceRef, FontPolicy, FontScript, FontWeight,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Concrete Linux font backend backed by NixOS/FHS font directories and Fontconfig.
pub struct LinuxFontBackend {
    search_dirs: Vec<PathBuf>,
    fc_match_bin: Option<PathBuf>,
    fc_list_bin: Option<PathBuf>,
    user_fontconfig_dir: Option<PathBuf>,
}

impl LinuxFontBackend {
    /// Discover system font roots and Fontconfig binaries on Linux/NixOS.
    pub fn discover() -> Self {
        let search_dirs = discover_linux_font_dirs();
        let fc_match_bin = find_in_path("fc-match");
        let fc_list_bin = find_in_path("fc-list");
        let user_fontconfig_dir = resolve_user_fontconfig_dir();
        Self {
            search_dirs,
            fc_match_bin,
            fc_list_bin,
            user_fontconfig_dir,
        }
    }

    /// Locate the first existing file matching any candidate filename across `self.search_dirs`.
    fn find_in_dirs(&self, names: &[&str]) -> Option<PathBuf> {
        for name in names {
            for dir in &self.search_dirs {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// Query `fc-match -f %{file}` as a secondary fallback when a family is only registered
    /// in Fontconfig's binary cache.
    fn fc_match_file(&self, pattern: &str) -> Option<PathBuf> {
        let bin = self.fc_match_bin.as_deref()?;
        let out = run_cmd_capture(bin, &["-f", "%{file}\n", pattern])?;
        let line = out.lines().next()?.trim();
        if line.is_empty() {
            return None;
        }
        let path = PathBuf::from(line);
        path.is_file().then_some(path)
    }

    fn scan_installed_families(&self) -> Vec<String> {
        let mut families = BTreeSet::new();
        if let Some(bin) = self.fc_list_bin.as_deref()
            && let Some(out) = run_cmd_capture(bin, &[":", "family"])
        {
            for line in out.lines() {
                for part in line.split(',') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        families.insert(trimmed.to_string());
                    }
                }
            }
        }
        if families.is_empty() {
            for dir in &self.search_dirs {
                let Ok(entries) = std::fs::read_dir(dir) else {
                    continue;
                };
                for entry in entries.flatten() {
                    let path = entry.path();
                    let ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    if !matches!(ext.as_str(), "ttf" | "otf" | "ttc") {
                        continue;
                    }
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let base = stem.split('-').next().unwrap_or(stem).trim();
                        if !base.is_empty() {
                            families.insert(base.to_string());
                        }
                    }
                }
            }
        }
        families.into_iter().collect()
    }
}

impl FontBackend for LinuxFontBackend {
    fn backend_label(&self) -> &str {
        if self.fc_match_bin.is_some() {
            "linux-fontconfig"
        } else {
            "linux-fs"
        }
    }

    fn discover_catalog(&self, policy: &FontPolicy) -> FontCatalog {
        let primary_path = self.resolve_primary_font(policy);
        let persian_path = self.resolve_script_font(FontScript::Persian, policy);
        let serif_path = self
            .find_in_dirs(&[
                "NotoSerif-Regular.ttf",
                "DejaVuSerif.ttf",
                "LiberationSerif-Regular.ttf",
                "DroidSerif-Regular.ttf",
            ])
            .or_else(|| self.fc_match_file(&policy.serif_family));
        let mono_path = self
            .find_in_dirs(&[
                "AdwaitaMono-Regular.ttf",
                "DroidSansMono.ttf",
                "DejaVuSansMono.ttf",
                "LiberationMono-Regular.ttf",
                "NotoSansMono-Regular.ttf",
            ])
            .or_else(|| self.fc_match_file(&policy.mono_family));

        FontCatalog {
            primary_sans: Some(FontFaceRef {
                family: policy.latin_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Latin,
                path: primary_path,
            }),
            persian_sans: Some(FontFaceRef {
                family: policy.persian_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Persian,
                path: persian_path,
            }),
            serif: Some(FontFaceRef {
                family: policy.serif_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Latin,
                path: serif_path,
            }),
            monospace: Some(FontFaceRef {
                family: policy.mono_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Latin,
                path: mono_path,
            }),
            search_dirs: self.search_dirs.clone(),
            installed_families: self.scan_installed_families(),
        }
    }

    fn resolve_primary_font(&self, policy: &FontPolicy) -> Option<PathBuf> {
        if let Some(env_path) = std::env::var_os("BHOO_FONT") {
            let p = PathBuf::from(env_path);
            if p.is_file() {
                return Some(p);
            }
        }
        self.find_in_dirs(&[
            "Roboto-Regular.ttf",
            "RobotoStatic-Regular.ttf",
            "AdwaitaSans-Regular.ttf",
            "DejaVuSans.ttf",
            "LiberationSans-Regular.ttf",
            "NotoSans-Regular.ttf",
        ])
        .or_else(|| self.fc_match_file(&policy.latin_family))
        .or_else(|| self.fc_match_file("sans-serif"))
    }

    fn resolve_script_font(&self, script: FontScript, policy: &FontPolicy) -> Option<PathBuf> {
        match script {
            FontScript::Latin => self.resolve_primary_font(policy),
            FontScript::Persian => {
                if let Some(env_path) = std::env::var_os("BHOO_PERSIAN_FONT") {
                    let p = PathBuf::from(env_path);
                    if p.is_file() {
                        return Some(p);
                    }
                }
                self.find_in_dirs(&[
                    "Shabnam-FD.ttf",
                    "Shabnam.ttf",
                    "Shabnam-WOL.ttf",
                    "Shabnam-FD-WOL.ttf",
                    "Shabnam-Medium-FD.ttf",
                    "Shabnam-Medium.ttf",
                    "Vazirmatn-Regular.ttf",
                    "NotoSansArabic-Regular.ttf",
                ])
                .or_else(|| self.fc_match_file(&format!("{}:lang=fa", policy.persian_family)))
                .or_else(|| self.fc_match_file("sans-serif:lang=fa"))
            }
            FontScript::Arabic => self
                .find_in_dirs(&[
                    "Shabnam.ttf",
                    "Shabnam-FD.ttf",
                    "NotoSansArabic-Regular.ttf",
                    "Vazirmatn-Regular.ttf",
                ])
                .or_else(|| self.fc_match_file("sans-serif:lang=ar")),
            FontScript::Symbols => self
                .find_in_dirs(&[
                    "NotoSansSymbols2-Regular.ttf",
                    "DejaVuSans.ttf",
                    "AdwaitaSans-Regular.ttf",
                ])
                .or_else(|| self.fc_match_file("Noto Sans Symbols 2")),
        }
    }

    fn apply_policy(&mut self, policy: &FontPolicy) {
        let Some(ref conf_dir) = self.user_fontconfig_dir else {
            return;
        };
        if std::fs::create_dir_all(conf_dir).is_err() {
            return;
        }
        let target = conf_dir.join("60-spectre-fonts.conf");
        let tmp = conf_dir.join("60-spectre-fonts.conf.tmp");
        let xml = policy.to_fontconfig_xml();
        if std::fs::write(&tmp, xml).is_ok() {
            let _ = std::fs::rename(&tmp, &target);
        }
    }
}

fn resolve_user_fontconfig_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        return Some(PathBuf::from(xdg).join("fontconfig/conf.d"));
    }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/fontconfig/conf.d"))
}

/// Discover font directories across `BHOO_FONT_DIRS`, Fontconfig XML configs
/// (`/etc/fonts/conf.d/00-nixos-cache.conf`, `/etc/fonts/fonts.conf`), NixOS profile,
/// user `~/.local/share/fonts`, and standard FHS font roots.
pub fn discover_linux_font_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut push_dir = |p: PathBuf| {
        if p.is_dir() && !dirs.contains(&p) {
            dirs.push(p);
        }
    };
    let push_font_root = |root: &Path, push: &mut dyn FnMut(PathBuf)| {
        push(root.to_path_buf());
        let sf = root.join("share/fonts");
        if sf.is_dir() {
            push(sf.clone());
            if let Ok(entries) = std::fs::read_dir(&sf) {
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if sub.is_dir() {
                        push(sub);
                    }
                }
            }
        }
    };

    if let Some(env_dirs) = std::env::var_os("BHOO_FONT_DIRS") {
        for part in std::env::split_paths(&env_dirs) {
            push_font_root(&part, &mut push_dir);
        }
    }

    for cfg in [
        "/etc/fonts/conf.d/00-nixos-cache.conf",
        "/etc/fonts/fonts.conf",
        "/etc/fonts/local.conf",
    ] {
        if let Ok(text) = std::fs::read_to_string(cfg) {
            let mut rest = text.as_str();
            while let Some(start) = rest.find("<dir>") {
                let after = &rest[start + 5..];
                let Some(end) = after.find("</dir>") else {
                    break;
                };
                let raw = after[..end].trim();
                if raw.starts_with('/') {
                    push_font_root(Path::new(raw), &mut push_dir);
                }
                rest = &after[end + 6..];
            }
        }
    }

    if let Some(home) = std::env::var_os("HOME") {
        let user_fonts = PathBuf::from(home).join(".local/share/fonts");
        push_font_root(&user_fonts, &mut push_dir);
    }

    for dir in [
        "/run/current-system/sw/share/fonts/shabnam-fonts",
        "/run/current-system/sw/share/fonts/truetype",
        "/run/current-system/sw/share/fonts/Adwaita",
        "/run/current-system/sw/share/fonts/noto",
        "/run/current-system/sw/share/fonts",
        "/usr/share/fonts/shabnam-fonts",
        "/usr/share/fonts/truetype/shabnam",
        "/usr/share/fonts/truetype/roboto",
        "/usr/share/fonts/roboto",
        "/usr/share/fonts/Adwaita",
        "/usr/share/fonts/truetype/dejavu",
        "/usr/share/fonts/TTF",
        "/usr/share/fonts/truetype/liberation",
        "/usr/share/fonts/liberation",
        "/usr/share/fonts/noto",
        "/usr/share/fonts/truetype/noto",
        "/system/fonts",
    ] {
        push_dir(PathBuf::from(dir));
    }

    dirs
}
