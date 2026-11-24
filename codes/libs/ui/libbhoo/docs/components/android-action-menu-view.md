# Android XML ActionMenuView

Status: Initial host-supplied action items and overflow; cell layout and Android device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

ActionMenuView presents actions inline and moves remaining actions into an overflow menu. The XML importer accepts an empty ActionMenuView; the host supplies menu items using the existing Android menu item state and receives selected IDs. This initial renderer uses a 56dp minimum slot, allocates at least two cells to text actions, shares whole spare cells among text actions, centers a lone action, and supports enabled and visible items, RTL and overflow. Menu XML accepts the withText flag: on wide or landscape configurations the icon and label appear together; hosts may override the configuration. XML child views, Android's remaining fractional expansion and rounding, custom action views, full MenuBuilder behavior and Android platform styling remain open.

## State and interaction contract

Pressing an enabled action reports its item ID. The overflow button opens a menu of remaining items; pressing an enabled item reports its ID and closes the menu. If a requested member of an active group cannot fit, earlier inline members of that group move into overflow, including required members. The host can change the menu contents or view width. This example does not invoke external intents.

## Motion and animation

The overflow uses the existing Android-style menu overlay; exact platform timing remains to be compared.

**Reduced motion:** The overlay is shown without added motion.

## Acceptance criteria

- Import the XML tag and render host-supplied actions and overflow items.
- Activate enabled actions and overflow entries; respect disabled items and RTL.
- Implement the remaining platform menu measurement and compare on Android.

## References

- [Android 15 ActionMenuView source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/ActionMenuView.java)
