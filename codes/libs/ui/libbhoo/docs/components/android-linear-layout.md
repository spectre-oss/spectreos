# Android XML LinearLayout

Status: Partial vertical and horizontal XML layout with weights (including Wave 11B horizontal `Wrap`-height weight distribution for `width="0dp"` weighted children), direction-aware order and initial horizontal baseline alignment.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework layout |
| Priority | P1 |

## Scope and variants

Android LinearLayout places children vertically or horizontally, with fixed sizes, margins and weighted remaining space.

## State and interaction contract

Horizontal children follow the caller-selected or XML-inherited layout direction; relative margins follow start and end. Weighted children share positive or negative remaining space, clamped to nonnegative size. An explicit `weightSum` can leave space for parent gravity. Parent gravity positions the group and aligns children on the cross axis; child layout gravity can override it. Horizontal text and button children align by baseline within the top or bottom gravity group by default; `android:baselineAligned="false"` disables this alignment. Center-vertical gravity is positioned without baseline adjustment, as in the Android reference. A wrap-content horizontal layout accounts for the combined baseline ascent and descent. `android:baselineAlignedChildIndex` can expose a direct child's baseline to a containing LinearLayout or RelativeLayout, including when that child follows earlier vertical children. Bhoo returns no baseline for a missing or unsupported child or an Android-invalid preceding weighted child; it does not reproduce Android's runtime exceptions for those cases. Nested weighted sizing, margins, dividers and gravity still require Android device comparison. Solid rectangular divider drawables support beginning, middle and end placement, padding and direction-aware layout. Children are clipped to the padded area by default; `android:clipToPadding="false"` allows drawing into the padding while retaining the parent boundary. Exact Android measurement, baseline support for remaining view types, other divider drawables and complete gravity combinations remain incomplete.

Horizontal baseline alignment also accounts for the rendered label baselines of CheckBox, RadioButton and Switch children, including their current text size and padded/clipped control row. EditText, AutoCompleteTextView and MultiAutoCompleteTextView contribute the baseline of their entered text in Bhoo's field layout. The browser review page uses independent native input adapters for these fields; an automated browser check commits distinct values to all three and checks the rendered text. These baseline metrics follow Bhoo's renderer; Android font and theme parity remain unverified.

## Motion and animation

Layout changes are applied directly; child controls supply their own interaction motion.

**Reduced motion:** Layout remains directly positioned.

## Acceptance criteria

- Place horizontal children in both directions with weights and relative margins.
- Verify vertical orientation and disabled or hidden children.
- Align mixed-size horizontal text by baseline, honor the XML opt-out, and reserve enough wrap-content height.
- Use an explicitly indexed child baseline when a nested LinearLayout participates in baseline alignment.
- Compare sizing and layout with Android on a device.

## Validation

- **Rust runtime suite (`270` passing tests in [`tests/android_xml.rs`](../../tests/android_xml.rs); `842` default Rust tests across `73` suites in `libbhoo`, `0` warnings):**
  - Wave 11B: `horizontal_linear_wrap_height_distributes_weight_to_zero_dp_text_children` verifies horizontal `LinearLayout` (`Size::Wrap` height) distributing remaining width via `linear_main_lengths` + `layout_intrinsic_for_width(..., exact_width)` to `width="0dp", weight="1"` wrapped `TextView` rows (single-line subject + badge + icon, nested weighted vertical `LinearLayout` column, and multi-line wrapped weighted text) at `density = 1.5` so rows keep compact wrapped height instead of collapsing to `0px` width / 1 character per line.
  - Earlier baseline and layout tests cover mixed-size baselines, `baselineAligned="false"`, RTL, combined ascent/descent in wrap-content, indexed nested children in `LinearLayout` and `RelativeLayout`, and multi-line `Button` first-line baselines.
- **Python XML importer suite (`213` passing tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py)):** covers weights, `weightSum`, `baselineAligned` true/false, `baselineAlignedChildIndex`, dividers, and invalid attribute rejections.
- **Visual verification:** The focused browser example shows default alignment, the disabled variant and indexed nested rows; Wave 11B native headless Weston + Mesa GLES captures (`context/android-xml-comparison/gmail-xml-detail.png` and `gmail-xml-compose.png` via [`android_gmail_capture.rs`](../../../../codes/libs/ui/libbhoo/examples/android_gmail_capture.rs) at `585 × 1266`, `density = 1.5`) exercise weighted `0dp` horizontal subject, sender, and compose rows end-to-end. Text baseline metrics are those of Bhoo's renderer; Android font metrics and device-level pixel parity remain unverified.

## References

- [Android LinearLayout reference](https://developer.android.com/reference/android/widget/LinearLayout)
- [Pinned AOSP LinearLayout source (Apache-2.0)](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/LinearLayout.java)

## Guidelines and visual references

The [Android LinearLayout guide](https://developer.android.com/develop/ui/views/layout/linear) describes orientation, sizing and weights. No verified upstream image showing the mixed-size baseline case is recorded; the Bhoo example is a test illustration, not an Android screenshot.

## Downloaded reference implementation

The [Android 15 LinearLayout source snapshot](../../../../context/reference-sources/android-framework/LinearLayout.java) retains its Apache-2.0 copyright and license header. The matching upstream revision is linked above; see the [license and attribution record](../licenses-and-attribution.md). It was consulted for baseline measurement, the XML opt-out and top/bottom/center gravity handling.

## Nested wrap-content width

A wrap-content LinearLayout, RadioGroup or standalone TableRow measures the current widths of its supported visible children with the active text metrics, margins, padding and horizontal dividers. Horizontal rows sum child widths; vertical columns use the widest. A focused example changes a nested Button label and checks that the rest of the row and the following sibling move in LTR and RTL. Complex weighted, nested and Android device measurement parity remain open. The Apache-2.0 Android 15 LinearLayout source above is stored at `context/reference-sources/android-framework/LinearLayout.java`.

## Multi-line Button baseline

A multi-line Button reports its first rendered line’s Bhoo baseline, including top compound drawable spacing and supported vertical gravity, for horizontal LinearLayout and RelativeLayout alignment. Rust coverage compares the first rendered Button line to a same-size TextView in LTR and RTL. Android font and device parity remain unverified.

## Horizontal `Wrap`-height weight distribution (Wave 11B)

When measuring vertical `Size::Wrap` height on a horizontal `Kind::Linear` or `Kind::RadioGroup` in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` with `inner_width > 0.0` and any visible child carrying `weight > 0.0`, `layout_intrinsic_for_width` now precomputes horizontal child lengths with `linear_main_lengths(true, state, inner_width, inner_width, metrics, id)` and passes `(widths[i], n.weight > 0.0)` as `(available_width, exact_width)` to each child's vertical `layout_intrinsic_for_width` measurement (as well as to `standalone_row_metrics` for baseline row height). When `exact_width` is `true`, children declared with Android's idiomatic `android:layout_width="0dp"` + `android:layout_weight="1"` (such as wrapped `TextView`, `Button`, nested `LinearLayout`/`FrameLayout`/`ConstraintLayout`/`RelativeLayout`/`GridLayout`, `VideoView`, or `adjustViewBounds` images) measure their wrapped vertical height against their actual distributed width instead of `0.0` (which previously caused single-character-per-line vertical inflation). Vertical weighted containers with `Wrap` height, `measureWithLargestChild`, and live Android device parity remain open.
