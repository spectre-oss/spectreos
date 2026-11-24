{ mobile-nixos, runCommand, buildPackages, fingerprintBringup ? false, sensorBringup ? false, irBringup ? false, ... }:
let
  # Keep Mobile NixOS boot support and overlay the config shipped with this
  # exact SDM845 kernel. Its device workarounds must track the kernel version.
  configfile = runCommand "beryllium-kernel-config" {} ''
    cat ${./config.aarch64} > $out
    # This port is monolithic. Avoid the fragment's optional version suffix.
    sed -e 's/=m$/=y/' -e '/^CONFIG_LOCALVERSION=/d' ${./sdm845.config} >> $out
    cat ${./beryllium.config} >> $out
    ${if fingerprintBringup then "cat ${./fingerprint/bringup.config} >> $out" else ""}
    ${if irBringup then "cat ${./ir/bringup.config} >> $out" else ""}
  '';
in (mobile-nixos.kernel-builder {
  version = "7.1.0-rc1";
  modDirVersion = "7.1.0-rc1";
  src = ./linux.tar.gz;
  inherit configfile;
  # Recent Adreno builds generate register headers with Python on the host.
  nativeBuildInputs = [ buildPackages.python3 ];
  preConfigure = ''
    python3 drivers/gpu/drm/msm/registers/gen_header.py --no-validate \
      --rnn drivers/gpu/drm/msm/registers \
      --xml drivers/gpu/drm/msm/registers/adreno/a6xx.xml c-defines > /dev/null
  '';
  isModular = false;
  isCompressed = "gz";
  enableForceLogoPatch = false;
  enableCenteredLinuxLogo = false;
  # Mainline uses explicit toolchain variables; vendor path rewrites and the
  # optional perf/selftest interpreter rewrites are not needed to build Image.gz.
  enableFHSPathStripping = false;
  enableToolsShebangPatching = false;
  # Report independent driver failures in one pass; any error still fails the build.
  makeFlags = [ "-k" "LOCALVERSION=" ];
 }).overrideAttrs (old: {
  # Recent arm64 zinstall uses a versioned installed filename. This metadata
  # tells image consumers which file to use without changing the kernel build.
  patches = (old.patches or []) ++ [
    ./patches/0002-imx363-unity-digital-gain.patch
    ./patches/0003-beryllium-sunny-ak7374-focus.patch
    ./patches/0004-ath10k-wcn3990-pairwise-key-removal.patch
    # Registration can trigger OCP before rdev is assigned, independently
    # of QSEECOM. Keep the proven IRQ ordering fix in normal kernels too.
    ./fingerprint/0002-labibb-defer-ocp-irq.patch
  ] ++ (if sensorBringup then [
    ./sensors/0002-beryllium-slpi-userspace-start.patch
  ] else [ ./patches/0001-beryllium-disable-slpi-for-boot-recovery.patch ]) ++ (if fingerprintBringup then [
    ./fingerprint/0001-qseecom-tee.patch
  ] else []) ++ (if irBringup then [ ./ir/0001-ov7251-beryllium-strobe.patch ] else []);
  postPatch = (old.postPatch or "") + (if irBringup then ''
    cp ${./ir/leds-lm3644-ir.c} drivers/leds/leds-lm3644-ir.c
    echo 'obj-y += leds-lm3644-ir.o' >> drivers/leds/Makefile
    cat ${./ir/board.dtsi} >> arch/arm64/boot/dts/qcom/sdm845-xiaomi-beryllium-common.dtsi
  '' else "");
  passthru = old.passthru // { file = "vmlinuz-${old.version}"; };
})
