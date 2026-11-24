# Android XML RelativeLayout

Status: Initial rule-based layout support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework layout |
| Priority | P1 |

## Scope and variants

RelativeLayout positions children relative to parent edges or sibling views. Supported rules include left/right/start/end of, above/below, aligning sibling edges, aligning parent edges, centering, parent gravity for a child group, first-line baseline alignment, and align-with-parent-if-missing. Individual child center rules use the full parent bounds, including asymmetric padding. Opposing edge constraints size the child to the resulting space. If a single anchor leaves negative available space, fixed and wrap-content children retain their measured size, as in Android. Forward references, margins, padding, RTL start/end resolution, gone-anchor chains and basic wrap-content measurement are supported.

## State and interaction contract

Children retain their normal interaction and state. Invisible children keep their space; gone anchors are followed through the corresponding rule. The XML importer rejects circular dependencies within an axis. Dynamic visibility can change placement. This is an initial implementation: complete Android measurement, some nested and wrap-content corner cases remain open. Baseline alignment covers TextView, CheckedTextView, Chronometer, TextClock, Button and ToggleButton using renderer-supplied font metrics; other views have no baseline. Explicit layoutDirection is applied during measurement and drawing. Parent gravity includes child margins and resolves end in RTL; ignoreGravity supports direct children. Gravity on wrap-content axes and ignored descendants remain incomplete. Full Android font and measurement parity remains open. Unknown rules are rejected.

## Motion and animation

RelativeLayout places children directly. Children provide their own interaction motion.

**Reduced motion:** Placement remains direct.

## Acceptance criteria

- Resolve forward sibling references and parent alignment with margins in LTR and RTL.
- Follow gone anchors and support missing-anchor fallback.
- Compare platform measurement, baseline and wrap-content behavior with Android on a device.

## References

- [Android RelativeLayout reference](https://developer.android.com/reference/android/widget/RelativeLayout)
- [Android 15 RelativeLayout source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/RelativeLayout.java)

## Wrap-content child measurement

RelativeLayout now measures the horizontal wrap-content size of a child with the active Bhoo text metrics and current child state before resolving dependent sibling positions. This includes the supported Button, ToggleButton, Switch, CheckBox, RadioButton and TextView measurement paths. Parent constraints still limit the result. A dedicated example changes a button label and keeps its end-anchored sibling 8 dp away in both layout directions. Rust tests cover the changed child width and sibling position; Android device and full theme measurement parity remain unverified. The Apache-2.0 Android 15 reference above is stored locally at `context/reference-sources/android-framework/RelativeLayout.java`.

## Nested wrap-content width

A wrap-content RelativeLayout resolves its child positions using the active renderer text metrics and current state before reporting its own width. A focused nested example checks the resized child, its anchored sibling and the next outer sibling in LTR and RTL. Rust checks include non-default text metrics.

### RTL wrap-content width fix (Wave 1B)

- **Cause and fix:** During horizontal wrap-content measurement (`wrap_parent == true`), `codes/libs/ui/libbhoo/src/relative_layout.rs` previously deferred physical parent-right anchors in both directions and measured child positions from the left edge, which caused `layout_toEndOf` siblings inside an RTL wrap-content `RelativeLayout` to collapse to `0` width. `Axis::position` now defers the container's logical end edge (`allow_parent_low = !self.wrap_parent || !rtl_h`, `allow_parent_high = !self.wrap_parent || rtl_h`), anchors unconstrained RTL children to `origin + extent - after - size`, and `layout_intrinsic` / `intrinsic` in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` measure RTL horizontal wrap extent from the container's right edge `(inner.x + inner.width) - (r.x - m[0])`.
- **Validation evidence:** Rust regression test `relative_layout_rtl_wrap_content_preserves_to_end_of_sibling_width` in `codes/libs/ui/libbhoo/tests/android_xml.rs` renders `android_relative_nested_wrap::LAYOUT` in RTL (`300 × 56`) and verifies that `"More"` preserves its `56.0` dp width and `"Small"`, `"More"`, and `"After"` sit flush right-to-left without collapsing or overlapping.
- **Remaining limitations:** Opposing parent-edge rules on wrap-content axes, complete Android font/theme behavior and real-device parity remain unverified.

### Z-order sibling pointer occlusion

When `!f.any_scroll_dragging()`, visible higher-z siblings in `Kind::Relative` whose `blocks_pointer_for_lower_siblings()` is `true` and whose clipped bounds contain the pointer occlude lower siblings from pointer/wheel events via `Frame::without_pointer_input` ([`composition.rs`](../../src/systems/material/mod.rs), [`android_xml.rs`](../../src/systems/android/xml/mod.rs)) while preserving keyboard focus and key events.
