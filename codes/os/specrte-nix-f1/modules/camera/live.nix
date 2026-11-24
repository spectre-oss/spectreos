# Cross-compile the camera app on the x86 build server using cached ARM64
# desktop libraries, following the project's existing live-Phosh strategy.
# Full system images use the ordinary overlays in ../camera.nix.
{}:
let
  phone = (import ../../. {}).pkgs;
  arm = import phone.path { system = "aarch64-linux"; config.allowUnfree = true; };
  host = import phone.path { system = "x86_64-linux"; };
  camera = phone.megapixels.override {
    glib = host.glib;
    # Keep the cross-aware wrapper hook: the host-only hook emits an x86 ELF
    # launcher even though the camera executable itself is ARM64.
    wrapGAppsHook4 = phone.wrapGAppsNoGuiHook;
    inherit (arm) feedbackd gtk4 libdng libepoxy libpulseaudio libxrandr zbar
      libraw graphicsmagick exiftool;
    libmegapixels = phone.libmegapixels;
  };
in camera.overrideAttrs (_old: {
  # Official ARM64 libraries carry GCC stack-protector references.
  NIX_LDFLAGS = "-L${arm.stdenv.cc.cc.lib}/lib -lssp";
})
