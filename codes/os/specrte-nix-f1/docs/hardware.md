# Poco F1 hardware and Linux support evidence

Research date: **18 October 2026**. Device: Xiaomi POCOPHONE F1 / POCO F1,
Linux device name **xiaomi-beryllium**. This document distinguishes the phone's
hardware, upstream support evidence, the current project's configuration, and
actual device testing.

**Runtime evidence is being added as the phone is tested.** Wi-Fi initialization
and scanning were verified on 19 October 2026; see the Wi-Fi section. Other
subsystem entries retain their original verification requirements. The device definition
intentionally labels this port experimental/unsupported. A driver selected in
configuration, or an enabled device-tree node, is not a working-hardware claim.

## Hardware inventory

Xiaomi's UK specifications advertise Snapdragon 845, Adreno 630, 6 GB LPDDR4X,
64/128 GB UFS 2.1, a 6.18-inch 2246×1080 display, and a 4000 mAh typical battery
with Quick Charge 3.0. Connectivity includes dual-band 802.11ac Wi-Fi with 2×2
MIMO, Bluetooth 5, GPS/GLONASS/BeiDou, a hybrid dual-nano-SIM/microSD tray,
USB-C, and a 3.5 mm headset jack. Cameras are 12+5 MP rear and 20 MP front.
A rear fingerprint reader, infrared face unlock, accelerometer, gyroscope,
compass, proximity, ambient-light and Hall sensors, and vibration are listed.
These are marketed capabilities, not Linux validation. The UK page does not
establish every regional memory/storage variant. [Xiaomi specifications][xiaomi]

The Snapdragon 845 platform uses an eight-core, 64-bit Kryo 385 CPU, Adreno 630,
X20 LTE modem, and Spectra 280 imaging processor. Platform capabilities alone do
not prove a phone implementation or driver works. [Qualcomm platform][qualcomm]

Local records identify this particular phone as **6 GB RAM with a Tianma
NT36672A panel**. The project sets 6 GB and 1080×2246 accordingly. Other Poco F1
units can use an EBBG display: a model match alone is insufficient to select the
correct panel. [Local device records][local-readme], [project device definition][device]

## Subsystems

Unless runtime evidence is explicitly recorded below, a subsystem is unverified. “Configured” means
a device-tree, kernel or userspace path was found, not successful operation.
Driver selections refer to the generated [kernel build configuration][built-config].
Hardware wiring comes from the pinned local board sources indexed below;
project settings are in [device.nix][device] and [configuration.nix][configuration].
Unknown components and incomplete integrations are stated explicitly.

### CPU and platform

**Purpose:** Runs the kernel, applications and background services.

**Hardware:** Qualcomm SDM845 with eight ARM64 Kryo 385 CPU cores.

**Current driver:** ARM64 platform support, Qualcomm clocks and pin control; `CONFIG_ARM_QCOM_CPUFREQ_HW=y`, `CONFIG_ARM_QCOM_CPUFREQ_NVMEM=y` and `CONFIG_PINCTRL_SDM845=y`.

**Configuration and limitations:** The AArch64 kernel builds. CPU-frequency governors and idle support are compiled in; their presence does not establish the runtime governor or working low-power states.

**Verification needed:** Check all cores, frequency changes, sustained load, temperature and stability.

### Memory

**Purpose:** Holds running applications and kernel data.

**Hardware:** Local phone records and the device definition identify 6 GB RAM; the physical memory chip vendor is not established.

**Current driver:** Linux memory management on ARM64; this does not require a separate phone-specific application driver.

**Configuration and limitations:** The project configures zram with zstd at 25% of RAM. This is compressed swap, not additional physical RAM. Build-host memory limits are separate from phone memory handling.

**Verification needed:** Check detected usable RAM, memory pressure, zram activity and application recovery under load.

### Graphics

**Purpose:** Renders the desktop and accelerates compatible applications.

**Hardware:** Adreno 630 GPU.

**Current driver:** DRM MSM/DPU in Linux (`CONFIG_DRM_MSM=y`); Mesa Freedreno and Turnip provide the relevant userspace driver families.

**Configuration and limitations:** A630 firmware is included. Phosh selects the desktop environment; successful kernel compilation does not prove accelerated rendering. See [Mesa documentation][mesa] for driver-family coverage.

**Verification needed:** Check renderer identification, acceleration, visual corruption and compositor stability, including after resume.

### Tianma display

**Purpose:** Displays the desktop and controls screen output.

**Hardware:** Tianma panel with Novatek NT36672A controller, recorded for this phone.

**Current driver:** `CONFIG_DRM_PANEL_NOVATEK_NT36672A=y`, Qualcomm DSI/DPU, and `CONFIG_BACKLIGHT_QCOM_WLED=y`.

**Configuration and limitations:** The Tianma DTB is the default and was inspected in the built boot image. The project uses 1080×2246 screen geometry.

**Verification needed:** Check first output, orientation, brightness range, blanking and suspend/resume.

### Tianma touchscreen

**Purpose:** Reports finger positions to the interface.

**Hardware:** Novatek touchscreen, declared as `novatek,nt36672a-ts` in the Tianma device tree.

**Current driver:** Novatek NVT touchscreen driver (`CONFIG_TOUCHSCREEN_NOVATEK_NVT_TS=y`).

**Configuration and limitations:** The touchscreen node is present in the selected variant. Driver selection alone does not establish coordinates, interrupts or wake behaviour.

**Verification needed:** Check multitouch, edges, orientation, accuracy and behaviour after blanking and resume.

### EBBG display and touchscreen variant

**Purpose:** Provides the screen and touch input on the alternate phone variant.

**Hardware:** EBBG FT8719 panel with FocalTech touch controller.

**Current driver:** `CONFIG_DRM_PANEL_EBBG_FT8719=y` and `CONFIG_TOUCHSCREEN_EDT_FT5X06=y`, with separate EBBG device-tree wiring.

**Configuration and limitations:** An alternate DTB is available. Local records identify this particular phone as Tianma, so the EBBG configuration needs its own build selection and device validation.

**Verification needed:** Confirm the actual panel before selecting an image; test display, brightness and touch independently on an EBBG phone.

### Internal storage

**Purpose:** Stores the installed operating system and persistent data.

**Hardware:** UFS storage connected to the Qualcomm UFS controller; this phone’s actual capacity has not been read from hardware.

**Current driver:** `CONFIG_SCSI_UFSHCD=y`, `CONFIG_SCSI_UFS_QCOM=y` and the Qualcomm UFS PHY.

**Configuration and limitations:** The board enables UFS. Firmware flashing must respect existing partition boundaries; a marketed capacity is not a measured partition size.

**Verification needed:** Check detected partitions, read/write integrity, sustained I/O, reboot and recovery after interrupted operations.

### microSD storage

**Purpose:** Provides removable storage through the hybrid tray.

**Hardware:** External SD slot on SDHC2.

**Current driver:** Qualcomm SDHCI MSM (`CONFIG_MMC_SDHCI_MSM=y`).

**Configuration and limitations:** The external slot is enabled in the board tree. Desktop mounting and filesystem compatibility still require runtime checks.

**Verification needed:** Check card detection, removal, mounting and sustained reads/writes with a known-good card.

### Root filesystem and first-boot expansion

**Purpose:** Makes the installed root filesystem use the existing userdata partition capacity.

**Hardware:** An ext4 filesystem image flashed into the phone’s userdata partition.

**Current driver:** Linux ext4 (`CONFIG_EXT4_FS=y`) plus the NixOS boot/filesystem resizing configuration.

**Configuration and limitations:** Root `autoResize` is enabled and partition-table growth is explicitly disabled. Expansion belongs to system boot, so Phosh needs no resizing modification. This configuration has not yet been exercised on the phone.

**Verification needed:** Compare filesystem and partition sizes after first boot, check free space, and repeat boots to establish safe, repeatable expansion.

### Wi-Fi

**Purpose:** Connects to wireless networks.

**Hardware:** Qualcomm WCN3990 hw1.0 with phone-specific calibration data.

**Current driver:** ath10k SNOC (`CONFIG_ATH10K_SNOC=y`).

**Configuration and limitations:** The vendor archive supplies `wlanmdsp.mbn`
and the phone-specific `board-2.bin`, but lacks ath10k's API metadata file,
`ath10k/WCN3990/hw1.0/firmware-5.bin`. The firmware derivation now copies that
file from pinned linux-firmware without replacing board calibration. The
initrd includes the ath10k directory because this built-in driver probes early.

**Runtime evidence (19 October 2026):** Before repair, ath10k reported missing
`firmware-N.bin` and NetworkManager had no Wi-Fi hardware. Installing the API
file and rebinding `18800000.wifi` created `wlan0`; NetworkManager successfully
listed nearby access points. The live file is in `/lib/firmware`; a fresh boot
of the updated image has not yet been tested. The kernel reported a random
MAC address and EEPROM country CN; regulatory behaviour remains unverified.

**Verification needed:** Check association, credentials, throughput, regulatory domain, reconnection, hotspot operation and resume.

### Bluetooth

**Purpose:** Connects peripherals and compatible wireless audio devices.

**Hardware:** Qualcomm WCN3990 attached over UART.

**Current driver:** Qualcomm HCI UART (`CONFIG_BT_HCIUART_QCA=y`).

**Configuration and limitations:** QCA firmware is supplied. The controller path is present, but controller startup and application-level profiles need device testing.

**Verification needed:** Check pairing, reconnect, input devices, audio routing and suspend/resume.

### Cellular modem and SIM

**Purpose:** Provides mobile-network registration, calls and packet data.

**Hardware:** SDM845 modem and SIM interfaces.

**Current driver:** Qualcomm remote-processor drivers, QRTR/QMI transport and IPA (`CONFIG_QCOM_IPA=y`, `CONFIG_QRTR=y`).

**Configuration and limitations:** Modem firmware and Mobile NixOS services including rmtfs, qrtr-ns, tqftpserv and userspace pd-mapper are configured. Calls is enabled through the Phosh example. Firmware inclusion does not prove modem startup, voice routing or either SIM slot.

**Verification needed:** Check modem boot, registration, PIN handling, SIM selection, calls, data, reconnection and suspend reliability.

### SMS and messaging interface

**Purpose:** Exposes modem text messaging through an application.

**Hardware:** Uses the cellular modem; there is no separate SMS hardware controller.

**Current driver:** The modem/QMI stack supplies the hardware path; a messaging application supplies the interface.

**Configuration and limitations:** Chatty is commented out in the imported Phosh example. A complete messaging experience is therefore not established by the current configuration.

**Verification needed:** Select and configure a suitable client, then test sending, receiving, notifications and persistence after reboot.

### GNSS positioning

**Purpose:** Supplies location fixes to applications.

**Hardware:** Modem/DSP-assisted positioning hardware; see the advertised capabilities in the inventory.

**Current driver:** A complete project-specific positioning driver/service chain has not been established from the inspected files.

**Configuration and limitations:** Do not infer working location from modem support. No verified application-facing GNSS service or position fix is recorded for this image.

**Verification needed:** Establish the required service and firmware path, then measure cold/warm fixes, accuracy, supported interfaces and power use.

### Audio codec, microphones and headset

**Purpose:** Routes sound between applications, microphones and the headset jack.

**Hardware:** WCD9340 audio codec and board audio wiring.

**Current driver:** ALSA ASoC SDM845 (`CONFIG_SND_SOC_SDM845=y`) and WCD934x (`CONFIG_SND_SOC_WCD934X=y`).

**Configuration and limitations:** The SDM845 family imports ALSA UCM routing profiles. The kernel PD mapper is deliberately disabled because of a reported microphone race; userspace pd-mapper remains configured.

**Verification needed:** Test microphone recording, headphones, jack detection, volume, call routing and operation after suspend.

### Loudspeaker amplifiers

**Purpose:** Drives the phone’s speaker outputs.

**Hardware:** TAS2559/TAS2560 amplifier wiring in the pinned board tree.

**Current driver:** The pinned tree builds the TAS2559 driver (`CONFIG_SND_SOC_TAS2559=y`).

**Configuration and limitations:** Board declarations and a selected driver do not establish that amplifier firmware, tuning and audio routing are complete or correct.

**Verification needed:** Verify required firmware and profiles, then test playback, channel routing, distortion and safe volume levels.

### Main rear camera and focus

**Purpose:** Captures images and controls lens focus.

**Hardware:** Sony IMX363 sensor and ROHM BU64748 focus actuator in the pinned device tree.

**Current driver:** Qualcomm CAMSS, IMX363 and BU64748 (`CONFIG_VIDEO_QCOM_CAMSS=y`, `CONFIG_VIDEO_IMX363=y`, `CONFIG_VIDEO_BU64748=y`).

**Configuration and limitations:** Megapixels is selected by the Phosh example. The sensor, media pipeline and application configuration must work together; downstream device patches must be preserved during kernel updates.

**Verification needed:** Inspect media links, capture still images, test focus, orientation, exposure and repeated camera startup.

### Other cameras and infrared face unlock

**Purpose:** Provides additional camera views and the advertised face-unlock function.

**Hardware:** Secondary rear, front-facing and infrared hardware advertised for the phone.

**Current driver:** No functioning driver and complete media pipeline for these components has been established in the inspected project evidence.

**Configuration and limitations:** Do not claim these cameras or biometric face authentication as supported. Android feature availability does not establish a Linux implementation.

**Verification needed:** Identify actual component variants and device-tree support before implementing and testing capture or authentication.

### Camera flash

**Purpose:** Illuminates a camera scene.

**Hardware:** PMIC-controlled flash hardware declared by the board tree.

**Current driver:** The relevant Qualcomm flash driver exists, but `CONFIG_LEDS_QCOM_FLASH` is disabled in the generated kernel.

**Configuration and limitations:** Flash is not enabled in this build. A declared device-tree node is insufficient without its driver and application integration.

**Verification needed:** Enable and review the required path before testing current limits, torch mode and synchronized camera flash.

### Hardware video codecs

**Purpose:** Accelerates compatible video decoding and encoding.

**Hardware:** Qualcomm Venus video processor.

**Current driver:** Venus V4L2 (`CONFIG_VIDEO_QCOM_VENUS=y`).

**Configuration and limitations:** The board node and firmware path are present. FFmpeg is included in the build dependency stack, but hardware codec use by applications has not been demonstrated.

**Verification needed:** Check exposed V4L2 devices, supported formats, actual hardware use, playback/encoding stability and power consumption.

### Motion and environmental sensors

**Purpose:** Supplies orientation and other physical measurements to the interface.

**Hardware:** SLPI sensor DSP and the phone’s motion/environmental sensor hardware; individual chip models are not established here.

**Current driver:** Qualcomm remote-processor infrastructure and Linux IIO (`CONFIG_IIO=y`); Phosh enables IIO sensor support.

**Configuration and limitations:** The first phone boot of 0.5.5 showed a repeated SLPI FastRPC probe failure (`qcom_scm` memory assignment `-22`). The recovery kernel disables SLPI to remove this failing subsystem from boot. SLPI-backed sensors are unavailable; this is a temporary recovery workaround and successful boot still requires device testing. See [boot recovery](boot-recovery.md).

**Verification needed:** Identify and test accelerometer, gyroscope, compass, proximity, light and Hall events separately; verify rotation and call-time screen blanking.

### Fingerprint reader

**Purpose:** Provides fingerprint-based authentication when a complete stack exists.

**Hardware:** Rear fingerprint reader; its specific component variant is not established in the inspected evidence.

**Current driver:** No fingerprint device-tree node and working authentication integration have been established for this project.

**Configuration and limitations:** The reader is an advertised hardware feature, not a supported authentication method in this image.

**Verification needed:** Identify the component, driver and userspace requirements before testing enrollment, matching and authentication policy.

### Battery and charging

**Purpose:** Measures battery state and controls charging.

**Hardware:** PMI8998 charger and fuel gauge with the board’s battery description.

**Current driver:** Qualcomm SMB2 charger (`CONFIG_CHARGER_QCOM_SMB2=y`) and PMI8998 fuel gauge (`CONFIG_BATTERY_PMI8998_FG=y`).

**Configuration and limitations:** The corresponding board nodes are enabled. Advertised fast charging does not prove that charging mode is implemented safely by this Linux configuration.

**Verification needed:** Check charge detection, capacity/state reporting, current, temperature, full-charge behaviour, unplugging and low-battery shutdown.

### Haptics and notification LEDs

**Purpose:** Provides vibration and visual status feedback.

**Hardware:** PMI8998 haptics and board status-LED wiring.

**Current driver:** `CONFIG_INPUT_QCOM_SPMI_HAPTICS=y` and Qualcomm LPG (`CONFIG_LEDS_QCOM_LPG=y`).

**Configuration and limitations:** The family configuration exposes vibration to feedbackd through udev permissions. Camera flash is a separate subsystem with a disabled driver.

**Verification needed:** Check vibration strength, notifications, LED colours/brightness and feedback after resume.

### USB device mode

**Purpose:** Lets the phone act as a USB peripheral for a computer.

**Hardware:** USB-C connector, Qualcomm DWC3 controller and USB PHYs.

**Current driver:** DWC3 Qualcomm (`CONFIG_USB_DWC3_QCOM=y`) with QMP USB and QUSB2 PHY support.

**Configuration and limitations:** The board sets peripheral mode; the Mobile NixOS family selects gadget functions including ADB, mass storage and RNDIS. Fastboot is a separate bootloader interface, not proof the Linux USB gadget works.

**Verification needed:** Test enumeration, USB networking and selected gadget functions. Validate the laptop installer and bootloader flashing separately.

### USB host and OTG

**Purpose:** Would let the phone control USB peripherals.

**Hardware:** Shares the USB-C controller and physical connector with peripheral mode.

**Current driver:** The kernel has `CONFIG_USB_DWC3_DUAL_ROLE=y`, but the current board tree selects peripheral mode.

**Configuration and limitations:** Kernel capability alone does not configure board role switching or host power. Host/OTG operation is not configured or supported by the current evidence.

**Verification needed:** Investigate role switching, VBUS supply and board requirements before testing powered peripherals.

### Power management, thermals and suspend

**Purpose:** Controls supply rails, heat, idle states and wake/resume.

**Hardware:** PM8998/PMI8998 power management, platform regulators, thermal sensors and remote processors.

**Current driver:** Qualcomm RPMh/SPMI regulator support, TSENS (`CONFIG_QCOM_TSENS=y`), thermal management and PSCI CPU idle.

**Configuration and limitations:** Driver selections are present, but complete system suspend depends on every active subsystem and its firmware. No measured battery-life or dependable-suspend claim is available.

**Verification needed:** Measure idle drain and temperatures; test repeated deep suspend/resume, wake sources, modem connectivity and shutdown.

## Upstream support and caveats

Mesa documents Freedreno OpenGL/OpenGL ES support for the Adreno 6xx family and
Turnip Vulkan support for Adreno 6xx. This supports the chosen graphics stack;
it does not validate acceleration in our image. [Mesa driver documentation][mesa]

The live upstream Linux board sources contain common beryllium and separate
Tianma/EBBG device trees. The project's kernel is an SDM845-mainline fork with
additional device patches: “mainlined phone” must not be read as “every feature
works on an unpatched upstream release.” Compare against the exact pinned tree
when updating; live Linux master is a moving reference. [Upstream common tree][linux-common],
[Tianma][linux-tianma], [EBBG][linux-ebbg], [pinned board tree][pinned-common]

Historical postmarketOS reports describe working display, touch, graphics,
audio, Bluetooth and USB networking, with partial battery, Wi-Fi, camera, GPS
and cellular functionality, and broken OTG. Notes mention modem availability,
Wi-Fi regulatory restrictions, charging limitations, sensor userspace, and
camera application setup. The accessible search-index/cache evidence is dated;
the live page redirected during research. Treat these as historical reports
about another distribution, not a current validation matrix for this image.
[postmarketOS device page][pmos], [local cached copy][local-wiki]

The pinned SDM845 kernel configuration deliberately disables the in-kernel
Qualcomm PD mapper because of a microphone race reported even with the pinned
release candidate. The project retains the userspace mapper. Do not “enable
all drivers” during upgrades without checking device-specific regressions.
[Pinned configuration][pinned-config], [local kernel recipe][kernel-recipe]

Older Linaro testing demonstrated mainline-oriented Poco F1 display, touch,
Wi-Fi, Bluetooth, USB gadget, audio and SD-card functionality using a tracking
tree with extra patches. It is historical context, not proof that this newer
kernel or Mobile NixOS image works on the phone. [Linaro presentation, pages 11–13][linaro]

## Kernel update requirement

The OS must track the latest upstream Linux, including newer release candidates
for the requested cutting-edge track, with prereleases clearly labelled.
Source pins/checksums, the remaining device patch stack, device trees, firmware,
and regression evidence must be updated together. New releases must explicitly
record failed or untested subsystems rather than inheriting a “supported” label.
The detailed acceptance criteria are in [requirements.md](requirements.md#cutting-edge-kernel-updates).

At the time this document was prepared, the project pin is **Linux 7.1.0-rc1**
from the SDM845-mainline tree. This is a recorded pin, **not a claim that it is
the newest available release on the research date**. Documentation work does
not itself update that pin. [Project source pins][pins]

## Local evidence index

| Evidence | What it establishes |
| --- | --- |
| [Project device definition][device] | Experimental support level, 6 GB / screen geometry, firmware selection, non-A/B boot layout, userdata destination and panel DTB. |
| [Project configuration][configuration] | Phosh, filesystem auto-resize, storage limits and current package fixes. |
| [Kernel recipe][kernel-recipe] and [configuration fragment](../kernel/sdm845.config) | Monolithic kernel configuration and device-specific microphone workaround. |
| Generated [kernel build config][built-config] | Driver selections in the actual built kernel; flash driver disabled. Generated output is ignored by Git and must be rebuilt in a fresh checkout. |
| Generated [boot inspection report][boot-report] | Offline validation of ARM64 kernel, Tianma DTB, image layout and fifteen required built-in drivers. This does not prove the phone boots. |
| [Firmware recipe](../firmware/default.nix) and [checksums](../firmware/SHA256SUMS) | Phone archive plus selected A630/QCA firmware. Proprietary payloads remain outside Git. |
| Fetched Mobile NixOS Phosh example | Enables Calls, Megapixels and IIO sensor support; Chatty line is commented out. Inspect `mobile-nixos/examples/phosh/phosh.nix` after fetching sources. |
| Fetched SDM845 family modules | ALSA UCM profiles, rmtfs and userspace PD mapper. Inspect the family and Qualcomm modem modules after fetching. |
| [Workspace phone records][local-readme] / [cached wiki][local-wiki] | Earlier recorded identity and historical porting notes; these files live outside this standalone repository. |

The kernel archive is intentionally ignored by Git. After `make fetch`, inspect
these files inside `kernel/linux.tar.gz`, under the pinned revision's
`arch/arm64/boot/dts/qcom/` directory:

- `sdm845-xiaomi-beryllium-common.dtsi`: battery at approximately line 141;
  rear camera at 330; amplifiers at 487; charger/fuel gauge at 529; haptics at
  564; external SD at 619; SLPI at 632; Bluetooth at 825; UFS at 846; USB at 866;
  Venus at 890; codec at 895; Wi-Fi at 908.
- `sdm845-xiaomi-beryllium-tianma.dts`: variant panel/touch wiring near line 12.
- `sdm845-xiaomi-beryllium-ebbg.dts`: alternate panel/touch wiring near line 12.

Line locations refer to the current pin and will change on upgrades. In the
current generated kernel config, `CONFIG_LEDS_QCOM_FLASH` is disabled near line
5687. Never substitute a device-tree declaration for a driver/runtime check.

## Hardware acceptance evidence to collect

For each subsystem, record the exact project revision, upstream kernel version,
firmware revision, panel variant, test steps, observed result, relevant logs,
and regressions relative to the prior release. Test cold boot, at least one
reboot, and suspend/resume where relevant. Record failures and unsupported
functions explicitly. Installable images remain experimental until device tests
establish otherwise; no actual phone flashing has been performed in this port.

[xiaomi]: https://www.mi.com/uk/poco-f1/specs/
[qualcomm]: https://www.qualcomm.com/smartphones/products/8-series/snapdragon-845-mobile-platform
[mesa]: https://docs.mesa3d.org/drivers/freedreno.html
[linux-common]: https://raw.githubusercontent.com/torvalds/linux/master/arch/arm64/boot/dts/qcom/sdm845-xiaomi-beryllium-common.dtsi
[linux-tianma]: https://raw.githubusercontent.com/torvalds/linux/master/arch/arm64/boot/dts/qcom/sdm845-xiaomi-beryllium-tianma.dts
[linux-ebbg]: https://raw.githubusercontent.com/torvalds/linux/master/arch/arm64/boot/dts/qcom/sdm845-xiaomi-beryllium-ebbg.dts
[pinned-common]: https://gitlab.com/sdm845-mainline/linux/-/blob/85f1df2a4ec71d7a91dd95a7a49f889d1595ffa8/arch/arm64/boot/dts/qcom/sdm845-xiaomi-beryllium-common.dtsi
[pinned-config]: https://gitlab.com/sdm845-mainline/linux/-/blob/85f1df2a4ec71d7a91dd95a7a49f889d1595ffa8/sdm845.config
[pmos]: https://wiki.postmarketos.org/wiki/Xiaomi_POCO_F1_%28xiaomi-beryllium%29
[linaro]: https://static.linaro.org/connect/lvc21/presentations/lvc21-101.pdf
[local-readme]: ../../../pocof1/README.md
[local-wiki]: ../../../pocof1/wiki_postmarketos_beryllium.txt
[device]: ../device.nix
[configuration]: ../configuration.nix
[kernel-recipe]: ../kernel/default.nix
[built-config]: ../result-kernel/build.config
[boot-report]: ../.build/boot-inspection.json
[pins]: ../sources.json
