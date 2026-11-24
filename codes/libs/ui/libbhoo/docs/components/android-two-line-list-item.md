# Android XML TwoLineListItem

Status: Initial XML support for this deprecated Android widget.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework layout |
| Priority | P2 |

## Scope and variants

TwoLineListItem extends RelativeLayout. It accepts caller-supplied child views and RelativeLayout rules, including Android's standard text1, text2 and selectedIcon IDs. The host can update the two text values by their IDs. The implementation uses the existing RelativeLayout engine; Android framework styling and platform resource inflation remain incomplete.

## State and interaction contract

The row and its children use their normal visibility, direction, click and state behavior. Caller-controlled text updates are shown in the example. Android device comparison and full RelativeLayout parity remain open.

## Motion and animation

Layout is direct; interactive views provide their own press effects.

**Reduced motion:** Placement remains direct.

## Acceptance criteria

- Import standard Android text IDs and relative rules without ID collisions.
- Update both text children and activate the row.
- Compare against a device and validate remaining RelativeLayout behavior.

## References

- [Android TwoLineListItem source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TwoLineListItem.java)
- [Android RelativeLayout source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/RelativeLayout.java)
