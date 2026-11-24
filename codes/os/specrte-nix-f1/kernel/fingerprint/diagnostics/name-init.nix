# Explicit experiment: initialize the lookup buffer as the stock driver does.
(import ../default.nix).overrideAttrs (old: {
  patches = old.patches ++ [ ./qsee-name-init.patch ];
})
