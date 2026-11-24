{ pkgs, ... }:
let
  persianFontConfig = ''
    <match target="pattern">
      <test name="lang" compare="contains"><string>fa</string></test>
      <edit name="family" mode="prepend" binding="strong"><string>Shabnam</string></edit>
    </match>
  '';
in {
  fonts.packages = [
    pkgs.buildPackages.roboto
    pkgs.buildPackages.shabnam-fonts
  ];
  fonts.fontconfig = {
    defaultFonts.sansSerif = [ "Roboto" "Shabnam" ];
    localConf = persianFontConfig;
  };
}
