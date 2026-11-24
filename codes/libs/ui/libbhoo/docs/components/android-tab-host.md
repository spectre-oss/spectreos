# Android XML TabHost

Status: Initial view-ID content switching; Android theme and platform parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget (deprecated) |
| Priority | P1 |

## Scope and variants

The importer accepts a TabHost containing an XML TabWidget and `@android:id/tabcontent` FrameLayout. The host configures an ordered list of content view IDs corresponding to XML indicator children, or supplies tags and labels to generate indicators in an empty TabWidget. Current support requires matching child counts and a FrameLayout content container. A host callback can lazily supply XML content, and view-ID and factory-backed tabs can be mixed in the same content container. Factory nodes are cached until the host is reconfigured. Android icon and custom generated indicators, arbitrary runtime views and activity-backed tabs remain open.

## State and interaction contract

The first tab starts selected, and factory-backed content is created on first display. Tapping an indicator switches to its associated view and hides the others; programmatic selection validates the index or resolves the first matching tag. Nested content IDs are accepted. Right-to-left direction reverses indicator order. Android's lifecycle, focus, notifications and platform theme behavior remain open.

## Motion and animation

Switching view-ID content is immediate, as in the platform strategy. Child view animations and Android drawable feedback remain open.

**Reduced motion:** Content switching is immediate.

## Acceptance criteria

- Import the Android tabs and tabcontent IDs, configure view-ID tabs and select content.
- Implement additional content strategies and platform indicator behavior, then compare with Android.

## References

- [Android 15 TabHost source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TabHost.java)
