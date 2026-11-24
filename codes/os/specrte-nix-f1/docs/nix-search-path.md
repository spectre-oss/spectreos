# Nixpkgs search path

The old image set `NIX_PATH` to a root channel that was never installed.
Consequently `nix-shell -p ...` could not import `<nixpkgs>`.

`modules/nix-search-path.nix` retains the project's pinned Nixpkgs source in
the system closure and defines `nixpkgs=...`. It also defines `nixpkg=...` as
an alias for the singular spelling reported by the user. This requires no
channel updates and uses the same package recipes as the image. It does not
change the project's own cross-build package set.

Both imports were tested locally for `aarch64-linux`; `htop` evaluates
successfully. A shell can still need network access to download a requested
package and its dependencies. Search-path evaluation alone does not install it.
The full source increases image storage usage; keeping it is intentional so
these interactive Nix commands work without a missing channel or source.

The phone now has the matching pinned source and a persistent shell override
at `~/.config/spectre/nix-path.sh`, sourced by `~/.bashrc` and `~/.profile`.
New terminals use it automatically; existing Bash terminals can run
`source ~/.bashrc`. Both imports resolve correctly on the phone, and the actual
`nix-shell -p htop --run "htop --version"` test returned `htop 3.5.1`.

The compatibility source is protected by a GC root under `/var/lib`. A rebuilt
image uses the declarative system search path automatically; remove the local
shell override after migrating to it.
