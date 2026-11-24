{}:
let
  phone = (import ../../. {}).pkgs;
  arm = import phone.path { system = "aarch64-linux"; config.allowUnfree = true; };
in phone.callPackage ./default.nix { runtime = arm; }
