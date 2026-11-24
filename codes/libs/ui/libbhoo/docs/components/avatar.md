# Avatar

Status: Partial Rust avatar renderer with image, initials, fallback icon and three shapes.

| Field | Decision |
| --- | --- |
| Category | Data and application patterns |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

`Frame::avatar_at` draws a circle, 4 dp rounded square or square, using caller-provided RGBA image content, centered initials, or a bundled Material Symbols person fallback. Images use centered cover cropping and clip to the chosen shape. Background and foreground use Bhoo's secondary-container theme roles. Empty initials use the fallback icon.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

The caller owns loading and decoding images and chooses initials or the fallback icon if an image is loading or fails. `Image` construction validates dimensions and RGBA byte count; the renderer does not fetch remote URLs. The demonstration uses an original procedurally generated local image and simulates the host replacing a failed image with initials. The bounds remain fixed across content changes.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Content currently changes immediately within fixed bounds. Crossfade image arrival is a proposed improvement; there is no default looping animation.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Fail an image load and verify fallback content appears without changing adjacent layout.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Flutter CircleAvatar API and usage](https://api.flutter.dev/flutter/material/CircleAvatar-class.html)

No dedicated image was verified for this entry in the reference snapshot. The linked guides provide behavior and implementation references; a Bhoo visual specification remains to be produced.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Flutter material_ui | [circle_avatar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/circle_avatar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/circle_avatar.dart) |
| MUI | [Avatar.js](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Avatar/Avatar.js) | [Local checkout](../../../../context/material-ui/packages/mui-material/src/Avatar/Avatar.js), MIT license |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Avatar preview](http://127.0.0.1:8766/#/avatar).

Demonstrates: Image, centered initials and the person fallback in circular, rounded and square shapes; a switch simulates host image failure.

Validation: Rust avatar tests check centered initials, fallback icon, image cover crop, fixed bounds and each shape; the browser test exercises the focused example and image-error switch. Remaining scope: Host-specific image loading, optional image transitions, full theme token customization and device/framework visual parity. The fallback uses Material Symbols Outlined, while MUI's reference uses its own filled person icon.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
