# Missing-command suggestions

`configuration.nix` explicitly enables command-not-found and retains the pinned
Nixpkgs `programs.sqlite` as a standalone store file. It also exports
`NIX_SYSTEM=aarch64-linux`: the cross-built Perl interpreter's architecture
name does not match the database's Nix system identifiers.

The existing phone image had a handler pointing into an absent Nixpkgs source
path. Its compatibility fix is a database and patched handler under
`~/.local/share/command-not-found`, invoked by `command_not_found_handle` in
`~/.bashrc`. New terminals load this automatically; existing Bash terminals
can run `source ~/.bashrc`. Remove this user-local function after installing
a full image containing the declarative fix.

Verified on the phone: `htop` suggests `htop` and `htop-vim`; a fabricated command
returns exit 127 without a missing-database warning. No packages are installed
or executed automatically. The full image has not been rebuilt for this change.
