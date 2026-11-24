{ pkgs }:
let native = import pkgs.path { system = pkgs.stdenv.hostPlatform.system; };
in pkgs.stdenv.mkDerivation {
  name = "spectre-keyboard-test-client";
  src = ./client.c;
  dontUnpack = true;
  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ native.gtk3 ];
  buildPhase = ''$CC $src $($PKG_CONFIG --cflags --libs gtk+-3.0) -o keyboard-test'';
  installPhase = ''mkdir -p $out/bin; cp keyboard-test $out/bin/spectre-keyboard-test'';
}
