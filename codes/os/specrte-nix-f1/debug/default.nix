{ pkgs }:
pkgs.writeShellApplication {
  name = "spectre-phone-debug-agent";
  runtimeInputs = [ pkgs.python3 pkgs.grim pkgs.glib pkgs.systemd ];
  text = ''exec python3 ${./phone.py} "$@"'';
}
