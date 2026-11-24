{ stdenv, buildPackages, lib }:
stdenv.mkDerivation {
  pname = "spectre-uboot-beryllium";
  version = "2026.10-g8d534cec";
  src = ./u-boot.tar.gz;
  patches = [ ./patches/0001-qcom-register-usb-ethernet.patch ];
  nativeBuildInputs = with buildPackages; [
    bison flex swig pkg-config openssl dtc xxd
    (python3.withPackages (p: [ p.setuptools p.pyelftools ]))
  ];
  depsBuildBuild = [ buildPackages.stdenv.cc ];
  enableParallelBuilding = true;
  hardeningDisable = [ "all" ];
  makeFlags = [
    "CROSS_COMPILE=${stdenv.cc.targetPrefix}"
    "HOSTCC=${buildPackages.stdenv.cc}/bin/cc"
    "HOSTCXX=${buildPackages.stdenv.cc}/bin/c++"
  ];
  postPatch = "patchShebangs scripts tools";
  configurePhase = ''
    runHook preConfigure
    cp ${./spectre.env} board/qualcomm/spectre.env
    make $makeFlags qcom_defconfig
    scripts/kconfig/merge_config.sh -m .config ${./spectre.config}
    make $makeFlags olddefconfig
    grep -qx 'CONFIG_NETCONSOLE=y' .config
    grep -qx 'CONFIG_USB_ETHER=y' .config
    grep -qx 'CONFIG_BOOTDELAY=-1' .config
    runHook postConfigure
  '';
  installPhase = ''
    mkdir -p $out/bin $out/dtbs
    cp u-boot-nodtb.bin $out/
    cp .config $out/build.config
    cp dts/upstream/src/arm64/qcom/sdm845-xiaomi-beryllium-tianma.dtb $out/dtbs/
    cp tools/mkimage $out/bin/
  '';
  dontFixup = true;
}
