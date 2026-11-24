# Files

SpectreOS Files browses local files, XDG user folders, removable mounts, and Trash. It supports
recursive search, media and document filters, list and grid layouts, image previews, file details,
and sorting by name, modification date, size, or type. Use the item menu to select files, then
rename, copy, move, share file links, compress, extract ZIP, or move items to Trash. Trash supports
restoring items and permanent deletion with confirmation.

Wisp embeds the Files library and uses its image chooser in Wallpaper & style. Wayland apps and
scripts can use the separate executable to choose existing files, choose a directory, or save a file.

```sh
cargo build --locked --release --features wayland --manifest-path codes/apps/files/Cargo.toml
spectre-files
spectre-files --open --images --multiple
spectre-files --folder --start ~/Documents
spectre-files --save report.txt
```

Install the executable on `PATH` and `assets/org.spectreos.Files.desktop` in the XDG applications directory
for standalone use. **New window** opens the current folder in a separate window when the executable
is installed. The chooser prints percent-encoded file URIs, one per line. Cancel returns status 1.
Saving to an existing file requires confirmation. The caller writes the contents after receiving the URI.

The embedded chooser supports native SpectreOS callers and the executable supports Wayland clients.
Existing desktop portals remain available to applications that request their toolkit’s system dialog.
