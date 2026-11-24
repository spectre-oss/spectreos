#!/usr/bin/env bash
set -euo pipefail
mode=${1:?boot or install}; images=${2:?image directory or boot image}
[[ "$mode" == boot || "$mode" == install ]] || exit 2
command -v fastboot >/dev/null
boot_image="$images/boot.img"
if [[ "$mode" == boot && -f "$images" ]]; then boot_image=$images; fi
[[ -f "$boot_image" ]] || { echo "Build images first with make boot-image or make build" >&2; exit 1; }
[[ $(head -c 8 "$boot_image") == 'ANDROID!' ]] || { echo 'Invalid Android boot image' >&2; exit 1; }
mapfile -t devices < <(fastboot devices | awk '$2 == "fastboot" {print $1}')
if [[ -n ${SERIAL:-} ]]; then
  printf '%s\n' "${devices[@]}" | grep -Fxq "$SERIAL" || { echo 'Selected device not found' >&2; exit 1; }
else
  [[ ${#devices[@]} == 1 ]] || { echo 'Connect exactly one phone, or select SERIAL=' >&2; exit 1; }
  SERIAL=${devices[0]}
fi
fb=(fastboot -s "$SERIAL")
product=$("${fb[@]}" getvar product 2>&1)
printf '%s\n' "$product"
printf '%s\n' "$product" | grep -Eq 'product: *beryllium([[:space:]]|$)' || { echo 'Refusing: not a Poco F1' >&2; exit 1; }
if [[ "$mode" == boot ]]; then
  exec "${fb[@]}" boot "$boot_image"
fi
[[ ${ERASE_USERDATA:-} == beryllium ]] || { echo 'This erases userdata. Back up first, then use make install ERASE_USERDATA=beryllium' >&2; exit 1; }
[[ -f "$images/system.img" ]] || { echo 'Missing system.img' >&2; exit 1; }
unlocked=$("${fb[@]}" oem device-info 2>&1)
printf '%s\n' "$unlocked" | grep -Eiq 'Device unlocked: *true' || { echo 'Bootloader must already be unlocked' >&2; exit 1; }
# Confirm the boot image fits before writing either partition.
boot_size_output=$("${fb[@]}" getvar partition-size:boot 2>&1)
boot_size=$(printf '%s\n' "$boot_size_output" | sed -nE 's/.*partition-size:boot: *(0x[0-9a-fA-F]+|[0-9]+).*/\1/p' | head -n1)
[[ -n "$boot_size" ]] || { echo 'Cannot verify boot partition size' >&2; exit 1; }
(( $(stat -c %s "$boot_image") <= boot_size )) || { echo 'Boot image exceeds boot partition size' >&2; exit 1; }
# Rootfs first: a failed transfer must not replace the existing boot image.
"${fb[@]}" flash userdata "$images/system.img"
"${fb[@]}" flash boot "$boot_image"
echo 'Installation complete. Recovery and firmware partitions were preserved.'
"${fb[@]}" reboot
