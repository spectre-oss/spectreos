# Android XML DialerFilter

Status: Initial implementation of the deprecated QWERTY filter; Android input and device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (deprecated) |
| Priority | P1 |

## Scope and variants

The XML importer accepts a DialerFilter with required `@android:id/primary` and `@android:id/hint` EditText descendants. Five Android display modes are represented; input uses uppercase text and the Android dialer character set. Optional icon focus handling and exact Android child styling remain open.

## State and interaction contract

The host can type characters, delete, clear, set a mode, or append programmatically. Typing a non-dialer character in the combined mode hides digits; deleting back to equal lengths restores the combined mode. The web host can provide native text input, including paste and deletion, through complete-value updates. It keeps browser composition text until commit and preserves the cursor through common uppercase changes. Exact Android IME behavior, selection mapping for all filtered or expanded characters, hardware key mapping, text watchers, focus behavior and multiple simultaneously scoped instances remain open.

## Motion and animation

Mode and visibility changes are immediate. Android theme-specific focus and press behavior remains open.

**Reduced motion:** Changes are immediate.

## Acceptance criteria

- Import the two named EditText children and render their filter state.
- Test combined and single-line modes, deletion and clearing.
- Implement remaining Android input behavior and compare with a device.

## References

- [Android 15 DialerFilter source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/DialerFilter.java)
- [Android 15 DialerKeyListener source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/text/method/DialerKeyListener.java)
