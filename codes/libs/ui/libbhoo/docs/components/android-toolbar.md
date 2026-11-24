# Android framework Toolbar

Status: Initial XML title, subtitle, navigation drawable and custom child views. Initial flat XML menu import is supported; full Android menu inflation and device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The importer accepts `Toolbar` with `android:title`, `android:subtitle`, `android:navigationIcon` and `android:navigationContentDescription`. The navigation icon uses a local PNG drawable, and placement respects RTL. XML content insets support relative and absolute edges and the navigation/action-specific overrides; title margins support a common value and individual edges. Title and subtitle colors accept Android XML color values and runtime overrides. Their text appearances support inherited styles with textSize, textStyle and textColor.

Custom child views can use start, end and center layout gravity, margins and vertical alignment. Host-supplied menu entries can appear as actions or in an overflow popup. The menu importer supports items with IDs, titles, local PNG and static VectorDrawable paths and transformed groups or selected built-in icons, showAsAction including withText, enabled and visible attributes. Icon-and-text actions are shown in wide or landscape configurations and use additional menu space. XML groups can provide IDs, menu categories, ordering, and enabled and visible defaults, which individual items can override. Items are stably sorted according to Android's category and order mapping. The host can change all items' enabled or visible state by group ID at runtime. Action allocation follows group promotion and demotion rules; pixel-accurate measurement remains open. Checkable and exclusive XML groups render checkbox or radio indicators; hosts update checked state explicitly on selection, as Android menu handlers do. Nested menus, additional vector theme and tint modes, other drawable icon types, collapsing app bars, advanced inset interactions, and the full Android theme and additional text appearance attributes remain open.

## State and interaction contract

A navigation tap reports the toolbar ID and navigation event; custom child actions report their XML IDs. Menu actions report their host-supplied item IDs and keep stable press identities if the menu changes. Disabling the toolbar closes an open overflow menu. Title text can be overridden at runtime using the layout state. Title and subtitle are single-line and use end ellipsis when space is limited.

## Motion and animation

- **CollapsingToolbarLayout GPU title scaling**: Implemented via `CollapsingTitleMode::Scale` in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`. Interpolates title scale based on active scroll fraction `0.001 < collapse_t < 0.999`, using GPU scaling of `title_sp / expanded_sp`. Mirrors AOSP `CollapsingTextHelper` behavior with pixel‑identical endpoints `0.0` and `1.0`.
- Used in `Wisp Launcher Settings` (`codes/apps/wisp/src/settings.rs`) where the toolbar collapses from **136dp → 56dp** and **32sp → 20sp**, with top/bottom `EdgeStretch` split to keep the toolbar anchored.
- Validation: `launcher_demo::drag_tests` cover the collapsing toolbar layout and title scaling. Filmstrip evidence in `context/system-ui-flow/animations/strip-v5-collapsing-toolbar.png`.
- **Remaining limitations**: Full Android visual parity for toolbar elevation, icon animations, and system UI integration remains open.

The navigation button uses the shared state layer.

**Reduced motion:** The shared controls honor the reduced-motion setting.

## Acceptance criteria

- Render XML title, subtitle and navigation drawable and report navigation taps.
- Compare sizing, typography, actions and motion against Android.

## References

- [Android 15 Toolbar source](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/Toolbar.java) (Apache-2.0).

Vector menu drawables are rasterized at build time with `rsvg-convert` at the importer target density; nested translation, rotation, pivot, scaling and group-scoped clipping paths are supported. Fixed colors and `?attr/colorControlNormal` are resolved at paint time using SRC_IN; other tint modes and theme attributes are rejected explicitly. Unsupported attributes and elements are rejected explicitly.

## Wave 14A and density scaling updates

- **Unconditional density scaling for non-collapsing `Kind::Toolbar`:** In [`android_xml.rs`](../../src/systems/android/xml/mod.rs), non-collapsing toolbars scale navigation/action slot bounds and icon sizes unconditionally via `slot_unit = state.dp(48.0)` and `icon_unit = state.dp(24.0)` across all `AndroidTheme` variants.
- **RTL `autoMirrored` navigation icon (`RULE_AUTO_MIRRORED = 255`) & extended title/subtitle text style (`RULE_TEXT_STYLE_EXT = 257`):** Wave 14A flips `autoMirrored` toolbar navigation icons horizontally in RTL via `PushTransform`/`PopTransform` and propagates `FontFamily::{Sans, Serif, Monospace}` and `TextStyle.italic` to toolbar title and subtitle text (verified by `wave14a_auto_mirrored_flips_image_icon_and_toolbar_nav_in_rtl_only` and `wave14a_text_style_ext_italic_and_font_family_across_widgets` in [`tests/android_xml.rs`](../../tests/android_xml.rs)). See [Android XML import](../android-xml.md).
