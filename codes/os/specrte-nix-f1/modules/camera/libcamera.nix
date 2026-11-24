# Keep the pinned package and its IPA signing hook. Build only the pipeline
# required by this phone; desktop/GStreamer bindings are not needed by the app.
{ base, runtime ? base }:
(base.libcamera.override { withTracing = false; withQcam = false; }).overrideAttrs (old: {
  patches = (old.patches or []) ++ [
    ../../patches/libcamera-imx363-helper.patch
    ../../patches/libcamera-softisp-initialize-params.patch
    ../../patches/libcamera-imx363-digital-gain.patch
    ../../patches/libcamera-imx363-preview-mode.patch
    ../../patches/libcamera-softisp-preview-reduce.patch
    ../../patches/libcamera-softisp-black-level.patch
    ../../patches/libcamera-softisp-ccm-highlight.patch
    ../../patches/libcamera-softisp-highlight-guard.patch
    ../../patches/libcamera-softisp-denoise.patch
    ../../patches/libcamera-imx363-defects.patch
    ../../patches/libcamera-simple-manual-capture.patch
    ../../patches/libcamera-imx363-shading.patch
  ];
  postPatch = (old.postPatch or "") + ''
    cp ${./highlight-guard.h} src/ipa/simple/algorithms/highlight-guard.h
    cp ${./raw10-reduce.h} src/libcamera/software_isp/raw10-reduce.h
    cp ${./raw10-denoise.h} src/libcamera/software_isp/raw10-denoise.h
    cp ${./noise-profile.h} src/libcamera/software_isp/noise-profile.h
    cp ${./raw10-defects.h} src/libcamera/software_isp/raw10-defects.h
    cp ${./defect-profile.h} src/libcamera/software_isp/defect-profile.h
    cp ${./manual-capture.h} src/ipa/simple/manual-capture.h
    cp ${./raw10-shading.h} src/libcamera/software_isp/raw10-shading.h
    cp ${./shading-profile.h} src/libcamera/software_isp/shading-profile.h
  '';
  postInstall = (old.postInstall or "") + ''
    cp ${./imx363.yaml} $out/share/libcamera/ipa/simple/imx363.yaml
  '';
  buildInputs = with runtime; [ openssl libyaml udev libevent libdrm ];
  mesonFlags = [
    "-Dpipelines=simple" "-Dipas=simple" "-Dcam=enabled"
    "-Dqcam=disabled" "-Dgstreamer=disabled" "-Dpycamera=disabled"
    "-Dv4l2=disabled" "-Dtracing=disabled" "-Ddocumentation=disabled"
    "-Dlibunwind=disabled" "-Dlc-compliance=disabled" "-Drpi-awb-nn=disabled"
    "-Dwerror=false"
  ];
})
