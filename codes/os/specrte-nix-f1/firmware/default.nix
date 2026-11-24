{ runCommand, lib, linux-firmware }:
runCommand "pocophone-f1-firmware" {
  meta.license = lib.licenses.unfree;
  # ath10k can complete its asynchronous probe after switching to stage 2.
  # Keep the device firmware available by its original filename in both stages.
  passthru.compressFirmware = false;
} ''
  mkdir -p unpack $out/lib/firmware
  tar -xzf ${./beryllium.tar.gz} -C unpack --strip-components=1
  cp -r unpack/lib/firmware/. $out/lib/firmware/
  # SLPI requests DSP libraries and sensor configuration through FastRPC,
  # separately from the kernel firmware loader. Keep the archive's HexagonFS
  # tree so the userspace listener can serve those requests.
  mkdir -p $out/share
  cp -r unpack/usr/share/qcom $out/share/
  chmod -R u+w $out
  # The vendor archive supplies calibration and wlanmdsp.mbn, but ath10k
  # also needs its API metadata. Preserve the phone-specific board-2.bin.
  cp ${linux-firmware}/lib/firmware/ath10k/WCN3990/hw1.0/firmware-5.bin \
    $out/lib/firmware/ath10k/WCN3990/hw1.0/
  mkdir -p $out/lib/firmware/qca $out/lib/firmware/qcom
  cp -r ${linux-firmware}/lib/firmware/qca/. $out/lib/firmware/qca/
  cp ${linux-firmware}/lib/firmware/qcom/a630_{sqe.fw,gmu.bin} $out/lib/firmware/qcom/

''
