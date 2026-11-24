{ lib, stdenv, makeWrapper, runtime, bash, coreutils, util-linux, fetchurl, linuxHeaders }:
let
  supplicant = fetchurl {
    url = "https://codeload.github.com/wrobelda/qsee-supplicant/tar.gz/36e06680cf7f690fccbdcd07abc2a64c4bb061d8";
    sha256 = "a94eb2024d3431a7eeba63d2766ea0ad1984b6314e0c8b084ab0bb0cbaf03e08";
  };
  minkipc = fetchurl {
    url = "https://codeload.github.com/qualcomm/minkipc/tar.gz/5e3c6951d13162a262ea65ea8e8351228f092b58";
    sha256 = "2da103029796465773fa77746cf10799ff1fb43c62ba8dbb1efc51b62e3dd36f";
  };
  diagnostics = ../../kernel/fingerprint/diagnostics;
in
stdenv.mkDerivation {
  pname = "spectre-fingerprint";
  version = "0.3.0";
  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type: builtins.baseNameOf path != "__pycache__";
  };
  nativeBuildInputs = [ makeWrapper ];
  dontConfigure = true;
  buildPhase = ''
    mkdir supplicant minkipc rpmb-vendor scsi
    cp ${linuxHeaders}/include/scsi/scsi_bsg_ufs.h scsi/
    tar xf ${supplicant} -C supplicant --strip-components=1
    tar xf ${minkipc} -C minkipc --strip-components=1
    cp minkipc/listeners/librpmbservice/*.[ch] rpmb-vendor/
    $CC -O2 -Wall -Wextra -Werror -DFINGERPRINT_TA='"gfdip"' -DCAPTURE_PROBE \
      ${diagnostics}/gatekeeper-preflight.c -o gfber-identify
    $CC -O2 -Wall -Wextra -Werror -D_GNU_SOURCE -DRPMB_DEBUG_CONSOLE_OUTPUT \
      -I. -Isupplicant/include supplicant/src/{main,transport_qseecom,path,handle_db,fs,gpfs,notify}.c \
      ${diagnostics}/storage-services.c -pthread -o storage-broker
  '';
  installPhase = ''
    mkdir -p $out/{bin,libexec,share/spectre-fingerprint,share/applications}
    cp gfber-identify storage-broker $out/libexec/
    cp ${../../kernel/fingerprint/run-info.sh} $out/libexec/run-info.sh
    substituteInPlace $out/libexec/run-info.sh --replace-fail '#!/bin/sh' '#!${bash}/bin/bash'
    chmod +x $out/libexec/run-info.sh
    wrapProgram $out/libexec/run-info.sh --prefix PATH : ${lib.makeBinPath [ coreutils util-linux ]}
    cp backend.py app.js $out/share/spectre-fingerprint/
    makeWrapper ${runtime.python3}/bin/python3 $out/bin/fingerprint-client --add-flags "$out/share/spectre-fingerprint/backend.py"
    makeWrapper ${runtime.python3}/bin/python3 $out/bin/fingerprint-service --add-flags "$out/share/spectre-fingerprint/backend.py --helper $out/libexec/run-info.sh"
    makeWrapper ${runtime.gjs}/bin/gjs $out/bin/spectre-fingerprint \
      --add-flags "-m $out/share/spectre-fingerprint/app.js" \
      --set FINGERPRINT_CLIENT $out/bin/fingerprint-client \
      --prefix GI_TYPELIB_PATH : ${lib.makeSearchPathOutput "out" "lib/girepository-1.0" [ runtime.gobject-introspection runtime.gtk4 runtime.glib runtime.pango runtime.gdk-pixbuf runtime.graphene runtime.harfbuzz ]}
    cat > $out/share/applications/org.spectre.Fingerprint.desktop <<DESKTOP
    [Desktop Entry]
    Name=Fingerprint Lab
    Comment=Test your Poco F1 fingerprint sensor
    Exec=$out/bin/spectre-fingerprint
    Icon=fingerprint-symbolic
    Type=Application
    Terminal=false
    Categories=GTK;Utility;
    X-Purism-FormFactor=Workstation;Mobile;
    DESKTOP
  '';
  meta = { description = "Poco F1 fingerprint hardware workbench"; license = lib.licenses.gpl2Only; };
}
