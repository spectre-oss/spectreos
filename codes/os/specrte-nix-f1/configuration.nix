{ lib, pkgs, ... }:
{
  imports = [ ./modules/wisp.nix ./modules/shiraz.nix ./modules/sensors.nix ./modules/usb-ssh.nix ./modules/phone-debug.nix ./modules/audio.nix ./modules/camera.nix ./modules/fingerprint-lab.nix ./modules/keyboard.nix ./modules/fonts.nix ./modules/nix-search-path.nix ./modules/qualcomm-shutdown.nix ./modules/boot-reliability.nix ];
  spectre.sensors.enable = true;
  spectre.phoneDebug.enable = true;
  spectre.fingerprintLab.enable = true;
  # GTK4 uses native Wayland zwp_text_input_v3 automatically when GTK_IM_MODULE is unset
  # (setting GTK_IM_MODULE=wayland triggers GTK4 warnings and breaks SpectreOS Keyboard integration).
  # Let the phone owner select a persistent zone through systemd-timedated.
  time.timeZone = null;
  environment.etc."polkit-1/rules.d/50-wisp-timezone.rules".source = ./modules/50-wisp-timezone.rules;
  networking.hostName = "specrte-nix-f1";
  networking.networkmanager.enable = true;
  # Retain the package database itself in the image closure, rather than a
  # reference into a build-time Nixpkgs checkout that may not be installed.
  programs.command-not-found = {
    enable = true;
    dbPath = builtins.path {
      path = pkgs.path + "/programs.sqlite";
      name = "programs.sqlite";
    };
  };
  # Cross-built Perl reports a GNU target triple; the database uses Nix names.
  environment.sessionVariables.NIX_SYSTEM = pkgs.stdenv.hostPlatform.system;
  system.stateVersion = "26.05";
  nixpkgs.config.allowUnfree = true;
  documentation.enable = false;
  # Fonts contain no target machine code. Reuse the cached native package
  # rather than regenerating thousands of emoji bitmaps during cross builds.
  nixpkgs.overlays = [ (final: prev: {
    # Match Qt's bootstrap/native tools to its cross scope. The pinned Qt5
    # scope disables translations for cross builds but its native qmake hook
    # otherwise propagates a different qtbase, failing the consistency check.
    qt5 = prev.qt5.overrideScope (_qtFinal: qtPrev: {
      qtbase = (qtPrev.qtbase.override { qttranslations = null; }).overrideAttrs (old: {
        # This phone does not need Qt's MySQL client plugin. Its configure
        # probe fails in the cross build; keep native and target scopes aligned.
        configureFlags = builtins.filter (flag: flag != "-sql-mysql")
          (old.configureFlags or []) ++ [ "-no-sql-mysql" ];
      });
      qttranslations = null;
    });
  }) (final: prev:
    if prev.stdenv.buildPlatform == prev.stdenv.hostPlatform then {} else {
      # systemd 261 invokes raw build-host clang for BPF. Its separate flags
      # omit the target libc/kernel headers normally supplied by Nix's wrapper.
      systemd = prev.systemd.overrideAttrs (old: {
        postPatch = (old.postPatch or "") + ''
          substituteInPlace src/bpf/meson.build \
            --replace-fail "bpf_clang_flags = [" \
              "bpf_clang_flags = [ '-idirafter', '${lib.getDev final.stdenv.cc.libc}/include', '-idirafter', '${final.stdenv.cc.libc.linuxHeaders}/include',"
        '';
      });
      # Megapixels consumes libzbar's image scanner, not the standalone
      # GTK/Qt video-capture applications and their cross-build dependencies.
      zbar = prev.zbar.override { enableVideo = false; };
      # GNU ld was killed linking the GTK4 WebKit library with debug info.
      # Keep the runtime engine, omit debug symbols, and release linker caches.
      webkitgtk_6_0 = prev.webkitgtk_6_0.overrideAttrs (old: {
        separateDebugInfo = false;
        cmakeFlags = (old.cmakeFlags or []) ++ [
          "-DCMAKE_SHARED_LINKER_FLAGS=-Wl,--no-keep-memory"
        ];
      });
      # Poco F1 uses Qualcomm Adreno, so FFmpeg must not compile CUDA kernels.
      ffmpeg_8 = prev.ffmpeg_8.override { withCudaLLVM = false; };
      ffmpeg_8-headless = prev.ffmpeg_8-headless.override { withCudaLLVM = false; };
      ffmpeg_8-full = prev.ffmpeg_8-full.override { withCudaLLVM = false; };
      ffmpeg = final.ffmpeg_8;
      ffmpeg-headless = final.ffmpeg_8-headless;
      ffmpeg-full = final.ffmpeg_8-full;
      # Python runs at build time to install pure overrides into the target path.
      gom = prev.gom.overrideAttrs (old: {
        nativeBuildInputs = (old.nativeBuildInputs or []) ++ [ prev.buildPackages.python3 ];
        postPatch = (old.postPatch or "") + ''
          substituteInPlace bindings/python/meson.build \
            --replace-fail "find_installation('python3')" \
              "find_installation('${prev.buildPackages.python3.interpreter}')"
        '';
      });
      # Manpage/completion generation imports GI in Python on the build host.
      # pythonOnBuildForHost supplies target GI libraries during this cross build.
      power-profiles-daemon = prev.power-profiles-daemon.overrideAttrs (old: {
        # Meson's Python generators execute locally and must load native GI.
        # Restore the target search path before runtime wrappers are generated.
        preConfigure = (old.preConfigure or "") + ''
          f1TargetGiTypelibPath="''${GI_TYPELIB_PATH-}"
          export GI_TYPELIB_PATH="${lib.getLib prev.buildPackages.glib}/lib/girepository-1.0"
        '';
        preFixup = (old.preFixup or "") + ''
          export GI_TYPELIB_PATH="$f1TargetGiTypelibPath"
        '';
        nativeBuildInputs = builtins.filter
          (input: !(lib.hasPrefix "python3-" (lib.getName input)))
          (old.nativeBuildInputs or []) ++ [
            (prev.buildPackages.python3.withPackages (ps: with ps; [
              pygobject3 dbus-python python-dbusmock argparse-manpage shtab
            ]))
          ];
      });
      # Both gi-docgen lookups must use the build host during cross builds.
      gupnp-av = prev.gupnp-av.overrideAttrs (old: {
        patches = (old.patches or []) ++ [ ./patches/gupnp-av-native-docgen.patch ];
      });
      noto-fonts-color-emoji = prev.buildPackages.noto-fonts-color-emoji;
      # JSON device profiles also need native build-time validation tools.
      feedbackd-device-themes = prev.buildPackages.feedbackd-device-themes;
    }) ];
  # Initial local unlock PIN; change with passwd after first boot.
  users.users.spectre = {
    isNormalUser = true;
    initialPassword = "123456";
    extraGroups = [ "wheel" "networkmanager" "video" "render" "input" "dialout" "feedbackd" ];
  };
  mobile.boot.stage-1.kernel.useStrictKernelConfig = false;
  # The image is small; grow the filesystem inside existing userdata, not the GPT.
  boot.growPartition = lib.mkForce false;
  fileSystems."/" = {
    device = "/dev/disk/by-label/NIXOS_SYSTEM";
    fsType = "ext4";
    autoResize = true;
  };
  nix.settings = { auto-optimise-store = true; keep-outputs = false; keep-derivations = false; };
  nix.gc = { automatic = true; dates = "weekly"; options = "--delete-older-than 14d"; };
  services.journald.extraConfig = "SystemMaxUse=64M\nRuntimeMaxUse=32M\nMaxRetentionSec=7day";
  zramSwap = { enable = true; memoryPercent = 25; algorithm = "zstd"; };
  environment.systemPackages = [ pkgs.git
    # Use the pinned native ARM64 package so its official binary cache is
    # reusable on both the live phone and subsequent system images.
    (import pkgs.path { system = pkgs.stdenv.hostPlatform.system; }).nautilus
  ];
}
