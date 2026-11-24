# Wisp Wayland compositor + SpectreOS System UI service (replaces Phoc/Phosh).
{ lib, pkgs, ... }:
let
  powerKey = pkgs.writeText "spectre-power-key.conf" ''
    [Login]
    HandlePowerKey=ignore
  '';
  wispLibPath = lib.makeLibraryPath [
    pkgs.libglvnd
    pkgs.libdrm
    pkgs.mesa
    pkgs.libinput
    pkgs.seatd
    pkgs.systemd
    pkgs.libxkbcommon
    pkgs.wayland
  ];
  wispPrestart = pkgs.writeShellScript "wisp-prestart" ''
    export PATH=/run/current-system/sw/bin:/run/wrappers/bin:/usr/bin:/bin
    systemctl stop phosh.service getty@tty1.service 2>/dev/null || true
    chgrp video \
      /sys/class/backlight/*/brightness \
      /sys/class/backlight/*/bl_power \
      /sys/class/leds/*/brightness \
      /sys/class/leds/*/trigger \
      /sys/class/devfreq/*gpu*/min_freq \
      /sys/class/devfreq/*gpu*/polling_interval 2>/dev/null || true
    chmod g+w \
      /sys/class/backlight/*/brightness \
      /sys/class/backlight/*/bl_power \
      /sys/class/leds/*/brightness \
      /sys/class/leds/*/trigger \
      /sys/class/devfreq/*gpu*/min_freq \
      /sys/class/devfreq/*gpu*/polling_interval 2>/dev/null || true
    ${pkgs.kbd}/bin/chvt 1 2>/dev/null || true
    exit 0
  '';
  wispLauncher = pkgs.writeShellScript "wisp-launcher" ''
    if [ -x /home/spectre/run-wisp.sh ]; then
      exec /home/spectre/run-wisp.sh "$@"
    fi
    export LD_LIBRARY_PATH="/run/opengl-driver/lib:${wispLibPath}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export BHOO_FONT="''${BHOO_FONT:-${pkgs.buildPackages.roboto}/share/fonts/truetype/Roboto-Regular.ttf}"
    export BHOO_FONT_DIRS="''${BHOO_FONT_DIRS:-${pkgs.buildPackages.shabnam-fonts}/share/fonts/shabnam-fonts:${pkgs.buildPackages.roboto}/share/fonts/truetype}"
    export WISP_SCALE="''${WISP_SCALE:-2.7}"
    export RUST_LOG="''${RUST_LOG:-info}"
    exec /home/spectre/wisp "$@"
  '';
in {
  mobile.beautification = {
    silentBoot = lib.mkDefault true;
    splash = lib.mkDefault true;
  };

  hardware.graphics.enable = true;
  hardware.sensor.iio.enable = true;

  programs.dconf.enable = true;
  programs.calls.enable = true;
  services.gvfs.enable = true;

  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
    wireplumber.enable = true;
  };

  xdg.portal = {
    enable = true;
    extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
    config.common.default = [ "gtk" ];
  };

  # Core Wayland apps, SpectreOS Keyboard (`spectre-keyboard`), Freedesktop notification CLI, icons, and helpers used by Wisp.
  # Wisp itself owns the session D-Bus `org.freedesktop.Notifications` v1.3 service (`/org/freedesktop/Notifications`)
  # and live-syncs preferences with standalone `spectre-settings` (`codes/apps/settings`) via `~/.config/wisp/`.
  environment.systemPackages = with pkgs; [
    epiphany
    chatty
    gnome-console
    gnome-control-center
    megapixels
    adwaita-icon-theme
    hicolor-icon-theme
    libnotify
    glib
    util-linux
    wireplumber
    kbd
  ];

  # Allow video group (user `spectre`) to control panel backlight, LEDs/torch, and interactive GPU boost.
  services.udev.extraRules = ''
    ACTION=="add", SUBSYSTEM=="backlight", RUN+="${pkgs.coreutils}/bin/chgrp video /sys/class/backlight/%k/brightness /sys/class/backlight/%k/bl_power", RUN+="${pkgs.coreutils}/bin/chmod g+w /sys/class/backlight/%k/brightness /sys/class/backlight/%k/bl_power"
    ACTION=="add", SUBSYSTEM=="leds", RUN+="${pkgs.coreutils}/bin/chgrp video /sys/class/leds/%k/brightness /sys/class/leds/%k/trigger", RUN+="${pkgs.coreutils}/bin/chmod g+w /sys/class/leds/%k/brightness /sys/class/leds/%k/trigger"
    ACTION=="add", SUBSYSTEM=="devfreq", KERNEL=="*gpu*", RUN+="${pkgs.coreutils}/bin/chgrp video /sys/class/devfreq/%k/min_freq /sys/class/devfreq/%k/polling_interval", RUN+="${pkgs.coreutils}/bin/chmod g+w /sys/class/devfreq/%k/min_freq /sys/class/devfreq/%k/polling_interval"
  '';

  # Qualcomm WCN3990 boots unconfigured until a public Bluetooth address is assigned.
  systemd.services.poco-bt-addr = {
    description = "Configure Poco F1 WCN3990 Bluetooth public address";
    after = [ "bluetooth.service" ];
    wants = [ "bluetooth.service" ];
    wantedBy = [ "multi-user.target" ];
    path = [ pkgs.bluez pkgs.coreutils pkgs.gnused ];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
    };
    script = builtins.readFile ../scripts/poco-bt-addr.sh;
  };

  # Wisp handles power key presses (short press: sleep/wake, long press: power menu).
  services.logind.settings.Login.HandlePowerKey = "ignore";

  systemd.services.wisp = {
    description = "Wisp SpectreOS Unified Wayland Compositor & System UI";
    after = [ "systemd-user-sessions.service" "dbus.service" "systemd-logind.service" ];
    wantedBy = [ "graphical.target" ];
    conflicts = [ "getty@tty1.service" "phosh.service" ];
    unitConfig = {
      # Never permanently give up restarting Wisp after repeated crashes so the
      # shell and hardware power button always recover automatically.
      StartLimitIntervalSec = 0;
    };
    serviceConfig = {
      Type = "simple";
      User = "spectre";
      PAMName = "login";
      TTYPath = "/dev/tty1";
      TTYReset = true;
      TTYVHangup = true;
      StandardInput = "tty";
      StandardOutput = "journal";
      StandardError = "journal";
      Nice = -10;
      LimitRTPRIO = 50;
      Environment = [
        "XDG_SESSION_TYPE=wayland"
        "XDG_CURRENT_DESKTOP=Wisp:SpectreOS:GNOME"
        "GSK_RENDERER=gl"
        "NO_AT_BRIDGE=1"
        "MESA_DISK_CACHE_SINGLE_FILE=1"
      ];
      ExecStartPre = [ "+${wispPrestart}" ];
      ExecStart = "${pkgs.systemd}/bin/systemd-inhibit --what=sleep:idle:handle-power-key --who=wisp --why=\"Wisp manages display sleep and power key\" ${wispLauncher}";
      Restart = "always";
      RestartSec = 1;
      TimeoutStopSec = 5;
    };
  };

  mobile.boot.stage-1.contents = [
    { object = powerKey; symlink = "/spectre-power-key.conf"; }
  ];
  mobile.boot.stage-1.tasks = [ (pkgs.writeText "wisp-power-key.rb" ''
    class Tasks::WispPowerKey < SingletonTask
      def initialize()
        add_dependency(:Mount, "/run")
        Targets[:SwitchRoot].add_dependency(:Task, self)
      end
      def run()
        FileUtils.mkdir_p("/run/systemd/logind.conf.d")
        System.run("cp", "/spectre-power-key.conf", "/run/systemd/logind.conf.d/60-spectre-power-key.conf")
      end
    end
  '') ];
}
