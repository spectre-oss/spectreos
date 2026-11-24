# Stock Android sensor baseline and root investigation

Date: 2026-11-03. Device: Xiaomi Poco F1 / `beryllium`.

## Authorization and objective

The owner explicitly requested flashing stock Android, rooting it, testing the
front IR face-unlock camera, proximity/light sensing, and fingerprint, and
documenting the work. After being informed that Linux uses 44 GB (32GB in home),
the owner confirmed that the phone's existing files are not needed. Replacing
Linux userdata is therefore authorized. Do not relock the bootloader.

The Linux fingerprint transport can detect/initialize the 88x108 Goodix sensor,
but enrollment fails at the Keymaster wrapped-key exchange. Front/IR camera
and proximity/light operation have not been established in this Linux setup.
This does not prove missing individual drivers or defective physical sensors.

## Baseline and backups

Before modification: Linux 7.1.0-rc1, userdata 53.5 GiB ext4, 44 GiB used. Phone
partition table uses single boot/system/vendor/userdata with paired firmware
partitions. Stock-ROM hardware identity is `beryllium`.

Eleven raw partition backups were compressed and independently checked against
device SHA256: boot, recovery, dtbo, vbmeta, persist, persistbak, modemst1,
modemst2, fsg, fsc, devinfo. Raw total 319160320 bytes. They are private artifacts
under `.build/stock-android/backup-2026-11-03/`; manifest includes raw sizes,
raw hashes and archive hashes. These are NOT a backup of Linux userdata and
cannot restore discarded home files. Preserve calibration/radio identity data.

Original installed boot SHA256:
`8c27a090763dc1a31ec4cd8b0419756a81d876cf40e9b5b9589f28ffebbbfdf0`.

## Target stock software

MIUI Global V12.0.3.0.QEJMIXM, Android 10, security patch 2020-12-01.
Recovery package already present:
`miui_POCOF1Global_V12.0.3.0.QEJMIXM_cf3fccffce_10.0.zip`.
Metadata identifies Xiaomi/beryllium/beryllium:10/QKQ1.190828.002/
V12.0.3.0.QEJMIXM:user/release-keys.

Stock boot from that package SHA256:
`bc7960ca0f2078c90752f121adf7aae4e13675c88e69624141ceefee520333e1`.
The recovery package does not contain standalone dtbo/vbmeta images. The full fastboot download failed/throttled. Its extracted official flash script
also does not write dtbo/vbmeta. Reconstructed the complete system/vendor images
from the verified recovery archive with `infra/stock-android-images.py`. All 27
installed firmware image ranges match this ROM, so firmware stays in place. Do not execute bundled flash/lock scripts blindly.
Do not overwrite persist or radio calibration partitions with generic images.

Official fastboot archive URL:
https://bigota.d.miui.com/V12.0.3.0.QEJMIXM/beryllium_global_images_V12.0.3.0.QEJMIXM_20201227.0000.00_10.0_global_f4a9dbd1d3.tgz

## Test order and acceptance criteria

Test stock Android before adding root; record results separately after rooting.
The owner supplies finger/face interaction and chooses any required screen PIN.
Do not save fingerprint images, auth tokens, private keys, or face captures in
project logs. Redact personal identifiers from any shareable diagnostic output.

| Feature | Required observation | Before root | After root |
| --- | --- | --- | --- |
| Fingerprint | Enroll successfully, verify correct finger, reject different finger, survive reboot | Not tested before root | Enrollment and 3 authentications confirmed; rejection/reboot pending |
| IR face unlock | Identify IR device/illumination; test actual unlock in dark conditions, not merely selfie-camera capture | Not tested before root | Face unlock works in normal light and dark room per owner; enabled/enrolled flags confirmed; direct IR instrumentation pending |
| Proximity | Near/far readings change; screen blanks and restores during a call test | Not tested before root | Repeated 0/5 near/far transitions captured; call behavior pending |
| Ambient light | Numeric readings respond to dark/bright conditions; auto-brightness responds | Not tested before root | Light response confirmed (0–422 lux example); automatic display response pending |

Collect build fingerprint, kernel version, sensor list, camera provider list,
fingerprint service state, and relevant errors. Distinguish raw sensor operation
from framework/biometric integration. Root then enables more detailed tracing
of the original vendor services and secure-app initialization sequence.

## Root plan

Use official Magisk release artifacts. The release checked at preparation was
v31.1; Android 7+ is supported. Patch the exact installed stock boot image ON
THIS PHONE using Magisk, retain both original and patched hashes, then flash
only the appropriate patched boot image. Do not use somebody else's patched
image or install unrelated modules. Grant diagnostic shell root explicitly
through the app's Superuser controls as required by this version.

References:
- https://topjohnwu.github.io/Magisk/install.html
- https://github.com/topjohnwu/Magisk/releases/tag/v31.1

## Execution record

- Partition inventory captured; eleven recovery/calibration backups verified.
- Owner authorized discarding existing phone files.
- System raw SHA256: `892ca619c155aed606eff08b022ced124dbc6e0f5d81d1eefa4231e4b39999de`.
- Vendor raw SHA256: `baf130881505eec0575276b3f233ae84861637d36a447d78b37214be1148ac76`.
- Prepare blank ext4 userdata/cache using the AOSP Android 10 mke2fs config,
  explicitly reserving 16384 bytes at the end of userdata for its FDE footer.
  Userdata has 14026743 blocks of 4096 bytes; cache has 65536 blocks.
- Planned writes: system, vendor, erase/format userdata, erase/format cache,
  stock boot last. Preserve GPT, calibration, matching firmware and unlock state.
- Verify sparse round trips and filesystems before entering Fastboot.
- Fastboot confirmed product `beryllium`, device unlocked and critical unlocked.
  Its size queries subtract 16384 bytes from every checked physical partition
  (including boot); physical sizes were measured directly under Linux. Preserve
  the complete stock system/vendor images, including their nonzero tail data.
- All four prepared filesystems passed `e2fsck -fn`; system/vendor sparse images
  round-trip byte-for-byte to the reconstructed originals. Full OTA transfer
  ranges are disjoint.
- All authorized writes completed successfully (`OKAY`): system, vendor,
  erase userdata, flash blank userdata filesystem, erase cache, flash blank
  cache filesystem, stock boot. Issued `fastboot -s <device-serial> reboot` successfully.
  Command transcripts are `.build/stock-android/flash-*.txt`, `erase-*.txt`,
  and `reboot-stock.txt`. Linux userdata has now been replaced.
- No bootloader relock or firmware/calibration writes were issued. Android
  first-boot completion and USB debugging authorization await owner interaction.
- Official Magisk v31.1 APK downloaded and verified against release metadata:
  `89cefac0a318afa39fb3880c45c87b036c05bc1ebf02a7a715b7f8d7c26ac6e7`.
  Root installation and sensor baseline tests remain pending.

### Stock Android boot confirmed; root preparation

ADB authorized by owner. Verified the expected MIUI build fingerprint and stock
kernel `4.9.186-perf-g10af704`; userdata reports encrypted, verified boot orange
(unlocked). Before-root framework dumps are saved privately under
`.build/stock-android/baseline-before-root/`.

Android lists ams TMD2725 proximity and ambient-light sensors and six camera
devices. The fingerprint service reports zero enrolled fingers and zero HAL
deaths. Enumeration alone does not establish face unlock or enrollment success.

The verified Magisk v31.1 APK and exact stock boot image were copied into the
phone's Download folder. MIUI rejected ADB installation with
`INSTALL_FAILED_USER_RESTRICTED`; owner was asked to install locally and patch
`poco-stock-boot.img` in Magisk on this phone. Patched boot flashing and root
verification are pending. No additional partition changes have been made.

### Magisk patched boot installed

Retrieved the owner's on-device Magisk v31.1 output
`magisk_patched-31100_bdYF7.img` (22811946 bytes). Phone and host SHA256 match:
`60668309c18a030da576e24a0cd906024a58a980126bf62fe44cbd57de6bc644`.
The Android boot header is valid and its kernel payload SHA256 matches stock
(`de397aa688f020f3d307b96fd4d20cc0033af52c5ab7539edf4fd261b4ff4dcb`).
Confirmed Fastboot product beryllium, flashed only boot successfully, then
rebooted. Transcript: `.build/stock-android/flash-magisk.txt`.
Root execution verification remains pending after reboot.

Android returned over ADB with `sys.boot_completed=1`. `/sbin/su -v` reports
`31.1:MAGISKSU`, confirming Magisk is present after boot. `su -c id` returned
Permission denied (exit 13): diagnostic Shell authorization is not yet granted.
Opened Magisk and asked the owner to complete any additional setup and authorize
Shell UID 2000 in Superuser. Do not report successful root-shell verification yet.

### Root verified

After owner authorized Shell, `adb shell su -c id` succeeded:
`uid=0(root) gid=0(root) groups=0(root) context=u:r:magisk:s0`.
Android reports boot completed and MagiskSU 31.1. Evidence is under
`.build/stock-android/baseline-after-root/`. Fingerprint HAL and Goodix service,
MIUI face service, and Qualcomm sensor services are running. Fingerprint service
reports zero enrolled fingers and zero HAL deaths at this snapshot. Root is
verified; actual biometric operation is not yet established. Owner asked to
enroll a finger and test lock-screen fingerprint unlock without sharing the PIN.

### Fingerprint enrollment and authentication confirmed

After the owner completed enrollment/unlock testing, Android's fingerprint
service reports one enrolled fingerprint, three accepted authentications,
zero rejected attempts, zero lockouts and zero HAL deaths. Evidence:
`.build/stock-android/baseline-after-root/fingerprint-enrolled.txt`.
This establishes successful enrollment and authentication on rooted stock MIUI.
Wrong-finger rejection and persistence across reboot remain untested. The stock
result supports a Linux integration problem rather than a nonfunctional sensor;
it does not yet identify the failing Linux Keymaster/Goodix interaction.

### Face-unlock menu region restriction confirmed

Phone region property is `AT` (Austria). The installed stock
`/system/etc/device_features/beryllium.xml` defines
`support_face_unlock_region_global`: Austria is absent; France (`FR`),
Italy (`IT`), India (`IN`) and others are present. Saved this primary evidence
in `.build/stock-android/face-investigation/beryllium.xml`. Asked owner to
select France in MIUI Region settings and retry the menu. No system XML edits
or biometric bypasses performed. Face enrollment remains pending.

### Face unlock enabled and owner reports success

After changing region to France, owner reports face unlock is working. ADB
confirms region `FR`; secure settings `face_unlcok_apply_for_lock`,
`face_unlock_has_feature`, and `face_unlock_valid_feature` are all 1. MIUI face
service is registered. No face templates or images were collected. Evidence:
`.build/stock-android/baseline-after-root/face-status.txt`.
This confirms enrollment/enabling state plus the owner's functional report;
a controlled dark-room IR test has not yet been reported or instrumented.
Fingerprint now reports one enrollment, six accepted and one rejected
attempt, no lockouts, and no HAL deaths. The rejected attempt alone does not
establish a controlled wrong-finger test.

### Dark-room face-unlock test

Owner explicitly reports that face unlock also succeeded in a dark room.
Together with confirmed enrollment/enabling flags, this establishes reported
low-light face-unlock operation on rooted stock MIUI. Direct camera/illuminator
instrumentation has not been performed, so this alone does not isolate IR
illumination from any light emitted by the screen. Proximity and ambient-light
response tests remain pending; framework enumeration identified TMD2725.

### Proximity response verified

Following the owner's cover/uncover test, TMD2725 proximity event history shows
repeated transitions between 0 (near) and 5 (far), at sensor timestamps
1285.849–1293.108 seconds. Saved full framework dump to
`.build/stock-android/sensor-tests/proximity-after-action.txt`.
The UI snapshot was back on the lock screen; no CIT pass-screen claim is made.
Physical proximity response is established; call-time screen blanking remains
untested. Opened CIT light-sensor test next; waiting for controlled light change.

### Ambient-light response verified

Following the owner's cover/bright-light test, captured TMD2725 ambient-light
readings change through 205, 351, 422, 379, 250, 110, 14, 1 and 0 lux at sensor
timestamps 1389.581–1390.024 seconds. Evidence:
`.build/stock-android/sensor-tests/light-after-action.txt`.
This establishes dynamic light-sensor response on rooted stock Android, not
calibrated lux accuracy or automatic screen-brightness behavior. Together with
recorded proximity transitions, both TMD2725 sensing functions respond.
No calibration writes were performed by the agent.

### Instrumented IR baseline — 2026-11-04

Camera 5 is the OV7251 IR module (stock sensor ID 0x7750). The LM3644
illuminator was observed switching on/off during the owner's face-unlock test;
stock PWM is 40 Hz at 10% duty cycle. A standalone NDK client subsequently
captured 146 frames at 640×480 in five seconds, exporting only luminance
statistics. Initial native-client timeouts were fixed by starting its Binder
callback pool; no provider-process restart was observed. Debug logging was
restored. Full wiring, evidence and Linux work are in
[kernel/ir/README.md](../kernel/ir/README.md).
