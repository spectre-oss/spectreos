{ panel ? "tianma", ... }@args:
import ./mobile-nixos/lib/eval-with-configuration.nix ((builtins.removeAttrs args [ "panel" ]) // {
  device = ./device.nix;
  configuration = [ ./configuration.nix { _module.args.f1Panel = panel; } ];
})
