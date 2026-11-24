# Build explicitly; the normal phone kernel remains unchanged until hardware tests.
let project = import ../.. {};
in project.pkgs.callPackage ../default.nix { fingerprintBringup = true; }
