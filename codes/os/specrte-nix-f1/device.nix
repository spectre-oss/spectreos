{ config, lib, pkgs, f1Panel ? "tianma", ... }:
{
  imports = [ ./mobile-nixos/devices/families/sdm845-mainline ];
  assertions = [{ assertion = builtins.elem f1Panel [ "tianma" "ebbg" ]; message = "Choose PANEL=tianma or PANEL=ebbg"; }];
  mobile.device = {
    name = "xiaomi-beryllium";
    identity = { name = "Pocophone F1"; manufacturer = "Xiaomi"; };
    supportLevel = "unsupported"; # Experimental port until hardware tested.
    firmware = pkgs.callPackage ./firmware {};
  };
  mobile.hardware = { ram = 6 * 1024; screen = { width = 1080; height = 2246; }; };
  # Combine the independently tested transports in the installed phone kernel.
  # IR illumination stays off until an explicit root-only bounded request.
  mobile.boot.stage-1.kernel.package = lib.mkForce (pkgs.callPackage ./kernel {
    sensorBringup = config.spectre.sensors.enable;
    fingerprintBringup = config.spectre.fingerprintLab.enable;
    irBringup = true;
  });
  # Linux 7.1 uses the vendor-qualified firmware directory. Keep only early
  # remote-processor/GPU and Wi-Fi firmware in the initrd; the modem is loaded
  # in stage 2. The built-in ath10k driver probes before the root switch.
  mobile.boot.stage-1.firmware = lib.mkForce [
    (pkgs.runCommand "beryllium-initrd-firmware" {} ''
      mkdir -p $out/lib/firmware/qcom $out/lib/firmware/qca
      cp -r ${config.mobile.device.firmware}/lib/firmware/ath10k $out/lib/firmware/
      # Built-in radio drivers request these before switching to the rootfs.
      cp ${config.mobile.device.firmware}/lib/firmware/qca/{crbtfw21.tlv,crnv21.bin} $out/lib/firmware/qca/
      cp ${pkgs.wireless-regdb}/lib/firmware/regulatory.db* $out/lib/firmware/
      # The amplifier also probes before stage 2; missing DSP firmware puts
      # its first playback into the driver's safeguard/reset path.
      cp ${config.mobile.device.firmware}/lib/firmware/tas2559_uCDSP.bin $out/lib/firmware/
      cp -r ${config.mobile.device.firmware}/lib/firmware/qcom/sdm845 $out/lib/firmware/qcom/
      cp ${config.mobile.device.firmware}/lib/firmware/qcom/a630_{sqe.fw,gmu.bin} $out/lib/firmware/qcom/
      chmod -R u+w $out
      find $out/lib/firmware/qcom/sdm845 -name modem.mbn -delete
      # SLPI starts only after the stage-2 listener and registry are ready.
      find $out/lib/firmware/qcom/sdm845 -name slpi.mbn -delete
    '')
  ];
  mobile.system.android = {
    device_name = "beryllium";
    ab_partitions = false;
    system_partition_destination = "userdata";
    appendDTB = lib.mkForce [ "dtbs/qcom/sdm845-xiaomi-beryllium-${f1Panel}.dtb" ];
    bootimg.flash.offset_second = lib.mkForce "0x00f00000";
  };
  # Ship only the firmware this phone needs, not the complete PC firmware bundle.
  hardware.enableRedistributableFirmware = lib.mkForce false;
  boot.kernelParams = [ "console=tty0" ];
}
