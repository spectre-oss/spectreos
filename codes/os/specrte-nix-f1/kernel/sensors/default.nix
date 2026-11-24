# Diagnostic logging only. Normal kernel configuration is not changed.
(import ../fingerprint/default.nix).overrideAttrs (old: {
  patches = old.patches ++ [ ./0001-fastrpc-diagnostic.patch ];
})
