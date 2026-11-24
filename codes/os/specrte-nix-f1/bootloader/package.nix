{ runCommand, callPackage, buildPackages, linuxBoot }:
let uboot = callPackage ./. {};
in runCommand "spectre-uboot-recovery-package" {
  nativeBuildInputs = [ buildPackages.python3 buildPackages.dtc ];
  SOURCE_DATE_EPOCH = "1792161762";
} ''
  python3 ${./package.py} --linux-boot ${linuxBoot} --uboot ${uboot} --output $out
''
