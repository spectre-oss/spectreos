# A boot-only diagnostic bridge for existing images, plus normal SSH for new ones.
{ config, lib, pkgs, ... }:
let
  address = "172.16.42.1";
  payload = pkgs.runCommand "spectre-usb-ssh-payload" {} ''
    mkdir -p $out
    cp ${pkgs.pkgsStatic.dropbear}/bin/dropbear $out/dropbear
    cp ${pkgs.pkgsStatic.dropbear}/bin/dropbearkey $out/dropbearkey
    cp ${./usb-ssh-start.sh} $out/start.sh
    substituteInPlace $out/start.sh \
      --replace-fail @iptables@ ${pkgs.iptables}/bin/iptables
  '';
  connection = pkgs.writeText "spectre-usb.nmconnection" ''
    [connection]
    id=spectre-usb
    uuid=5d41b2b5-86cb-4e8c-bb99-b7dff2871721
    type=ethernet
    interface-name=usb0
    autoconnect=true
    autoconnect-priority=100
    [ipv4]
    method=manual
    address1=${address}/24
    never-default=true
    [ipv6]
    method=disabled
  '';
  service = pkgs.writeText "spectre-usb-ssh.service" ''
    [Unit]
    Description=Spectre diagnostic SSH over USB
    Wants=NetworkManager.service
    After=NetworkManager.service firewall.service systemd-user-sessions.service
    ConditionPathExists=!/etc/ssh/sshd_config
    [Service]
    Type=simple
    ExecStart=/run/current-system/sw/bin/bash /run/spectre-usb-ssh/start.sh
    StandardOutput=journal+console
    StandardError=journal+console
    Restart=on-failure
    RestartSec=3
    UMask=0077
    [Install]
    WantedBy=multi-user.target
  '';
  usbUplinkScript = pkgs.writeShellScript "poco-usb-uplink" ''
    export PATH=/run/current-system/sw/bin:/run/wrappers/bin:/usr/bin:/bin
    # 1. Keep SDM845 dwc3 USB controller active so usb0 does not drop into runtime suspend
    for p in /sys/bus/platform/devices/a600000.usb/power/control /sys/bus/platform/devices/a6f8800.usb/power/control; do
      [ -w "$p" ] && echo on > "$p" 2>/dev/null || true
    done
    # 2. Self-heal usb0 link & IP after cable replug or NetworkManager deactivation
    if ip link show dev usb0 >/dev/null 2>&1; then
      if ! ip -4 addr show dev usb0 2>/dev/null | grep -q '172\.16\.42\.1/'; then
        nmcli -w 2 con up spectre-usb 2>/dev/null || {
          ip link set dev usb0 up 2>/dev/null || true
          ip addr replace 172.16.42.1/24 dev usb0 2>/dev/null || true
        }
      fi
      if ip -4 addr show dev usb0 2>/dev/null | grep -q '172\.16\.42\.1/'; then
        if ! ip -4 route show default | grep -q 'via 172\.16\.42\.2 dev usb0'; then
          ip route add default via 172.16.42.2 dev usb0 metric 600 2>/dev/null || true
        fi
      fi
    fi
    # 3. Ensure fallback DNS when using USB uplink
    if ! grep -qE '^[[:space:]]*nameserver[[:space:]]+[0-9]' /etc/resolv.conf 2>/dev/null; then
      rm -f /etc/resolv.conf 2>/dev/null || true
      printf "nameserver 1.1.1.1\nnameserver 8.8.8.8\n" > /etc/resolv.conf 2>/dev/null || true
    fi
    # 4. Keep SSH port 22 open on usb0 and tailscale0 across firewall reloads
    if command -v iptables >/dev/null 2>&1; then
      for iface in tailscale0 usb0; do
        if iptables -L nixos-fw -n >/dev/null 2>&1; then
          iptables -C nixos-fw -i "$iface" -p tcp --dport 22 -j ACCEPT 2>/dev/null || \
            iptables -I nixos-fw 1 -i "$iface" -p tcp --dport 22 -j ACCEPT 2>/dev/null || true
        fi
        iptables -C INPUT -i "$iface" -p tcp --dport 22 -j ACCEPT 2>/dev/null || \
          iptables -I INPUT 1 -i "$iface" -p tcp --dport 22 -j ACCEPT 2>/dev/null || true
      done
    fi
    exit 0
  '';
  tailscaleWrapper = pkgs.writeShellScriptBin "tailscale" ''
    exec /home/spectre/tailscale/tailscale --socket=/run/tailscale/tailscaled.sock "$@"
  '';
in {
  # The gadget survives switch_root. No unauthenticated stage-1 SSH is enabled.
  mobile.boot.stage-1.usb.features = [ "rndis" ];
  mobile.boot.stage-1.contents = [
    { object = payload; symlink = "/usb-ssh-payload"; }
    { object = connection; symlink = "/usb-ssh-connection"; }
    { object = service; symlink = "/usb-ssh-service"; }
  ];
  mobile.boot.stage-1.tasks = [ (pkgs.writeText "usb-ssh-runtime.rb" ''
    class Tasks::USBSSHRuntime < SingletonTask
      def initialize()
        add_dependency(:Mount, "/run")
        Targets[:SwitchRoot].add_dependency(:Task, self)
      end
      def run()
        System.run("cp", "-rL", "/usb-ssh-payload", "/run/spectre-usb-ssh")
        FileUtils.mkdir_p("/run/NetworkManager/system-connections")
        System.run("cp", "/usb-ssh-connection", "/run/NetworkManager/system-connections/spectre-usb.nmconnection")
        System.run("chmod", "600", "/run/NetworkManager/system-connections/spectre-usb.nmconnection")
        FileUtils.mkdir_p("/run/systemd/system/multi-user.target.wants")
        System.run("cp", "/usb-ssh-service", "/run/systemd/system/spectre-usb-ssh.service")
        System.run("ln", "-s", "../spectre-usb-ssh.service", "/run/systemd/system/multi-user.target.wants/spectre-usb-ssh.service")
      end
    end
  '') ];

  # Future complete system images use NixOS OpenSSH instead of the recovery bridge.
  services.openssh = {
    enable = true;
    openFirewall = false;
    # Wi-Fi uses DHCP, so bind IPv4 generally and restrict access by interface.
    listenAddresses = [{ addr = "0.0.0.0"; port = 22; }];
    settings = {
      PasswordAuthentication = true;
      KbdInteractiveAuthentication = false;
      PermitRootLogin = "no";
      AllowUsers = [ "spectre" ];
    };
  };
  # The USB gadget is marked unmanaged by a udev rule. Explicitly allow
  # NetworkManager to activate the static USB profile on the installed system.
  networking.networkmanager.settings."device-spectre-usb" = {
    match-device = "interface-name:usb0";
    managed = 1;
  };
  networking.firewall.interfaces.usb0.allowedTCPPorts = [ 22 ];
  networking.firewall.interfaces.wlan0.allowedTCPPorts = [ 22 ];
  networking.firewall.interfaces.tailscale0.allowedTCPPorts = [ 22 ];
  networking.firewall.trustedInterfaces = [ "tailscale0" ];

  # Persistent USB uplink fallback route (metric 600 so Wi-Fi always takes priority)
  # and Tailscale SSH firewall continuity when using USB internet sharing.
  systemd.services.poco-usb-uplink = {
    description = "Poco F1 USB uplink route, DNS fallback, and Tailscale SSH firewall rule";
    after = [ "NetworkManager.service" "firewall.service" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${usbUplinkScript}";
    };
  };
  systemd.timers.poco-usb-uplink = {
    description = "Periodically maintain Poco F1 USB uplink route and Tailscale SSH firewall rule";
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnBootSec = "5s";
      OnUnitActiveSec = "15s";
      AccuracySec = "2s";
    };
  };

  # Auto-start static Tailscale daemon when installed under /home/spectre/tailscale.
  systemd.services.tailscaled-poco = {
    description = "Tailscale node agent for Poco F1";
    after = [ "network-pre.target" "NetworkManager.service" "firewall.service" "poco-usb-uplink.service" ];
    wants = [ "network-online.target" "poco-usb-uplink.service" ];
    wantedBy = [ "multi-user.target" ];
    unitConfig.ConditionPathExists = "/home/spectre/tailscale/tailscaled";
    serviceConfig = {
      Type = "simple";
      Environment = [ "PATH=/run/current-system/sw/bin:/run/wrappers/bin:/usr/bin:/bin" ];
      RuntimeDirectory = "tailscale";
      StateDirectory = "tailscale";
      ExecStartPre = "+${usbUplinkScript}";
      ExecStart = "/home/spectre/tailscale/tailscaled --state=/var/lib/tailscale/tailscaled.state --socket=/run/tailscale/tailscaled.sock --port=41641";
      ExecStartPost = "+${usbUplinkScript}";
      Restart = "always";
      RestartSec = 3;
    };
  };

  environment.systemPackages = [ tailscaleWrapper ];
  system.extraDependencies = [ pkgs.iptables ];
}
