# Explicit diagnostic build; not imported by the regular phone configuration.
(import ../default.nix).overrideAttrs (old: {
  patches = old.patches ++ [ ./qsee-log.patch ];
})
