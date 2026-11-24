# Main rear IMX363 camera: the kernel pipeline exists, but upstream
# libmegapixels 0.2.3 does not install a Poco F1 configuration.
{ pkgs, ... }:
{
  nixpkgs.overlays = [ (final: prev: {
    libmegapixels = prev.libmegapixels.overrideAttrs (old: {
      postInstall = (old.postInstall or "") + ''
        install -Dm644 ${./camera}/xiaomi,beryllium.conf \
          "$out/share/megapixels/config/xiaomi,beryllium.conf"
      '';
    });
    spectre-libcamera = import ./camera/libcamera.nix { base = prev; };
    spectre-camera = final.callPackage ../apps/spectre-camera {
      # Use the same complete cached ARM Qt runtime as the tested live app.
      # The cross Qt scope omits Quick, which this QML application requires.
      qt6 = (import prev.path { system = prev.stdenv.hostPlatform.system; }).qt6;
      qtHost = prev.buildPackages.qt6.qtbase;
      libcamera = final.spectre-libcamera;
    };
    megapixels = prev.megapixels.overrideAttrs (old: {
      patches = (old.patches or []) ++ [
        ../patches/megapixels-initialize-multiplane-buffers.patch
        ../patches/megapixels-release-blank-frame-buffers.patch
      ];
    });
  }) ];
  # Ship the library's config discovery and single-frame diagnostic tools too.
  environment.systemPackages = [ pkgs.libmegapixels pkgs.spectre-camera ];
}
