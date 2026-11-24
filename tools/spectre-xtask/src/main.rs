//! Unified SpectreOS app Cargo task runner (`tools/spectre-xtask`).
//!
//! Invoked via `.cargo/config.toml` aliases both inside any app directory
//! (`cd codes/apps/files && cargo test-web`, `cargo release`, `cargo app-clean`, …)
//! and from the repository root (`cargo test-web files`, `cargo release all`, …).
//!
//! Zero external dependencies so it compiles in ~0.25s and works completely offline.

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Clone, Copy, Debug)]
struct AppSpec {
    name: &'static str,
    dir: &'static str,
    web_slug: &'static str,
    dev_args: &'static [&'static str],
    release_args: &'static [&'static str],
    check_args: &'static [&'static str],
    test_args: &'static [&'static str],
    run_args: Option<&'static [&'static str]>,
}

const APPS: &[AppSpec] = &[
    AppSpec {
        name: "files",
        dir: "codes/apps/files",
        web_slug: "files",
        dev_args: &["--features", "wayland"],
        release_args: &["--release", "--features", "wayland"],
        check_args: &["--all-targets", "--features", "wayland"],
        test_args: &["--lib"],
        run_args: Some(&["--features", "wayland"]),
    },
    AppSpec {
        name: "settings",
        dir: "codes/apps/settings",
        web_slug: "settings",
        dev_args: &["--features", "wayland"],
        release_args: &["--release", "--features", "wayland"],
        check_args: &["--all-targets", "--features", "wayland"],
        test_args: &["--lib"],
        run_args: Some(&["--features", "wayland"]),
    },
    AppSpec {
        name: "calendar",
        dir: "codes/apps/calendar",
        web_slug: "calendar",
        dev_args: &["--lib"],
        release_args: &["--release", "--lib"],
        check_args: &["--all-targets"],
        test_args: &["--lib"],
        run_args: None,
    },
    AppSpec {
        name: "keyboard",
        dir: "codes/apps/keyboard",
        web_slug: "keyboard",
        dev_args: &["--features", "osk"],
        release_args: &["--release", "--features", "osk"],
        check_args: &["--all-targets"],
        test_args: &["--lib"],
        run_args: Some(&["--features", "preview", "--", "--preview"]),
    },
    AppSpec {
        name: "wisp",
        dir: "codes/apps/wisp",
        web_slug: "notifications",
        dev_args: &["--features", "wayland"],
        release_args: &["--release", "--features", "compositor"],
        check_args: &["--lib"],
        test_args: &["--lib"],
        run_args: Some(&["--features", "wayland", "--", "--window"]),
    },
];

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("tools/spectre-xtask must live under <repo>/tools/spectre-xtask")
        .to_path_buf()
}

fn find_app(raw: &str) -> Option<&'static AppSpec> {
    let key = raw.trim().to_ascii_lowercase();
    match key.as_str() {
        "notifications" | "notif" => APPS.iter().find(|a| a.name == "wisp"),
        other => APPS.iter().find(|a| a.name == other),
    }
}

fn detect_app_from_cwd(root: &Path) -> Option<&'static AppSpec> {
    let cwd = env::current_dir().ok()?;
    let rel = cwd.strip_prefix(root).ok()?;
    for app in APPS {
        if rel.starts_with(app.dir) {
            return Some(app);
        }
    }
    None
}

fn print_help() {
    println!(
        r#"SpectreOS Cargo App Commands

Works inside any app folder (`cd codes/apps/<app>`) or from the repo root (`[app|all]`).

Native App Commands:
  cargo dev [app|all]            Build app(s) in dev mode with native features
  cargo app-build [app|all]      Alias for `cargo dev`
  cargo release [app|all]        Build optimized production/phone release binary (zero web-debug code)
  cargo app-release [app|all]    Alias for `cargo release`
  cargo app-check [app|all]      Fast `cargo check` for app library & targets
  cargo app-test [app|all]       Run native Rust unit tests
  cargo app-run [app]            Launch native desktop preview window (Wayland)
  cargo app-clean [app|all]      Clean native Cargo target and debug WASM artifacts
  cargo clean-all [app|all]      Alias for `cargo app-clean`

Debug-Only Browser WASM Workbench:
  cargo test-web [app|all]       Build debug WASM for this app (or all) & serve browser debugger
  cargo test-web-watch [app|all] Same as `test-web` with live Rust auto-rebuild and a Reload WASM button
  cargo test-web-check [app|all] Headless debug WASM build + canvas-painter + test/web-tests.json check

Apps: files, settings, calendar, keyboard, wisp (notifications), all
Extra flags (e.g. --port 8770, --offline, --release-wasm, --target-dir PATH) are forwarded."#
    );
}

fn run_status(cmd: &mut Command) -> Result<(), i32> {
    let status = cmd.status().map_err(|e| {
        eprintln!("spectre-xtask: failed to spawn {:?}: {e}", cmd.get_program());
        1
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(status.code().unwrap_or(1))
    }
}

fn resolve_target_dir_override() -> Option<String> {
    env::var("SPECTRE_TARGET_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| env::var("CARGO_TARGET_DIR").ok().filter(|s| !s.is_empty()))
        .or_else(|| Some("/tmp/spectre-apps-target".into()))
}

fn run_cargo_for_app(
    root: &Path,
    subcmd: &str,
    app: &AppSpec,
    base_args: &[&str],
    extra: &[String],
) -> Result<(), i32> {
    let manifest = root.join(app.dir).join("Cargo.toml");
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    println!("[spectre] {subcmd} {} ({})", app.name, app.dir);
    let mut cmd = Command::new(cargo);
    cmd.current_dir(root)
        .arg(subcmd)
        .arg("--manifest-path")
        .arg(&manifest);
    if let Some(td) = resolve_target_dir_override()
        && !extra.iter().any(|a| a == "--target-dir" || a.starts_with("--target-dir="))
    {
        cmd.arg("--target-dir").arg(td);
    }
    cmd.args(base_args);
    cmd.args(extra);
    run_status(&mut cmd)
}

fn run_web_debug(
    root: &Path,
    targets: &[&'static AppSpec],
    is_all: bool,
    watch: bool,
    check: bool,
    extra: &[String],
) -> Result<(), i32> {
    let script = root.join("tools/debug-bhoo-app.py");
    let arg_apps = if is_all {
        "all".to_string()
    } else {
        targets
            .iter()
            .map(|a| a.web_slug)
            .collect::<Vec<_>>()
            .join(",")
    };
    let mut cmd = Command::new("python3");
    cmd.current_dir(root).arg(&script).arg(&arg_apps);
    if watch && !extra.iter().any(|a| a == "--watch") {
        cmd.arg("--watch");
    }
    if check && !extra.iter().any(|a| a == "--check") {
        cmd.arg("--check");
    }
    cmd.args(extra);
    run_status(&mut cmd)
}

fn main() -> ExitCode {
    let root = repo_root();
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.is_empty() || matches!(raw_args[0].as_str(), "help" | "-h" | "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let action = raw_args[0].as_str();
    let mut pinned_app: Option<String> = None;
    let mut explicit_app: Option<String> = None;
    let mut extra: Vec<String> = Vec::new();

    let mut i = 1;
    while i < raw_args.len() {
        let arg = &raw_args[i];
        if arg == "--app" && i + 1 < raw_args.len() {
            pinned_app = Some(raw_args[i + 1].clone());
            i += 2;
            continue;
        }
        if let Some(val) = arg.strip_prefix("--app=") {
            pinned_app = Some(val.to_string());
            i += 1;
            continue;
        }
        if explicit_app.is_none()
            && !arg.starts_with('-')
            && (arg.eq_ignore_ascii_case("all")
                || arg.contains(',')
                || find_app(arg).is_some())
        {
            explicit_app = Some(arg.clone());
            i += 1;
            continue;
        }
        extra.push(arg.clone());
        i += 1;
    }

    // Precedence: explicit CLI target > `--app <name>` (per-app alias) > CWD auto-detect > `all`.
    let target_spec = explicit_app
        .or(pinned_app)
        .or_else(|| detect_app_from_cwd(&root).map(|a| a.name.to_string()))
        .unwrap_or_else(|| "all".into());

    let (targets, is_all): (Vec<&'static AppSpec>, bool) = if target_spec.eq_ignore_ascii_case("all") {
        (APPS.iter().collect(), true)
    } else {
        let mut selected = Vec::new();
        for part in target_spec.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            match find_app(trimmed) {
                Some(spec) => {
                    if !selected.iter().any(|s: &&AppSpec| s.name == spec.name) {
                        selected.push(spec);
                    }
                }
                None => {
                    eprintln!(
                        "spectre-xtask: unknown app '{trimmed}'. Supported: files, settings, calendar, keyboard, wisp, notifications, all"
                    );
                    return ExitCode::from(2);
                }
            }
        }
        (selected, false)
    };

    let result: Result<(), i32> = match action {
        "build" | "dev" => targets
            .iter()
            .try_for_each(|app| run_cargo_for_app(&root, "build", app, app.dev_args, &extra)),
        "release" => targets
            .iter()
            .try_for_each(|app| run_cargo_for_app(&root, "build", app, app.release_args, &extra)),
        "check" => targets
            .iter()
            .try_for_each(|app| run_cargo_for_app(&root, "check", app, app.check_args, &extra)),
        "test" => targets
            .iter()
            .try_for_each(|app| run_cargo_for_app(&root, "test", app, app.test_args, &extra)),
        "run" => {
            if targets.len() != 1 {
                eprintln!("spectre-xtask: `cargo app-run` expects a single app (e.g. `cargo app-run files`)");
                Err(2)
            } else {
                let app = targets[0];
                match app.run_args {
                    Some(args) => run_cargo_for_app(&root, "run", app, args, &extra),
                    None => {
                        eprintln!(
                            "[spectre] '{}' is embedded inside Wisp (no standalone desktop binary). Run `cargo test-web {}` to debug it in the browser.",
                            app.name, app.web_slug
                        );
                        Ok(())
                    }
                }
            }
        }
        "clean" => {
            let mut r = targets
                .iter()
                .try_for_each(|app| run_cargo_for_app(&root, "clean", app, &[], &extra));
            if r.is_ok() && is_all {
                let debug_target = env::var("BHOO_DEBUG_TARGET_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/tmp/bhoo-web-debug-target"));
                if debug_target.is_dir() {
                    println!("[spectre] cleaning debug WASM target {}", debug_target.display());
                    let _ = std::fs::remove_dir_all(&debug_target);
                }
            } else if r.is_ok() {
                // Clean only the bhoo-web-debug crate artifact in the shared debug target when cleaning a single app.
                let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
                let debug_manifest = root.join("tools/bhoo-web-debug/Cargo.toml");
                let debug_target = env::var("BHOO_DEBUG_TARGET_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/tmp/bhoo-web-debug-target"));
                if debug_target.is_dir() {
                    println!("[spectre] cleaning bhoo-web-debug artifacts for {}", targets[0].name);
                    let mut cmd = Command::new(cargo);
                    cmd.current_dir(&root)
                        .args(["clean", "-p", "bhoo-web-debug", "--manifest-path"])
                        .arg(&debug_manifest)
                        .arg("--target-dir")
                        .arg(&debug_target);
                    r = run_status(&mut cmd);
                }
            }
            r
        }
        "test-web" => run_web_debug(&root, &targets, is_all, false, false, &extra),
        "test-web-watch" => run_web_debug(&root, &targets, is_all, true, false, &extra),
        "test-web-check" => run_web_debug(&root, &targets, is_all, false, true, &extra),
        other => {
            eprintln!("spectre-xtask: unknown command '{other}'. Run `cargo app-help` for usage.");
            Err(2)
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code.clamp(1, 255) as u8),
    }
}
