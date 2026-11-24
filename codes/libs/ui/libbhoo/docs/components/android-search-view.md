# Android XML SearchView

Status: Initial local query field and string suggestions; Android theme fidelity and device comparison remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

SearchView supports iconifiedByDefault and queryHint XML attributes. It can start collapsed or always expanded. A host may enable a submit button and receives query changes and nonblank submissions. A host can supply local string suggestions and a minimum query length; the popup filters by case-insensitive word prefix, wraps its results within the available viewport, and reports the selected item index without launching an external intent. The search, clear and submit icons come from the Android framework vectors, and the field underline reflects focus. Popup and remaining field styling are approximate; Android CursorAdapter/searchable-provider integration, voice search, query backgrounds and full platform theme behavior remain open.

## State and interaction contract

Opening expands the field and requests text focus. The close button clears nonempty text first; when empty it collapses an iconified-by-default view. An always-expanded view stays open. Enter or the enabled submit button reports the original nonblank query. Arrow keys highlight local suggestions and preview their text in the query without refiltering the list; Enter activates the highlighted item without also submitting the query. The example is local and performs no network search.

## Motion and animation

SearchView visibility changes are immediate, following the framework's basic visibility update. Platform theme-specific effects remain to be verified.

**Reduced motion:** Visibility changes remain immediate.

## Acceptance criteria

- Expand, type, clear, collapse and submit a nonblank query with the native input.
- Verify the always-expanded variant and disabled behavior.
- Implement Android suggestion providers and searchable configuration, then compare voice affordance and platform styling.

## References

- [Android 15 SearchView](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/SearchView.java)
- [Android 15 SearchView layout](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/layout/search_view.xml)
