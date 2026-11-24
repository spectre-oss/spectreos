{ ... }:
{
  # Stop network clients before rmtfs tears down the modem. systemd reverses
  # startup ordering at shutdown; keep QRTR helpers alive through that teardown.
  systemd.services.rmtfs = {
    before = [ "NetworkManager.service" "wpa_supplicant.service" "ModemManager.service" ];
    after = [ "pd-mapper.service" "tqftpserv.service" ];
  };
}
