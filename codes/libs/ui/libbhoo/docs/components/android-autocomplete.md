# Android XML AutoCompleteTextView

Status: Initial host-backed autocomplete field; Android device styling and additional adapter features remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

AutoCompleteTextView accepts XML completion threshold, input type, dropdown width, height and offsets. The host supplies string suggestions. Filtering follows Android ArrayAdapter's case-insensitive whole-string or word-prefix matching, and a scrollable popup shows matches when the threshold is reached. The initial implementation filters synchronously and uses fixed-height text suggestions; custom adapters, asynchronous filters, alternate anchors and Android theme fidelity remain open.

## State and interaction contract

Typing updates the field through the host input adapter. Selecting a suggestion replaces its text and reports the field ID and original suggestion index. The popup dismisses after selection, outside press or Escape, and reopens when the text changes. Arrow keys highlight suggestions and Enter selects one. The initial threshold uses UTF-16 code-unit length as Android does. Popup behavior and input-method interaction still require Android device comparison.

The web input forwards Up, Down, Enter and Escape to an open popup while retaining input focus; committed selections update the native editor and cursor.

## Motion and animation

The popup opens and closes immediately; scrolling uses the shared touch fling physics.

**Reduced motion:** Popup changes are immediate and fling is disabled.

## Acceptance criteria

- Import and validate supported field and dropdown attributes.
- Filter by threshold and prefix, select an item, and dismiss without an unintended selection.
- Scroll a long suggestion list and test touch and typing in the webxdc.
- Compare platform appearance, timing and input behavior against Android.

## References

- [Android 15 AutoCompleteTextView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/AutoCompleteTextView.java)
- [Android 15 ArrayAdapter source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/ArrayAdapter.java)
