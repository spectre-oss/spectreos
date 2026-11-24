# Explicit opt-in diagnostic; installed/normal kernel is unchanged.
(import ../default.nix).overrideAttrs (old: {
  patches = old.patches ++ [ ./qsee-whitelist.patch ];
})
