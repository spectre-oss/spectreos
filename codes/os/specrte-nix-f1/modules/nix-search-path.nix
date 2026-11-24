{ pkgs, ... }:
let
  # Retain the pinned source in the system closure so nix-shell works offline
  # for evaluation without a stateful root channel.
  source = builtins.path { path = pkgs.path; name = "nixpkgs"; };
in {
  nix.nixPath = [ "nixpkgs=${source}" "nixpkg=${source}" ];
}
