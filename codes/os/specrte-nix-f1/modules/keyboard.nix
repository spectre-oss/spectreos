# SpectreOS default on-screen keyboard (`spectre-keyboard`, built in Rust + libbhoo under
# `codes/apps/keyboard/`). Replaces legacy Phosh/Stevia OSK services with SpectreOS naming:
# - Binary: `spectre-keyboard` (or `/home/spectre/spectre-keyboard`)
# - Systemd user service: `org.spectreos.OSK.service`
# - Freedesktop D-Bus service: `org.spectreos.OSK0` (`/org/spectreos/OSK0`)
{ lib, pkgs, ... }:
let
  keyboardLibPath = lib.makeLibraryPath [
    pkgs.libglvnd
    pkgs.mesa
    pkgs.systemd
    pkgs.wayland
  ];
  spectreKeyboardLauncher = pkgs.writeShellScriptBin "spectre-keyboard" ''
    export LD_LIBRARY_PATH="/run/opengl-driver/lib:${keyboardLibPath}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export BHOO_FONT="''${BHOO_FONT:-${pkgs.buildPackages.roboto}/share/fonts/truetype/Roboto-Regular.ttf}"
    export BHOO_FONT_DIRS="''${BHOO_FONT_DIRS:-${pkgs.buildPackages.shabnam-fonts}/share/fonts/shabnam-fonts:${pkgs.buildPackages.roboto}/share/fonts/truetype}"
    if [ -x /home/spectre/spectre-keyboard ]; then
      exec /home/spectre/spectre-keyboard "$@"
    fi
    echo "spectre-keyboard binary not found at /home/spectre/spectre-keyboard" >&2
    exit 127
  '';
  spectreKeyboardDbus = pkgs.writeTextDir "share/dbus-1/services/org.spectreos.OSK0.service" ''
    [D-BUS Service]
    Name=org.spectreos.OSK0
    Exec=${spectreKeyboardLauncher}/bin/spectre-keyboard
    SystemdService=org.spectreos.OSK.service
  '';
in {
  fonts.packages = [
    pkgs.buildPackages.roboto
    pkgs.buildPackages.shabnam-fonts
  ];
  programs.dconf.enable = true;

  environment.systemPackages = [
    spectreKeyboardLauncher
    spectreKeyboardDbus
  ];

  systemd.user.services."org.spectreos.OSK" = {
    description = "SpectreOS Default On-Screen Keyboard (spectre-keyboard)";
    partOf = [ "graphical-session.target" ];
    after = [ "graphical-session.target" ];
    serviceConfig = {
      Type = "simple";
      ExecStart = "${spectreKeyboardLauncher}/bin/spectre-keyboard";
      Restart = "on-failure";
      RestartSec = 1;
    };
  };

  programs.dconf.profiles.user.databases = [{
    settings."org/gnome/desktop/input-sources".sources = [
      (lib.gvariant.mkTuple [ "xkb" "us" ])
      (lib.gvariant.mkTuple [ "xkb" "ir" ])
    ];
  }];
}
