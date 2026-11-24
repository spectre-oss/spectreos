# Android XML MultiAutoCompleteTextView

Status: Initial comma-tokenized suggestions; platform fidelity remains open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

MultiAutoCompleteTextView shares autocomplete XML dropdown configuration and host-provided suggestions. The host explicitly enables the comma tokenizer, as Android requires a tokenizer to be installed. Filtering uses the token before the UTF-16 selection end, preserving the remaining text when a suggestion is selected and appending comma-space unless already terminated. The browser editor supplies its cursor position. Custom tokenizers, adapters, asynchronous filtering, rich text, input method composition and Android visual parity remain open.

## State and interaction contract

The threshold applies to the current token. Selection replaces from the start of that token to the cursor and returns the selected index. Remaining text is retained. Dismissal, scrolling and keyboard behavior share AutoCompleteTextView's initial implementation.

The web input forwards Up, Down, Enter and Escape to an open popup while retaining input focus; committed selections update the native editor and cursor.

## Motion and animation

The popup appears immediately; suggestion scrolling uses shared touch fling physics.

**Reduced motion:** Popup changes are immediate and fling is disabled.

## Acceptance criteria

- Test threshold and selection within an existing comma-separated field, including UTF-16 cursor positions.
- Test repeat selection and moving the cursor in a browser.
- Compare popup styling and editing behavior against Android on a device.

## References

- [Android 15 MultiAutoCompleteTextView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/MultiAutoCompleteTextView.java)
- [Android 15 ArrayAdapter source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/ArrayAdapter.java)
