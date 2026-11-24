# Sensor response validated in RAM; full desktop/suspend integration remains untested.
{ config, lib, pkgs, ... }:
let
  cfg = config.spectre.sensors;
  data = "${config.mobile.device.firmware}/share/qcom/sdm845/Xiaomi/beryllium";
in {
  options.spectre.sensors.enable = lib.mkEnableOption "Poco F1 SLPI proximity and light sensors";
  config = lib.mkIf cfg.enable {
    hardware.sensor.iio.enable = true;
    environment.systemPackages = [ pkgs.libssc ];
    systemd.services.poco-sensors = {
      description = "Poco F1 sensor processor and FastRPC listener";
      wantedBy = [ "multi-user.target" ];
      after = [ "local-fs.target" "pd-mapper.service" ];
      wants = [ "pd-mapper.service" ];
      before = [ "iio-sensor-proxy.service" ];
      path = [ pkgs.coreutils pkgs.qrtr pkgs.gawk ];
      preStart = ''
        cp -r ${data}/. /run/poco-sensors/
        chmod -R u+w /run/poco-sensors
        # These files match stock; sns_reg_config caches their vendor mtime.
        TZ=UTC touch -t 200812311830.00 /run/poco-sensors/sensors/config/*
        found=0
        for r in /sys/class/remoteproc/remoteproc*; do
          if [ "$(cat "$r/name")" = slpi ]; then
            found=1
            echo disabled > "$r/recovery"
            case "$(cat "$r/state")" in
              offline) echo start > "$r/state" ;;
              running) ;;
              *) echo 'SLPI crashed; reboot required, refusing automatic recovery' >&2; exit 1 ;;
            esac
          fi
        done
        [ "$found" = 1 ]
        for n in $(seq 1 50); do
          [ ! -e /dev/fastrpc-sdsp ] || exit 0
          sleep .1
        done
        echo 'FastRPC sensor device did not appear' >&2
        exit 1
      '';
      # Ordering against iio-sensor-proxy must include SSC readiness, not just
      # spawning the file listener. SSC is QRTR service 400 on this firmware.
      postStart = ''
        for n in $(seq 1 100); do
          if qrtr-lookup | awk '$1 == 400 { found = 1 } END { exit !found }'; then
            exit 0
          fi
          sleep .2
        done
        echo 'SSC service did not become ready' >&2
        exit 1
      '';
      serviceConfig = {
        RuntimeDirectory = "poco-sensors";
        ExecStart = "${pkgs.hexagonrpc}/bin/hexagonrpcd -s -f /dev/fastrpc-sdsp -d sdsp -R /run/poco-sensors";
        Restart = "on-failure";
        RestartSec = 3;
        TimeoutStopSec = 5;
      };
      startLimitIntervalSec = 60;
      startLimitBurst = 3;
    };
  };
}
