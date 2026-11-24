# No fingerprint/QSEECOM experiments or diagnostic logging in this kernel.
let project = import ../.. {};
in project.pkgs.callPackage ../default.nix { sensorBringup = true; }
