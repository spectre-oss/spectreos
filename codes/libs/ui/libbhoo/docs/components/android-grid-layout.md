# Android XML GridLayout

Status: Initial automatic-placement support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework layout |
| Priority | P1 |

## Scope and variants

GridLayout places children into shared rows and columns. The initial implementation supports automatic and explicit row/column placement, row and column spans, horizontal or vertical orientation, columnCount or rowCount, natural track sizes, cell start/end/center/fill gravity, initial row and column weights, baseline alignment for supported text-based views, AOSP default margins, explicit margins, and alignMargins/alignBounds modes, RTL, basic wrap-content height and child actions.

## State and interaction contract

Children retain normal visibility and interaction behavior. Hosts can supply the theme's default gap; explicit margins and Space views retain their own values. Minimum track sizes use grid-line constraints so overlapping spans can share space. Weight distribution accounts for individual children's minimum sizes, including multiple children sharing a row or column. Complex weighted constraints, baseline behavior for other views, Android's complete constraint solver and measurement and device parity remain open. Unsupported XML attributes are rejected.

## Motion and animation

Grid placement is direct; children provide their own effects. `Kind::Button` children derive their click ripple and hover state-layer corner radius from their own `self.radius` so rectangular, rounded, and oval buttons clip feedback to their actual bounds.

**Reduced motion:** Placement remains direct; child state layers follow the shared reduced-motion preference.

## Acceptance criteria

- Place automatically in horizontal and vertical order and mirror in RTL.
- Keep child button state layers and click ripples clipped to each button's own corner radius inside nested wrap-content grids.
- Implement the remaining GridLayout constraints and compare with Android on a device.

## Guidelines and visual references

- [Android GridLayout reference](https://developer.android.com/reference/android/widget/GridLayout)
- No verified side-by-side upstream Android screenshot is bundled locally; visual parity with Android remains unverified.

## Downloaded reference implementation

- [Android 15 GridLayout source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/GridLayout.java), stored locally at `context/reference-sources/android-framework/GridLayout.java`. See [licenses and attribution](../licenses-and-attribution.md).

## Wrap-content child measurement

Grid track minimums, child height/baseline calculation and placement use the current measured width of supported wrap-content children, including runtime text changes. A focused example shows a Button resizing and moving the next column in LTR and RTL. The parent width still constrains measurement, and complex GridLayout constraints, full Android theme/text measurement and device parity remain open. Rust grid regression tests cover the measured width and placement.

## Nested wrap-content width

A wrap-content GridLayout calculates its own width from grid tracks using the active renderer text metrics and current child state. A focused nested example checks the resized child, next column and following outer sibling in LTR and RTL; Rust coverage includes non-default text metrics. Complex weighted constraints, Android font/theme measurement and device parity remain unverified.

## Nested grid button ripple boundary repair (`issue-android-grid-nested-wrap-r1`)

User review of `android-grid-nested-wrap-r1` reported that clicking a button inside the nested wrap-content GridLayout showed an incorrect click animation and clipping boundary (`issue-android-grid-nested-wrap-r1`).

- **Cause and fix:** `Kind::Button` in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` previously passed a hardcoded `12.0` dp corner radius to `Frame::state_layer`, which clipped hover and click ripples with rounded corners even on rectangular buttons (`self.radius == 0.0`) such as those in `codes/apps/bhoo-web/src/android_grid_nested_wrap.rs`. `Kind::Button` now derives its state-layer and click ripple corner radius from `self.radius` (`b.width.min(b.height) * 0.5` when `self.radius < 0.0` for oval shapes, otherwise `self.radius`).
- **Validation evidence:** The Rust regression test `nested_grid_button_ripple_matches_button_radius` in `codes/libs/ui/libbhoo/tests/android_xml.rs` clicks the `"Small"` button in `android_grid_nested_wrap` and asserts that the emitted `DrawCommand::Ripple` matches the button's exact bounds and has `corner_radius == 0.0`.
- **Remaining limitations:** `issue-android-grid-nested-wrap-r1` remains open in `codes/libs/ui/libbhoo/docs/reviews/problems.json` until a linked retest packaged with `--repairs issue-android-grid-nested-wrap-r1` is accepted by the user. Custom Android `RippleDrawable` masks, unbounded/borderless ripples, theme-driven button defaults, complex weighted GridLayout constraints, Android font/theme measurement, and real-device visual/motion parity remain unverified.
