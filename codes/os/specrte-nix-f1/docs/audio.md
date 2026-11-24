# Poco F1 audio recovery

The installed 0.5.5 image initially selected the ALSA loopback device, while
the vendor UCM profiles hard-coded `hw:0`. The physical phone card is `F1`
and can appear as card 1. `modules/audio.nix` patches the pinned vendor UCM
profiles to use `hw:F1` for playback and capture. It also hides snd-aloop from
desktop audio selection and gives the HiFi speaker sink a higher priority.

`device.nix` includes `tas2559_uCDSP.bin` in early-boot firmware. Previously
the amplifier probed before that firmware was available and entered its
safeguard/reset path on first playback.

## Validation and remaining limits

The patched UCM derivation and initrd firmware derivation build successfully.
On the phone, the corrected profile exposes the physical speaker and selecting
it makes it the default. The speaker selection survived a WirePlumber restart.
The user confirmed an audible test tone after constraining the speaker sink
to S16LE at 48 kHz. Previously PipeWire negotiated S24_LE and ALSA reported
RUNNING, but the speaker was silent even unmuted at 100%. `modules/audio.nix`
now declares the tested format and rate. Successful stream submission alone
was not proof of working sound. Microphones, headphones, calls and resume also
need separate hardware tests.

Do not restart the live ADSP through remoteproc sysfs as a recovery technique.
That attempt hung this phone and lost USB SSH access. A temporary fastboot
boot of the known working image recovered Phosh without flashing partitions.

## Existing-image compatibility changes

The running phone has a patched UCM tree at
`~/.local/share/spectre-alsa/ucm2`, selected by persistent user-service drop-ins
for PipeWire, PipeWire Pulse and WirePlumber. Its WirePlumber rule is copied
from the evaluated Nix configuration into
`~/.config/wireplumber/wireplumber.conf.d/51-beryllium-audio.conf`.

New full images get the same UCM correction and rules from Nix. Once such an
image is installed, remove the compatibility UCM environment drop-ins and
user-local WirePlumber rule to let subsequent declarative updates take effect.
Early-boot firmware changes require an updated boot image; editing the root
filesystem does not update the installed initrd.

## Speaker route recovery on an existing installation

Testing the 0.10.2 boot image confirmed that amplifier firmware loads early.
A separate saved-state problem remained: WirePlumber restored the Earpiece
profile and a headphone default. Merely targeting the speaker node did not
activate its UCM route; the amplifier remained in configuration 3 (earpiece).
Explicitly selecting the speaker route changed it to configuration 4 and
powered both amplifier channels (`chl=3`). ALSA reported RUNNING during playback.
This establishes the software path, not audible output.

For manual recovery, inspect `wpctl status` and the card's `EnumProfile` and
`EnumRoute` parameters first. Numeric object IDs change on every restart.
On this pinned UCM, profile 1 is `HiFi (Headphones, Mic, Speaker)` and route 2
is `[Out] Speaker`. Set the profile on the card, then set the route on the
newly created speaker **node**, not the card:

```sh
pw-cli set-param CARD_ID Profile '{ index: 1, save: true }'
# Re-read wpctl status to obtain the speaker node ID.
wpctl set-route SPEAKER_NODE_ID 2
wpctl set-default SPEAKER_NODE_ID
wpctl set-volume SPEAKER_NODE_ID 0.35
wpctl set-mute SPEAKER_NODE_ID 0
```

Allow WirePlumber's delayed state save to complete before restarting it
(12 seconds was sufficient during testing). The profile, speaker route and
35% volume then survived a WirePlumber restart. This saved user state is
separate from the Nix UCM and priority rules; rebuilding the image alone does
not remove an existing user's saved earpiece selection. Do not force the
speaker route continuously, since calls and headphones need other routes.

## Bottom-speaker media output

The user identified the physical mapping: front-left feeds the quiet top
receiver; front-right feeds the bottom loudspeaker. Normal media now uses a
`Phone Speaker` virtual sink (`spectre_bottom_speaker`). PipeWire mixes the
application's stereo channels into MONO, and a loopback maps that one channel
to the physical FR port with remixing disabled. The FL port is not linked.
This avoids increasing the receiver's amplifier gain or dropping left-channel
content. The separate earpiece and headphone devices remain available.

The module is declared in `modules/audio.nix` using PipeWire's documented
[loopback channel mapping](https://pipewire.pages.freedesktop.org/pipewire/page_module_loopback.html).
The running older image has the identical evaluated configuration at
`~/.config/pipewire/pipewire.conf.d/52-beryllium-speaker.conf`. Remove that
compatibility file when installing a full image with the module, to avoid
loading two copies. The running user's default sink is saved to Phone Speaker.

Validation: Nix configuration evaluates; the live graph connects the virtual
MONO input to only physical FR, and hardware playback remains S16_LE/48 kHz.
A left-only 440 Hz tone and right-only 660 Hz tone were sent separately for
listening verification. Calls, headphone switching and suspend require their
own tests; this media routing change does not establish those as working.

The user confirmed both isolated channel tones came from the bottom speaker.
CallAudio reconnecting to PulseAudio was also observed to overwrite the media
default with headphones. `spectre-media-output.service` starts CallAudio via
D-Bus, allows its initial routing to settle, then selects the named mono sink
once at graphical-session startup. The same service is enabled on the phone.
It does not continuously force routing during calls or headphone use. Manual
PipeWire restarts may require restarting this service too. The startup
sequence was tested by restarting audio and the selection service; a full
phone reboot has not yet been tested. Remove the user-local service and script
when migrating to the declarative full image.
