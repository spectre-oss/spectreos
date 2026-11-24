{ lib, stdenv, cmake, ninja, pkg-config, makeWrapper, qt6, libcamera
, qtHost ? qt6.qtbase }:
stdenv.mkDerivation {
  pname = "spectre-camera";
  version = "0.4.4";
  src = ./.;
  nativeBuildInputs = [ cmake ninja pkg-config makeWrapper ];
  buildInputs = [ qt6.qtbase qt6.qtdeclarative qt6.qtwayland libcamera ];
  dontWrapQtApps = true;
  postPatch = ''
    cp ${../../modules/camera/noise-profile.h} noise-profile.h
  '';
  cmakeFlags = [ "-DQT_HOST_PATH=${qtHost}" "-DQT_HOST_PATH_CMAKE_DIR=${qtHost}/lib/cmake" ];
  postFixup = ''
    wrapProgram $out/bin/spectre-capture-probe \
      --set LIBCAMERA_IPA_MODULE_PATH "${libcamera}/lib/libcamera/ipa" \
      --set LIBCAMERA_IPA_PROXY_PATH "${libcamera}/libexec/libcamera" \
      --set LIBCAMERA_IPA_CONFIG_PATH "${libcamera}/share/libcamera/ipa" \
      --set-default SPECTRE_IMX363_FULL_SENSOR 1 \
      --set-default SPECTRE_IMX363_RAW_DENOISE 1 \
      --set-default SPECTRE_IMX363_SHADING colour \
      --set-default SPECTRE_IMX363_SUNNY_DEFECTS 1
    wrapProgram $out/bin/spectre-camera \
      --prefix QT_PLUGIN_PATH : "${qt6.qtbase}/${qt6.qtbase.qtPluginPrefix}:${qt6.qtwayland}/${qt6.qtbase.qtPluginPrefix}" \
      --prefix QML_IMPORT_PATH : "${qt6.qtdeclarative}/${qt6.qtbase.qtQmlPrefix}" \
      --set LIBCAMERA_IPA_MODULE_PATH "${libcamera}/lib/libcamera/ipa" \
      --set LIBCAMERA_IPA_PROXY_PATH "${libcamera}/libexec/libcamera" \
      --set LIBCAMERA_IPA_CONFIG_PATH "${libcamera}/share/libcamera/ipa" \
      --set QT_QUICK_CONTROLS_STYLE Basic \
      --set-default SPECTRE_IMX363_FULL_SENSOR 1 \
      --set-default SPECTRE_IMX363_RAW_DENOISE 1 \
      --set-default SPECTRE_IMX363_SUNNY_DEFECTS 1 \
      --set-default SPECTRE_IMX363_SHADING colour \
      --set-default QT_QPA_PLATFORM wayland
  '';
  meta = {
    description = "A touch-first Qt camera for Spectre Nix on the Poco F1";
    license = lib.licenses.gpl3Plus;
    mainProgram = "spectre-camera";
    platforms = lib.platforms.linux;
  };
}
