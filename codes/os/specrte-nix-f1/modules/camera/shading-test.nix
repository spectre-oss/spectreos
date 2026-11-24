# Build only on the build host; run the ARM executable on the phone.
{ stdenv }:
stdenv.mkDerivation {
  pname = "spectre-shading-test";
  version = "1";
  src = ./.;
  dontConfigure = true;
  buildPhase = ''
    $CXX -std=c++17 -O2 -Wall -Wextra test-raw10-shading.cpp -o spectre-shading-test
  '';
  installPhase = ''
    install -Dm755 spectre-shading-test $out/bin/spectre-shading-test
  '';
}
