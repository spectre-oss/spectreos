# Standalone IR diagnostic uses the same implementation as the installed kernel.
let project = import ../.. {};
in project.pkgs.callPackage ../default.nix { sensorBringup = true; irBringup = true; }
