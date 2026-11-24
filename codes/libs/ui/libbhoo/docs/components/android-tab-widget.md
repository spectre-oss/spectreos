# Android XML TabWidget

Status: Initial standalone implementation of the deprecated Android widget; TabHost integration is available; platform theme parity remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (deprecated) |
| Priority | P1 |

## Scope and variants

TabWidget accepts XML child views and the boolean `tabStripEnabled` attribute. Fixed-width children retain their XML widths and wrap-content children use Bhoo's intrinsic content measurement, including resolved margins, when they fit; positive `layout_weight` values divide the remaining width, with optional `weightSum`. When their total exceeds the available width, Bhoo distributes overflow across them in child order, clamping each width at zero, following Android's approach. Their group is aligned to logical start and mirrors in RTL; child height remains an approximation based on the widget height and vertical margins. Combinations containing match-parent children currently use equal-width slots. Wrap-content text uses renderer-provided ASCII glyph advances and is recalculated after host text changes; non-ASCII glyphs use an approximation and kerning is not reproduced. Bhoo approximates Android's preliminary unconstrained measurement of weighted zero-width children, weight edge cases and integer rounding; Android's pixel rounding and full measurement remain unverified. Children retain their XML content. Initial selected background and default strip are approximations rather than Android theme drawables. `android:tabStripLeft` and `android:tabStripRight` accept `@null` or local solid rectangular shape drawables with explicit width and height. Strips are painted after children, extending on either side of the selected tab according to Android 15 geometry; ancestor clipping applies to the portions extending beyond the widget. The intrinsic widths set minimum extension, and each height controls its bottom alignment. For an unspecified side when the other is custom, Bhoo draws a 2 dp theme-colored approximation. Selectors, bitmaps, nine-patches, tinting and full Android measurement remain unsupported.

## State and interaction contract

Tap an enabled tab to select it. Selection and the latest clicked child index are available to the host. Right-to-left layout reverses visual order. Disabled, invisible and gone children cannot be clicked. Invisible children retain their layout slot but do not paint or receive input; gone children lose their slot. An already-selected invisible child remains selected. Behavior when the selected child becomes gone is an approximation and needs comparison with Android. Content switching is not provided by standalone TabWidget.

## Motion and animation

Selection changes immediately; press state uses the existing Bhoo state layer. Android theme-specific press behavior remains open.

**Reduced motion:** Selection is immediate.

## Acceptance criteria

- Import XML child tabs, select an enabled tab and change visual order in RTL.
- Compare theme drawables and measurement on an Android device.

## References

- [Android 15 TabWidget source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TabWidget.java)

## Validation and source

Importer tests cover sized solid shapes, `@null`, invalid shapes and wrong widget types. Rust rendering tests cover selected-tab geometry, padded widgets, fixed and wrap-content sizing, glyph widths and text changes, weights, margins and overflow, invisible child input and layout, right-to-left ordering, `@null` and disabled strips. The focused example uses red and blue strips; a 320 × 440 browser test checks their colors and RTL ordering. Another phone-sized browser test checks invisible-child selection and restoration in both layout directions. A 320 × 600 browser test checks the fixed-width XML example, overflow and RTL ordering; a 320 × 650 browser test checks margins, overflow and RTL. A 320 × 700 browser test checks weighted proportions and selection in narrow and RTL layouts. A 320 × 760 browser test checks content-sized widths, narrow layout, selection and RTL. A 320 × 850 browser test compares widths with the renderer's individual glyph advances. Android device rendering, drawable states and pixel parity remain unverified. The Apache-2.0 [Android 15 TabWidget implementation](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TabWidget.java) is saved at `context/reference-sources/android-framework/TabWidget.java`.
