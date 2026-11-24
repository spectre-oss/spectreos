# Shiraz Android APK compatibility subsystem integration for SpectreOS (Wisp + Freedesktop).
{ config, lib, pkgs, ... }:
let
  cfg = config.spectre.shiraz;
  findShirazRoot = ''
    resolve_shiraz_root() {
      for cand in \
        "''${SHIRAZ_HOME:-}" \
        "${cfg.root}" \
        "/home/spectre/shiraz/codes/shiraz" \
        "/home/spectre/spectreos/codes/shiraz-core" \
        "/usr/share/shiraz"; do
        if [ -n "$cand" ] && [ -f "$cand/src/shiraz/spectre_daemon.py" ]; then
          printf '%s' "$cand"
          return 0
        fi
      done
      return 1
    }
    ROOT="$(resolve_shiraz_root)" || {
      echo "shiraz: runtime checkout not found (set SHIRAZ_HOME or deploy to ${cfg.root})" >&2
      exit 1
    }
    export SHIRAZ_HOME="$ROOT"
    export PYTHONPATH="$ROOT/src''${PYTHONPATH:+:$PYTHONPATH}"
    export SHIRAZ_FORM_FACTOR="''${SHIRAZ_FORM_FACTOR:-mobile}"
  '';

  shirazDaemon = pkgs.writeShellScriptBin "shiraz-daemon" ''
    set -eu
    ${findShirazRoot}
    exec ${pkgs.python3}/bin/python3 -m shiraz daemon "$@"
  '';

  shirazLaunch = pkgs.writeShellScriptBin "shiraz-launch" ''
    set -eu
    ${findShirazRoot}
    exec ${pkgs.python3}/bin/python3 -m shiraz launch-app "$@"
  '';

  shirazCtl = pkgs.writeShellScriptBin "shiraz-ctl" ''
    set -eu
    ${findShirazRoot}
    exec ${pkgs.python3}/bin/python3 -m shiraz ctl "$@"
  '';

  shirazInstallApk = pkgs.writeShellScriptBin "shiraz-install-apk" ''
    set -eu
    ${findShirazRoot}
    exec ${pkgs.python3}/bin/python3 -m shiraz install-app "$@"
  '';

  # Freedesktop Desktop Entry v1.5 + MIME Apps v1.0.1 association for `.apk` files.
  shirazMimePkg = pkgs.runCommand "spectreos-shiraz-freedesktop" {} ''
    mkdir -p "$out/share/applications"
    cat > "$out/share/applications/org.spectreos.ShirazInstaller.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Version=1.5
Name=Shiraz APK Installer
Comment=Install Android APK packages into SpectreOS & Wisp Launcher
Exec=shiraz-install-apk %f
Icon=application-vnd.android.package-archive
MimeType=application/vnd.android.package-archive;
Terminal=false
NoDisplay=true
Categories=System;PackageManager;
X-Purism-FormFactor=Workstation;Mobile;
EOF
    cat > "$out/share/applications/mimeapps.list" <<'EOF'
[Default Applications]
application/vnd.android.package-archive=org.spectreos.ShirazInstaller.desktop;

[Added Associations]
application/vnd.android.package-archive=org.spectreos.ShirazInstaller.desktop;
EOF
  '';
in {
  options.spectre.shiraz = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable Shiraz Android APK compatibility CLI wrappers, Freedesktop MIME handler, and warm user daemon.";
    };
    root = lib.mkOption {
      type = lib.types.str;
      default = "/opt/shiraz";
      description = "Default path to the Shiraz runtime tree on the device.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.sessionVariables = {
      SHIRAZ_FORM_FACTOR = "mobile";
    };

    environment.systemPackages = [
      pkgs.python3
      pkgs.bubblewrap
      shirazDaemon
      shirazLaunch
      shirazCtl
      shirazInstallApk
      shirazMimePkg
    ];

    # Optional systemd --user service (on-demand; Wisp's ShirazService manages start/stop
    # automatically when the user toggles "Enable Shiraz" in Notification Center or Settings).
    systemd.user.services.shiraz-daemon = {
      description = "Shiraz Android Compatibility Warm Runtime Daemon";
      partOf = [ "graphical-session.target" ];
      after = [ "graphical-session.target" "pipewire.service" ];
      serviceConfig = {
        ExecStart = "${shirazDaemon}/bin/shiraz-daemon";
        Restart = "on-failure";
        RestartSec = 3;
        Nice = -5;
        Environment = [
          "XDG_SESSION_TYPE=wayland"
          "SDL_VIDEODRIVER=wayland"
          "SHIRAZ_FORM_FACTOR=mobile"
        ];
      };
    };
  };
}
