# Run on the build server. Cached native ARM64 Qt libraries, cross compiler.
{ native ? false }:
let
  phone = (import ../../. {}).pkgs;
  host = import phone.path { system = "x86_64-linux"; config.allowUnfree = true; };
  arm = import phone.path { system = "aarch64-linux"; config.allowUnfree = true; };
  base = if native then host else phone;
  qt = if native then host.qt6 else arm.qt6;
  cameraLib = import ../../modules/camera/libcamera.nix {
    base = base; runtime = if native then host else arm;
  };
in base.callPackage ./default.nix {
  qt6 = qt;
  qtHost = host.qt6.qtbase;
  libcamera = cameraLib;
}
