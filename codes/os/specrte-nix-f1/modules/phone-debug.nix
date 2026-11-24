# Opt-in development access. Transport is the existing authenticated USB SSH.
{ config, lib, pkgs, ... }:
let cfg = config.spectre.phoneDebug;
in {
  options.spectre.phoneDebug.enable = lib.mkEnableOption "USB SSH graphical debugging";
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ (import ../debug { inherit pkgs; }) ];
    users.groups.phone-debug = {};
    users.users.spectre.extraGroups = [ "phone-debug" ];
    services.udev.extraRules = ''
      KERNEL=="uinput", SUBSYSTEM=="misc", GROUP="phone-debug", MODE="0660"
    '';
  };
}
