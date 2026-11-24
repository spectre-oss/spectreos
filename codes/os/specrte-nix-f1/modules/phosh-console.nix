# Phosh uses tty1, but the mobile boot console can leave tty2 active.
{ pkgs, ... }:
let
  dropin = pkgs.writeText "phosh-active-vt.conf" ''
    [Service]
    ExecStartPre=+/run/current-system/sw/bin/chvt 1
  '';
  powerKey = pkgs.writeText "spectre-power-key.conf" ''
    [Login]
    HandlePowerKey=ignore
  '';
in {
  # GLib resolves desktop-entry commands against the shell's PATH while
  # building the app list. The default user unit's PATH omits system apps.
  systemd.user.services."mobi.phosh.Shell" = {
    overrideStrategy = "asDropin";
    serviceConfig.Environment = [ "PATH=/run/current-system/sw/bin:/run/wrappers/bin" ];
  };

  # Phosh handles short presses once it holds the logind inhibitor. Before
  # then, logind's desktop default would power the phone off during startup.
  services.logind.settings.Login.HandlePowerKey = "ignore";
  # Full system images activate the VT before creating the logind session.
  systemd.services.phosh.serviceConfig.ExecStartPre = [ "+${pkgs.kbd}/bin/chvt 1" ];

  # Boot-only updates also repair the already installed root filesystem.
  mobile.boot.stage-1.contents = [
    { object = dropin; symlink = "/phosh-active-vt.conf"; }
    { object = powerKey; symlink = "/spectre-power-key.conf"; }
  ];
  mobile.boot.stage-1.tasks = [ (pkgs.writeText "phosh-active-vt.rb" ''
    class Tasks::PhoshActiveVT < SingletonTask
      def initialize()
        add_dependency(:Mount, "/run")
        Targets[:SwitchRoot].add_dependency(:Task, self)
      end
      def run()
        FileUtils.mkdir_p("/run/systemd/system/phosh.service.d")
        System.run("cp", "/phosh-active-vt.conf", "/run/systemd/system/phosh.service.d/10-spectre-vt.conf")
        FileUtils.mkdir_p("/run/systemd/logind.conf.d")
        System.run("cp", "/spectre-power-key.conf", "/run/systemd/logind.conf.d/60-spectre-power-key.conf")
      end
    end
  '') ];
}
