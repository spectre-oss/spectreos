# Reference keyboard for Stevia

The English and Persian reference images are preserved unchanged under
`references/`. Normal-layout key faces, labels and footer artwork are rendered
from those images as a visual skin. Native interactive keys, press feedback,
shift/symbol layers, popups, input-method integration and emoji selection remain
functional. Persian text elsewhere in the OS uses Shabnam; the reference artwork
preserves the supplied glyph shapes exactly.

The skin has a 1080×777 canvas and scales with output width. Normal-layout key
bounds are measured from the artwork in `reference-geometry.json` and compiled
from `reference-geometry.h`. The measured skin geometry is also used for touch
selection; it is not an inert screenshot placed over another keyboard.
`tools/measure-keyboard-reference.py` reproduces the measurements without
modifying either reference image.

`stevia-widget.inc` integrates the renderer and adaptive scorer. The small
upstream patch is reproducible with `scripts/prepare-stevia-patch.py SOURCE`.
`modules/keyboard.nix` builds the modified Stevia, embeds the images and helper
code, and installs the layout/theme overlay. Its cached ARM64 runtime libraries
are linked with the cross compiler. See `typing/README.md` for the algorithm and
its limits, and `COPYING` for the Stevia license.

The existing phone uses user-local resource overlays and a persistent OSK service
override selecting the modified executable, protected by a GC root. Remove those
compatibility overrides when installing the declarative full image so future
Nix updates take effect. Image builds contain the same implementation.

Device validation and pixel comparisons are still in progress. A successful
build or mathematical test alone does not establish visual or typing correctness.

## Live installation — 0.12.1

The latest candidate is installed on the Poco F1 and selected by the persistent
`~/.config/systemd/user/mobi.phosh.OSK.service.d/50-spectre-layout.conf` override:
`/nix/store/sbw69r31mgcb47956clmjm0fhl7ab3l7-stevia-aarch64-unknown-linux-gnu-0.54.0`.
Its root is `/var/lib/spectre-stevia-0.12.1`. The service restart was verified
active with that exact executable; layout resources also load successfully.
The temporary debug override was removed. The keyboard appears when a text
field requests it; installation does not force it over unrelated applications.

The complete pixel comparison and live typing test remain pending exclusive
phone access. An earlier image's visible first 660 reference rows matched with
zero differing pixels, but that image clipped the footer. This build includes
the height correction; it must not be called fully verified until both complete
reference rectangles and the typing checks pass. A full image rebuild is still
separate from this live installation.
