# Android XML CheckedTextView

Status: Initial checked-text rendering and XML support.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

CheckedTextView renders text with an optional trailing checked indicator and supports right-to-left layout.

## State and interaction contract

The host owns checked state and responds to click actions; the view does not toggle itself. Without an explicit drawable the check indicator is a built-in approximation, shown only when checked. `android:checkMarkGravity` accepts `start`, `end`, `left` and `right`; logical edges follow layout direction. `android:checkMarkTint` accepts a literal color, color resource or supported state selector, including checked and enabled state.

`android:checkMark` accepts `@null`, a local solid XML shape with explicit dimensions, or a drawable selector containing such shapes or `@null`. Supported shapes are rectangles with optional uniform corners and circular ovals; selectors may match enabled, pressed, focused, selected, activated, hovered and checked states. Custom selectors can display a mark when unchecked. Bhoo reserves the maximum custom drawable width plus 8 dp; the built-in approximation reserves 32 dp. The default source-in tint combines the custom shape alpha with tint alpha. Bitmap, vector, animated and other drawable kinds, tint blend modes beyond src_in, src_atop, src_over, multiply, screen and add, antialiased edge compositing, theme resolution and exact Android spacing/font metrics remain unsupported.

`CheckedTextView` also accepts the supported TextView compound shape drawables, timed `<animation-list>` compound drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`, Wave 21A/21B, advancing via `f.time()` and clip-gated `f.wake_at()` and freezing on frame `0` under reduced motion), Wave 24A/24B `autoLink`/`textColorLink`/`linksClickable` auto-link highlighting, `1dp` underlining, deferred clickable-container handling, and `Role::Link` clicks updating `LayoutState.last_clicked_link` (`RULE_TEXT_LINKS_SPANS = 306`, without offline `map` address geocoding or arbitrary runtime `SpannableString`/`ClickableSpan`), drawable padding and drawable tint selectors. Checked-state tint conditions follow the host-owned checked value; enabled, pressed and other supported states use the same rules as TextView. Check-mark space is reserved separately, so the compound shapes/frames and text do not overlap it. Logical start and end follow layout direction. Static bitmap/vector compound drawables outside `<animation-list>`, `<animated-vector>`/`<animated-selector>` compound drawables, selectors without initial intrinsic dimensions, tint modes beyond src_in, src_atop, src_over, multiply, screen and add, exact Android intrinsic measurement and device appearance comparison remain open; see [TextView](android-textview.md).

Explicit `android:checkMarkTintMode` accepts `src_in`, `src_atop`, `src_over`, `multiply`, `screen` and `add` for supported custom mark shapes. The importer rejects this attribute for the built-in mark until its drawable tinting is supported.

## Motion and animation

The checked indicator updates with state. Click feedback uses Bhoo state layers.

**Reduced motion:** State changes remain visible without requiring motion.

## Acceptance criteria

- Import the initial checked state and preserve host changes.
- Place the indicator at the trailing edge in either layout direction.
- Check start/end and left/right placement, plus enabled and disabled tint selectors.
- Check shape selector changes for checked, unchecked and disabled states while text position remains stable.
- Compare appearance with Android on a device.

## Validation and visual status

Importer and Rust rendering tests verify supported gravity and selectors, custom mark size and unchecked visibility, disabled tint and RTL. The compound drawable tests additionally cover drawable import, checked tint, mark spacing and RTL; their focused review is queued. Focused webxdc examples allow interactive visual review. Exact Android check-mark shape, spacing and motion parity have not been verified on a device.

## Guidelines and visual references

- [Android CheckedTextView reference](https://developer.android.com/reference/android/widget/CheckedTextView); no verified comparison screenshot is available for this change.

## Downloaded reference implementations

- Android 15 [CheckedTextView.java](https://android.googlesource.com/platform/frameworks/base/+/android-15.0.0_r36/core/java/android/widget/CheckedTextView.java), local `context/reference-sources/android-framework/CheckedTextView.java` (Apache-2.0), for gravity, checked state and tint attributes. Bhoo renders its own Material check vector.

## References

- [Android CheckedTextView reference](https://developer.android.com/reference/android/widget/CheckedTextView)
