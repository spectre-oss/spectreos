# Persian font

`modules/fonts.nix` includes Shabnam 5.0.1 in the system font packages. Its
Fontconfig rule prepends Shabnam when the requested language contains `fa`,
without changing the default Latin UI font. The Persian keyboard letter keys
also explicitly select Shabnam in their stylesheet.

The font is a pinned Nixpkgs package, not an untracked download or a phone-only
change. Local Fontconfig validation returns `Shabnam` for `sans-serif:lang=fa`.
Installed on the existing phone through the pinned font package and a persistent
user Fontconfig file at `~/.config/fontconfig/conf.d/99-spectre-persian.conf`.
The phone also returns `Shabnam` for `sans-serif:lang=fa`. The updated keyboard
executable and Persian stylesheet are installed; a Persian screenshot comparison
is still pending. The full image has not been rebuilt yet.

The compatibility package is protected by a GC root under `/var/lib`. Remove
the user Fontconfig override after installing the declarative image so future
system font settings can take effect.
