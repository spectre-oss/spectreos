{ pkgs, ... }:
{
  # The vendor UCM files assumed card 0, but snd-aloop can occupy that slot.
  # Use the board's stable ALSA ID for both playback and capture endpoints.
  nixpkgs.overlays = [ (final: prev: {
    sdm845-alsa-ucm = final.runCommand "beryllium-alsa-ucm" {} ''
      mkdir -p $out/share
      cp -rL ${prev.sdm845-alsa-ucm}/share/alsa $out/share/alsa
      chmod -R u+w $out
      substituteInPlace $out/share/alsa/ucm2/Xiaomi/beryllium/{HiFi,VoiceCall}.conf \
        --replace-fail 'hw:0,' 'hw:F1,'
    '';
  }) ];

  services.pipewire.wireplumber.extraConfig."51-beryllium-audio" = {
    "monitor.alsa.rules" = [
      {
        # Keep the kernel loopback available for explicit users, but do not
        # offer it as the desktop's default speaker or microphone.
        matches = [ { "device.name" = "~alsa_card.platform-snd_aloop.*"; } ];
        actions.update-props."device.disabled" = true;
      }
      {
        matches = [ { "node.name" = "alsa_output.platform-sound.HiFi__Speaker__sink"; } ];
        actions.update-props = {
          "priority.session" = 1100;
          # S24_LE negotiated successfully but produced silence on this phone.
          # Match the QUAT_MI2S speaker backend; S16_LE is audibly verified.
          "audio.format" = "S16LE";
          "audio.rate" = 48000;
        };
      }
    ];
  };

  # The user verified FL is the quiet top earpiece and FR the bottom speaker.
  # Downmix applications to mono, then map that single channel to FR only.
  # Keep the original earpiece/call and headphone devices available.
  services.pipewire.extraConfig.pipewire."52-beryllium-speaker" = {
    "context.modules" = [ {
      name = "libpipewire-module-loopback";
      args = {
        "node.description" = "Phone Speaker";
        "capture.props" = {
          "node.name" = "spectre_bottom_speaker";
          "media.class" = "Audio/Sink";
          "audio.position" = [ "MONO" ];
          "priority.session" = 1200;
        };
        "playback.props" = {
          "node.name" = "spectre_bottom_speaker_output";
          "audio.position" = [ "FR" ];
          "target.object" = "alsa_output.platform-sound.HiFi__Speaker__sink";
          "stream.dont-remix" = true;
          "node.passive" = true;
        };
      };
    } ];
  };

  # CallAudio selects a physical sink when it connects to PulseAudio. Restore
  # the media default once after session startup, without policing later calls.
  systemd.user.services.spectre-media-output = {
    description = "Select the phone's mono media speaker after audio startup";
    wantedBy = [ "graphical-session.target" ];
    after = [ "pipewire.service" "pipewire-pulse.service" "wireplumber.service" ];
    serviceConfig.Type = "oneshot";
    path = [ pkgs.systemd pkgs.pipewire pkgs.wireplumber pkgs.python3 pkgs.coreutils ];
    script = ''
      busctl --user call org.freedesktop.DBus /org/freedesktop/DBus \
        org.freedesktop.DBus StartServiceByName su org.mobian_project.CallAudio 0
      sleep 3
      for attempt in $(seq 1 30); do
        sink=$(pw-dump | python3 -c 'import json,sys; print(next((n["id"] for n in json.load(sys.stdin) if n.get("info",{}).get("props",{}).get("node.name")=="spectre_bottom_speaker"),""))' )
        if [ -n "$sink" ]; then
          wpctl set-default "$sink"
          exit 0
        fi
        sleep 1
      done
      echo "Phone Speaker did not appear" >&2
      exit 1
    '';
  };
}
