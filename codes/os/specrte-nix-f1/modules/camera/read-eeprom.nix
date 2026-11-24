# Diagnostic only; build on the designated build host, never on the phone.
let pkgs = (import ../../. {}).pkgs;
in pkgs.stdenv.mkDerivation {
  name = "spectre-read-camera-eeprom";
  src = ./read-eeprom.c;
  dontUnpack = true;
  buildPhase = "$CC -O2 -Wall -Wextra $src -o read-camera-eeprom";
  installPhase = "mkdir -p $out/bin; cp read-camera-eeprom $out/bin/";
}
