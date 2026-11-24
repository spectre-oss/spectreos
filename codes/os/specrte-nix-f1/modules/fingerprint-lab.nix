# Diagnostic app; device.nix selects the matching transport. No login authentication.
{ config, lib, pkgs, ... }:
let
  cfg = config.spectre.fingerprintLab;
  runtime = import pkgs.path { system = pkgs.stdenv.hostPlatform.system; };
  app = pkgs.callPackage ../apps/spectre-fingerprint { inherit runtime; };
in {
  options.spectre.fingerprintLab = {
    enable = lib.mkEnableOption "Fingerprint Lab hardware diagnostic app";
    user = lib.mkOption { type = lib.types.str; default = "spectre"; description = "Local user allowed to run sensor tests."; };
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ app ];
    systemd.services.spectre-fingerprint-storage = {
      description = "Fingerprint Lab secure-storage listeners";
      wantedBy = [ "multi-user.target" ];
      unitConfig.ConditionPathExists = "/dev/teepriv0";
      serviceConfig = {
        Type = "notify";
        ExecStart = "${app}/libexec/storage-broker --state-dir /var/lib/spectre-fingerprint";
        StateDirectory = "spectre-fingerprint";
        StateDirectoryMode = "0700";
        UMask = "0077";
        Restart = "on-failure";
        TimeoutStopSec = 15;
      };
    };
    systemd.services.spectre-fingerprint = {
      description = "Poco F1 fingerprint hardware service";
      after = [ "local-fs.target" "spectre-fingerprint-storage.service" ];
      requires = [ "spectre-fingerprint-storage.service" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${app}/bin/fingerprint-service --user ${lib.escapeShellArg cfg.user}";
        RuntimeDirectory = "spectre-fingerprint";
        RuntimeDirectoryMode = "0755";
        Restart = "on-failure";
        TimeoutStopSec = 45;
      };
    };
  };
}
