//! System Font Service (platform-agnostic).
//!
//! Owns the active system typography policy (primary Latin UI font, Persian/Farsi
//! script font defaulting to **Shabnam**, Serif/Monospace families, and user font style
//! selection) and delegates concrete font discovery & Fontconfig persistence to a
//! platform [`FontBackend`] (e.g. `backends::linux::fonts::LinuxFontBackend`).

use spectre_settings::SystemState;
use std::path::PathBuf;

/// Primary script buckets managed by the system font policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontScript {
    Latin,
    Persian,
    Arabic,
    Symbols,
}

impl FontScript {
    pub const fn bcp47(self) -> &'static str {
        match self {
            Self::Latin => "en",
            Self::Persian => "fa",
            Self::Arabic => "ar",
            Self::Symbols => "und-Zsym",
        }
    }
}

/// Generic typography roles corresponding to `bhoo::FontFamily` and CSS generics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontRole {
    Sans,
    Serif,
    Monospace,
}

/// Standard OpenType weight buckets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FontWeight {
    Thin = 100,
    Light = 300,
    Regular = 400,
    Medium = 500,
    Bold = 700,
    Black = 900,
}

/// User-selectable system font style presets (matching `wallpaper::FONT_STYLES`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStylePreset {
    Default = 0,
    Serif = 1,
    Monospace = 2,
    CompactSans = 3,
}

impl FontStylePreset {
    pub const fn from_index(idx: usize) -> Self {
        match idx {
            1 => Self::Serif,
            2 => Self::Monospace,
            3 => Self::CompactSans,
            _ => Self::Default,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Serif => "Serif",
            Self::Monospace => "Monospace",
            Self::CompactSans => "Compact sans",
        }
    }

    pub const fn primary_role(self) -> FontRole {
        match self {
            Self::Serif => FontRole::Serif,
            Self::Monospace => FontRole::Monospace,
            Self::Default | Self::CompactSans => FontRole::Sans,
        }
    }
}

/// Resolved face entry discovered by a platform [`FontBackend`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFaceRef {
    pub family: String,
    pub style: String,
    pub weight: FontWeight,
    pub script: FontScript,
    pub path: Option<PathBuf>,
}

/// Active typography policy applied across Wisp, `libbhoo` apps, and desktop Fontconfig.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontPolicy {
    pub latin_family: String,
    pub persian_family: String,
    pub serif_family: String,
    pub mono_family: String,
    pub style_preset: FontStylePreset,
}

impl Default for FontPolicy {
    fn default() -> Self {
        Self {
            latin_family: "Roboto".into(),
            persian_family: "Shabnam".into(),
            serif_family: "Noto Serif".into(),
            mono_family: "Adwaita Mono".into(),
            style_preset: FontStylePreset::Default,
        }
    }
}

impl FontPolicy {
    /// CSS font-family stack for web/WASM previews & HTML widgets.
    pub fn css_font_stack(&self, role: FontRole, prefer_persian: bool) -> String {
        match role {
            FontRole::Serif => {
                if prefer_persian {
                    format!("\"{}\", \"{}\", Georgia, serif", self.persian_family, self.serif_family)
                } else {
                    format!("\"{}\", Georgia, \"{}\", serif", self.serif_family, self.persian_family)
                }
            }
            FontRole::Monospace => {
                if prefer_persian {
                    format!("\"{}\", \"{}\", monospace", self.persian_family, self.mono_family)
                } else {
                    format!("\"{}\", \"{}\", monospace", self.mono_family, self.persian_family)
                }
            }
            FontRole::Sans => {
                if prefer_persian {
                    format!("\"{}\", \"{}\", Arial, sans-serif", self.persian_family, self.latin_family)
                } else {
                    format!("\"{}\", Arial, \"{}\", sans-serif", self.latin_family, self.persian_family)
                }
            }
        }
    }

    /// Generate user Fontconfig XML enforcing the active family policy and binding
    /// Persian (`lang=fa`) strongly to **Shabnam** (or `self.persian_family`).
    pub fn to_fontconfig_xml(&self) -> String {
        let primary_sans = match self.style_preset {
            FontStylePreset::Serif => self.serif_family.as_str(),
            FontStylePreset::Monospace => self.mono_family.as_str(),
            FontStylePreset::Default | FontStylePreset::CompactSans => self.latin_family.as_str(),
        };
        let esc = |s: &str| {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        };
        let fa = esc(&self.persian_family);
        let sans = esc(primary_sans);
        let serif = esc(&self.serif_family);
        let mono = esc(&self.mono_family);
        format!(
            r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<!-- Managed by SpectreOS FontService (Wisp) -->
<fontconfig>
  <match target="pattern">
    <test name="lang" compare="contains">
      <string>fa</string>
    </test>
    <edit name="family" mode="prepend" binding="strong">
      <string>{fa}</string>
    </edit>
  </match>
  <alias>
    <family>sans-serif</family>
    <prefer>
      <family>{sans}</family>
      <family>{fa}</family>
    </prefer>
  </alias>
  <alias>
    <family>serif</family>
    <prefer>
      <family>{serif}</family>
      <family>{fa}</family>
    </prefer>
  </alias>
  <alias>
    <family>monospace</family>
    <prefer>
      <family>{mono}</family>
      <family>{fa}</family>
    </prefer>
  </alias>
</fontconfig>
"#
        )
    }
}

/// Snapshot of system fonts discovered by the active [`FontBackend`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontCatalog {
    pub primary_sans: Option<FontFaceRef>,
    pub persian_sans: Option<FontFaceRef>,
    pub serif: Option<FontFaceRef>,
    pub monospace: Option<FontFaceRef>,
    pub search_dirs: Vec<PathBuf>,
    pub installed_families: Vec<String>,
}

/// Platform-agnostic contract for discovering and applying system fonts.
pub trait FontBackend: Send + Sync {
    fn backend_label(&self) -> &str;
    fn discover_catalog(&self, policy: &FontPolicy) -> FontCatalog;
    fn resolve_primary_font(&self, policy: &FontPolicy) -> Option<PathBuf>;
    fn resolve_script_font(&self, script: FontScript, policy: &FontPolicy) -> Option<PathBuf>;
    fn apply_policy(&mut self, policy: &FontPolicy);
}

/// Safe no-op backend for desktop previews, WASM, and unit tests.
pub struct NullFontBackend;

impl FontBackend for NullFontBackend {
    fn backend_label(&self) -> &str {
        "null"
    }

    fn discover_catalog(&self, policy: &FontPolicy) -> FontCatalog {
        FontCatalog {
            primary_sans: Some(FontFaceRef {
                family: policy.latin_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Latin,
                path: None,
            }),
            persian_sans: Some(FontFaceRef {
                family: policy.persian_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Persian,
                path: None,
            }),
            serif: Some(FontFaceRef {
                family: policy.serif_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Latin,
                path: None,
            }),
            monospace: Some(FontFaceRef {
                family: policy.mono_family.clone(),
                style: "Regular".into(),
                weight: FontWeight::Regular,
                script: FontScript::Latin,
                path: None,
            }),
            search_dirs: Vec::new(),
            installed_families: vec![
                policy.latin_family.clone(),
                policy.persian_family.clone(),
                policy.serif_family.clone(),
                policy.mono_family.clone(),
            ],
        }
    }

    fn resolve_primary_font(&self, _policy: &FontPolicy) -> Option<PathBuf> {
        None
    }

    fn resolve_script_font(&self, _script: FontScript, _policy: &FontPolicy) -> Option<PathBuf> {
        None
    }

    fn apply_policy(&mut self, _policy: &FontPolicy) {}
}

/// Returns `true` when `text` contains Persian/Arabic script characters or Persian digits.
pub fn contains_persian_or_arabic(text: &str) -> bool {
    text.chars().any(|ch| {
        matches!(
            ch as u32,
            0x0600..=0x06FF
                | 0x0750..=0x077F
                | 0x08A0..=0x08FF
                | 0xFB50..=0xFDFF
                | 0xFE70..=0xFEFF
        )
    })
}

/// Domain service managing system typography discovery, Persian `Shabnam` binding,
/// and runtime font policy synchronization with `SystemState`.
pub struct FontService {
    backend: Box<dyn FontBackend>,
    policy: FontPolicy,
    catalog: FontCatalog,
}

impl FontService {
    pub fn new(mut backend: Box<dyn FontBackend>, sys: &SystemState) -> Self {
        let policy = FontPolicy {
            style_preset: FontStylePreset::from_index(sys.font_style),
            ..FontPolicy::default()
        };
        backend.apply_policy(&policy);
        let catalog = backend.discover_catalog(&policy);
        Self {
            backend,
            policy,
            catalog,
        }
    }

    pub fn backend_label(&self) -> &str {
        self.backend.backend_label()
    }

    pub fn policy(&self) -> &FontPolicy {
        &self.policy
    }

    pub fn catalog(&self) -> &FontCatalog {
        &self.catalog
    }

    /// Re-scan system fonts via the active backend.
    pub fn refresh_catalog(&mut self) {
        self.catalog = self.backend.discover_catalog(&self.policy);
    }

    /// Resolve the primary UI TrueType/OpenType font file for `libbhoo` / Wisp.
    pub fn primary_font_path(&self) -> Option<PathBuf> {
        self.catalog
            .primary_sans
            .as_ref()
            .and_then(|f| f.path.clone())
            .or_else(|| self.backend.resolve_primary_font(&self.policy))
    }

    /// Resolve the Persian/Farsi UI font file (preferring **Shabnam**).
    pub fn persian_font_path(&self) -> Option<PathBuf> {
        self.catalog
            .persian_sans
            .as_ref()
            .and_then(|f| f.path.clone())
            .or_else(|| self.backend.resolve_script_font(FontScript::Persian, &self.policy))
    }

    /// Resolve a font path for a specific script bucket.
    pub fn script_font_path(&self, script: FontScript) -> Option<PathBuf> {
        match script {
            FontScript::Latin => self.primary_font_path(),
            FontScript::Persian => self.persian_font_path(),
            _ => self.backend.resolve_script_font(script, &self.policy),
        }
    }

    /// All discovered system font directories (including NixOS `shabnam-fonts` and `truetype`).
    pub fn font_dirs(&self) -> &[PathBuf] {
        &self.catalog.search_dirs
    }

    /// Build a `BHOO_FONT_DIRS` search-path value including the discovered Persian (`Shabnam`)
    /// and primary font directories so child `libbhoo` apps (e.g. `spectre-keyboard`) always
    /// locate Shabnam and Roboto immediately.
    pub fn bhoo_font_dirs_env(&self) -> Option<std::ffi::OsString> {
        let mut dirs: Vec<PathBuf> = Vec::new();
        for path in [self.persian_font_path(), self.primary_font_path()]
            .into_iter()
            .flatten()
        {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !dirs.iter().any(|d| d == parent)
            {
                dirs.push(parent.to_path_buf());
            }
        }
        for dir in &self.catalog.search_dirs {
            if !dirs.contains(dir) {
                dirs.push(dir.clone());
            }
        }
        if dirs.is_empty() {
            return None;
        }
        std::env::join_paths(dirs).ok()
    }

    /// Update the Persian font family (defaults to `"Shabnam"`).
    pub fn set_persian_family(&mut self, family: impl Into<String>) {
        let next = family.into();
        if !next.is_empty() && next != self.policy.persian_family {
            self.policy.persian_family = next;
            self.backend.apply_policy(&self.policy);
            self.refresh_catalog();
        }
    }

    /// Sync user font style selection from `SystemState`. Returns `true` if policy changed.
    pub fn sync_from_state(&mut self, sys: &SystemState) -> bool {
        let next = FontStylePreset::from_index(sys.font_style);
        if next != self.policy.style_preset {
            self.policy.style_preset = next;
            self.backend.apply_policy(&self.policy);
            self.refresh_catalog();
            return true;
        }
        false
    }

    /// JSON summary for debug tooling and live inspector state.
    pub fn status_json(&self) -> String {
        fn esc(s: &str) -> String {
            s.replace('\\', "\\\\").replace('"', "\\\"")
        }
        let path_str = |p: Option<PathBuf>| -> String {
            match p {
                Some(path) => format!("\"{}\"", esc(&path.display().to_string())),
                None => "null".into(),
            }
        };
        format!(
            "{{\"backend\":\"{}\",\"style\":\"{}\",\"latinFamily\":\"{}\",\"persianFamily\":\"{}\",\"primaryPath\":{},\"persianPath\":{},\"dirCount\":{},\"familyCount\":{}}}",
            esc(self.backend_label()),
            esc(self.policy.style_preset.label()),
            esc(&self.policy.latin_family),
            esc(&self.policy.persian_family),
            path_str(self.primary_font_path()),
            path_str(self.persian_font_path()),
            self.catalog.search_dirs.len(),
            self.catalog.installed_families.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn default_policy_uses_shabnam_for_persian_and_emits_fontconfig() {
        let policy = FontPolicy::default();
        assert_eq!(policy.latin_family, "Roboto");
        assert_eq!(policy.persian_family, "Shabnam");
        assert!(policy.css_font_stack(FontRole::Sans, true).starts_with("\"Shabnam\""));

        let xml = policy.to_fontconfig_xml();
        assert!(xml.contains("<string>fa</string>"));
        assert!(xml.contains("<string>Shabnam</string>"));
        assert!(xml.contains("<family>Roboto</family>"));
    }

    #[test]
    fn contains_persian_detects_farsi_letters_and_digits_only() {
        assert!(contains_persian_or_arabic("فارسی"));
        assert!(contains_persian_or_arabic("Salam / سلام"));
        assert!(contains_persian_or_arabic("۱۲۳۴۵۶۷۸۹۰"));
        assert!(!contains_persian_or_arabic("Hello World 12345"));
    }

    #[test]
    fn font_service_syncs_style_changes_and_builds_bhoo_font_dirs() {
        struct MockFontBackend {
            applied: Arc<Mutex<Vec<FontPolicy>>>,
        }
        impl FontBackend for MockFontBackend {
            fn backend_label(&self) -> &str {
                "mock-fontconfig"
            }
            fn discover_catalog(&self, policy: &FontPolicy) -> FontCatalog {
                FontCatalog {
                    primary_sans: Some(FontFaceRef {
                        family: policy.latin_family.clone(),
                        style: "Regular".into(),
                        weight: FontWeight::Regular,
                        script: FontScript::Latin,
                        path: Some(PathBuf::from("/usr/share/fonts/roboto/Roboto-Regular.ttf")),
                    }),
                    persian_sans: Some(FontFaceRef {
                        family: policy.persian_family.clone(),
                        style: "Regular".into(),
                        weight: FontWeight::Regular,
                        script: FontScript::Persian,
                        path: Some(PathBuf::from(
                            "/run/current-system/sw/share/fonts/shabnam-fonts/Shabnam-FD.ttf",
                        )),
                    }),
                    serif: None,
                    monospace: None,
                    search_dirs: vec![PathBuf::from("/usr/share/fonts")],
                    installed_families: vec!["Roboto".into(), "Shabnam".into()],
                }
            }
            fn resolve_primary_font(&self, _policy: &FontPolicy) -> Option<PathBuf> {
                Some(PathBuf::from("/usr/share/fonts/roboto/Roboto-Regular.ttf"))
            }
            fn resolve_script_font(&self, script: FontScript, _policy: &FontPolicy) -> Option<PathBuf> {
                match script {
                    FontScript::Persian => Some(PathBuf::from(
                        "/run/current-system/sw/share/fonts/shabnam-fonts/Shabnam-FD.ttf",
                    )),
                    _ => None,
                }
            }
            fn apply_policy(&mut self, policy: &FontPolicy) {
                self.applied.lock().unwrap().push(policy.clone());
            }
        }

        let applied = Arc::new(Mutex::new(Vec::new()));
        let mut sys = SystemState::default();
        let mut svc = FontService::new(
            Box::new(MockFontBackend {
                applied: Arc::clone(&applied),
            }),
            &sys,
        );
        assert_eq!(applied.lock().unwrap().len(), 1);
        assert_eq!(
            svc.persian_font_path().unwrap(),
            PathBuf::from("/run/current-system/sw/share/fonts/shabnam-fonts/Shabnam-FD.ttf")
        );
        let env_dirs = svc.bhoo_font_dirs_env().unwrap().to_string_lossy().into_owned();
        assert!(env_dirs.contains("shabnam-fonts"));
        assert!(env_dirs.contains("roboto"));

        assert!(!svc.sync_from_state(&sys));
        sys.font_style = 1;
        assert!(svc.sync_from_state(&sys));
        assert_eq!(svc.policy().style_preset, FontStylePreset::Serif);
        assert_eq!(applied.lock().unwrap().len(), 2);
    }
}
