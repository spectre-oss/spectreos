{ pkgs, ... }:
{
  # Beryllium has a single boot partition, not Android A/B boot slots.
  mobile.boot.boot-control.enable = false;

  systemd.services.msm-modem-uim-selection = {
    after = [ "rmtfs.service" "pd-mapper.service" "qrtr-ns.service" ];
    serviceConfig.ExecCondition = pkgs.writeShellScript "sim-card-present" ''
      cards=$(qmicli --silent -pd qrtr://0 --uim-get-card-status) || exit 255
      printf '%s\n' "$cards" | grep -Fq "Card state: 'present'"
    '';
  };
}
