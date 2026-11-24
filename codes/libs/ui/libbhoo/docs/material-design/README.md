# Material Design 3 Reference

Local reference documentation for Bhoo, copied from Google's Material Web repository. This snapshot includes its component documentation, theming documentation, illustrations, upstream README, and Apache 2.0 license. It is not a complete offline copy of the Material Design guidelines website or the Figma design kit.

## Start here

- [Bhoo component inventory](../components.md): component families researched across the official Material implementations, including gaps in this Web snapshot.
- [Introduction](upstream/docs/intro.md)
- [Component documentation](upstream/docs/components/)
- [Theming and design tokens](upstream/docs/theming/README.md)
- [Color](upstream/docs/theming/color.md)
- [Typography](upstream/docs/theming/typography.md)
- [Shape](upstream/docs/theming/shape.md)

For the launcher, start with [buttons](upstream/docs/components/button.md), [text fields](upstream/docs/components/text-field.md), [lists](upstream/docs/components/list.md), and [menus](upstream/docs/components/menu.md). Use these as reference material for Bhoo's own Rust components.

## Source and scope

| Field | Value |
| --- | --- |
| Repository | https://github.com/material-components/material-web |
| Snapshot commit | `d872455c4be56022da2095349b30ef46e0b55bc3` |
| Retrieved | 2026-11-08 |
| Copied paths | `docs/`, `README.md`, `LICENSE` |
| License | [Apache License 2.0](upstream/LICENSE) |

The upstream project describes itself as being in maintenance mode. This snapshot may differ from the latest Material 3 Expressive guidance. Consult the [official Material Design site](https://m3.material.io/) for current design specifications and the [component catalog](https://m3.material.io/components).

Upstream files are preserved without rewriting. Relative links to implementation files outside the copied documentation may require opening the [source repository at the pinned commit](https://github.com/material-components/material-web/tree/d872455c4be56022da2095349b30ef46e0b55bc3). JavaScript examples require the upstream web libraries; they do not run inside Bhoo.

## Updating the reference

The full reference repository was cloned to `context/material-web`, which is ignored by Git. Fetch and select the desired upstream revision there, replace the `upstream/docs` snapshot and the two root files above, then record the new commit and retrieval date in this document. Keep Bhoo-specific notes outside `upstream/` so a refresh preserves the distinction between our decisions and upstream guidance.

The upstream snapshot retains its own license and copyright notices. Bhoo's project license does not replace those notices.
