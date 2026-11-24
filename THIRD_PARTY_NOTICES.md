# Third-Party Notices

SpectreOS's own license is provided in [LICENSE](LICENSE). Third-party materials retain their applicable licenses and copyright notices.

## Material navigation icons

The home, folder, settings, alarm, clock, hourglass, stopwatch, menu, menu-open, back, forward, search, more, add and tab icons and their included variants in `codes/libs/ui/libbhoo/assets/material-icons` come from Google's [Material Icons](https://github.com/google/material-design-icons), Copyright Google LLC, Apache-2.0. Their SVG copies were obtained from [MUI's Material Icons package](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-icons-material/material-icons) at revision `0c82795c732a0ae809fd51f9537ee70747ef20a1`; the MUI repository's MIT license is retained with those copies. The [Apache-2.0 license](docs/TDD/libbhoo/reference-licenses/compose-material3/LICENSE.txt) accompanies distributions. The SVG paths and rectangles are sampled and triangulated into `material_navigation_icons.rs`; curves are approximated and no endorsement is implied.

## Material Web documentation snapshot

This repository includes selected Material Web documentation and illustrations under `docs/TDD/libbhoo/material-design/upstream`.

- Source: [material-components/material-web](https://github.com/material-components/material-web)
- Revision: `d872455c4be56022da2095349b30ef46e0b55bc3`
- Included paths from upstream: `docs/`, `README.md`, and `LICENSE`
- License: [Apache License 2.0](docs/TDD/libbhoo/material-design/upstream/LICENSE)
- Original attribution: retain the copyright and other notices in the included upstream files.

The snapshot preserves upstream files without rewriting. It is reference documentation, and its inclusion does not imply endorsement by Google or the Material contributors.

## Reference implementation research

### AndroidX standard motion token values

Bhoo's `codes/libs/ui/libbhoo/src/motion.rs` adapts the standard effects and spatial damping/stiffness values from AndroidX `StandardMotionTokens.kt`.

- Origin: [AndroidX source at the pinned revision](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/StandardMotionTokens.kt)
- Copyright 2024 The Android Open Source Project.
- License: [Apache License 2.0](docs/TDD/libbhoo/reference-licenses/compose-material3/LICENSE.txt).
- Changes: token names are adapted to Bhoo's API. Bhoo's analytic damped-oscillator solver is independently written.

Retain this attribution and the linked license with distributions using these values.

Selected Material Web, Material Android, Compose Material 3, Angular Material, and Flutter material_ui sources are downloaded into ignored `context/reference-sources`. They are not incorporated into Bhoo's application code by the reference-download task.

Their original legal texts are preserved in [the TDD license archive](docs/TDD/libbhoo/reference-licenses/). See the [source index](docs/TDD/libbhoo/reference-implementations.md) for revisions and the [license review](docs/TDD/libbhoo/licenses-and-attribution.md) for attribution requirements when source is adapted.

This record covers these reference materials. It is not an inventory of every dependency used to build SpectreOS.

## Material ink, timing, progress and loading geometry

- Material Web bounded ripple constants and geometry, menu/dialog timing and easing references: Copyright 2021–2022 Google LLC, Apache-2.0. Source revision `d872455c4be56022da2095349b30ef46e0b55bc3`, paths `ripple/internal/ripple.ts`, `ripple/internal/_ripple.scss`, `internal/motion/animation.ts`, `menu/internal/menu.ts`, `dialog/internal/animations.ts`. The Rust state machines, samplers, GLES shader and browser command renderer are independently implemented. The pinned [Material Web license](docs/TDD/libbhoo/reference-licenses/material-web/LICENSE) is retained.
- Material loading shape recipes, morph cadence/spring/rotation, progress timelines and Expressive spring values: Copyright 2022–2024 The Android Open Source Project, Apache-2.0. Pinned Compose revision above, paths `MaterialShapes.kt`, `LoadingIndicator.kt`, `ProgressIndicator.kt`, `tokens/ExpressiveMotionTokens.kt` and `tokens/MotionTokens.kt`.
- `tools/BhooShapes.java` adapts the seven shape recipes and repeat/mirroring logic. `src/loading_shapes.rs` contains derived matched cubic data, generated using AndroidX `graphics-shapes-desktop` 1.1.0. Original shape geometry is preserved; data are serialized as Rust constants. The generation process uses Kotlin stdlib 2.1.20, AndroidX collection 1.5.0 and JetBrains Compose animation-core-desktop 1.8.2 to export the reference spring duration. Checksums and URLs are in `tools/generate-bhoo-loading.py`. None of those JVM libraries is distributed in or required by Bhoo's runtime.

The AndroidX-derived materials retain the [Apache-2.0 license](docs/TDD/libbhoo/reference-licenses/compose-material3/LICENSE.txt). See [the implementation/provenance record](docs/TDD/libbhoo/effects-implementation.md). These adaptations do not imply upstream endorsement or Android platform pixel parity.

## Angular Material comparison runtime

The optional browser comparison reference bundles genuine, unmodified Angular Material/CDK 22.2.6 with Angular core/common/compiler/forms/platform-browser 22.2.5. Those packages are Copyright Google LLC and contributors, MIT licensed. Their source repositories are [angular/components](https://github.com/angular/components) and [angular/angular](https://github.com/angular/angular). The reference also bundles RxJS 7.8.2 (Apache-2.0) and tslib 2.9.1 (0BSD); their original notices are retained. This runtime is separate from the earlier pinned source-only research snapshot.

`tools/build-bhoo-angular.mjs` gathers the original license/notice files of every package actually included by esbuild, and writes the inventory and complete texts to `codes/apps/bhoo-web/site/angular/versions.json` and `THIRD_PARTY_LICENSES.txt`. Distribute those files and esbuild's `reference.js.LEGAL.txt` alongside the bundle. Build dependencies are pinned in `codes/apps/bhoo-web/angular-reference/package-lock.json`; the reference templates and host are independently written GPL-3.0-only project code. Official Angular Sass emits the reference theme. No upstream documentation page, branding, fonts or example image collections are copied. This comparison does not imply Google or Angular endorsement.

The Angular reference is development/demo code, not a dependency of the libbhoo Rust core or its Wayland/GLES backend. See [comparison decisions and limitations](docs/TDD/libbhoo/angular-comparison.md).

## Angular-derived Bhoo rendering values

Bhoo's Rust comparison profile adapts checkbox checkmark coordinates, classic MDC progress keyframe/arc values, elevation shadow parameters, and ripple/menu/dialog/form-field/datepicker/timepicker/expansion/snack-bar/tooltip/button-toggle timing and geometry from the installed Angular Material/CDK 22.2.6 distribution. Copyright Google LLC and contributors, MIT licensed. The original [Angular Material license](docs/TDD/libbhoo/reference-licenses/angular-material/LICENSE) is retained. Sources: [Angular components](https://github.com/angular/components), package modules `checkbox`, `progress-bar`, `progress-spinner`, `core`, `menu`, `dialog`, `form-field`, `datepicker`, `timepicker`, `expansion`, `snack-bar`, `tooltip` and `button-toggle`. Changes: rendering values are expressed as Rust vector commands, bounded state machines and timeline samplers for Canvas/GLES; no Angular runtime is linked into the Rust library.

The filter chip additionally adapts Angular Material 22.2.6 `chips/chip.scss`
transition values and the `chip-option` checkmark SVG coordinates/viewBox,
under the same Google/contributors MIT license. The Rust implementation draws
the path progressively and samples CSS-style reversible width/translation and
overlay transitions. The browser test seeks the genuine reference transitions;
it does not replace or retime Angular's chip.

The reference stepper's inline pencil icon uses Google's Material icon geometry, Copyright Google LLC, Apache-2.0; retain the [Apache-2.0 license](docs/TDD/libbhoo/reference-licenses/material-web/LICENSE). These adaptations do not imply upstream endorsement or full visual/state parity.

## Beer CSS reference

The optional browser comparison includes the minified CSS and JavaScript from Beer CSS 5.0.3, commit `47592ff1a644df437ebdf08c8dbdc967ed2e68db`, Copyright (c) 2021 Beer css!, under the MIT license. The full license is included in `beercss/LICENSE.txt` in the demo and in `docs/TDD/libbhoo/reference-licenses/beercss/LICENSE.txt` in the source. The Bhoo page transition is an independent implementation of behavior described by Beer CSS. The static shape meshes in `codes/libs/ui/libbhoo/src/beercss_shapes.rs` are derived from Beer CSS SVG files; Beer CSS credits Daria Dororoshchuk for the shape artwork and KTibow for the animated loading indicator.

## Beer CSS-inspired continuous showcase

The one-page Bhoo gallery adapts the menu order and page layout of `context/beercss/src/home/page.vue` (Beer CSS 5.0.3, Copyright (c) 2021 Beer css!, MIT). Its navigation rail, floating toolbar, introduction, headings, source viewer and examples are independently rendered by Bhoo Rust/WASM. Beer CSS's JavaScript and CSS framework are not loaded by this gallery. The original MIT license is retained in `docs/TDD/libbhoo/reference-licenses/beercss/LICENSE.txt` and packaged as `beercss/LICENSE.txt`. The Bhoo name and introduction replace Beer CSS branding.

## Material Symbols Outlined

The 40 SVGs in `codes/libs/ui/libbhoo/assets/material-symbols` are official Google Material Symbols Outlined, Copyright Google LLC, Apache-2.0. Source: https://github.com/google/material-design-icons at revision `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`, `symbols/web/<name>/materialsymbolsoutlined/<name>_24px.svg`. `SOURCE.json` lists the bundled subset; `LICENSE.txt` preserves the license. `tools/generate-material-symbols.py` normalizes the SVG viewBox and triangulates curves into `material_symbols.rs` for Bhoo native and WASM rendering. The app bundles geometry, not an icon font or network dependency.

## Bhoo branding: Koboyo ghost

The Bhoo logo and webxdc icon incorporate “A ghost under a sheet” from https://koboyo.com/icons/solid-ghost-sheet, used as application branding under https://koboyo.com/icons/license. This artwork is separately licensed and is not covered by the repository's GPL grant. The supplied terms are preserved at `codes/apps/bhoo-web/assets/branding/LICENSE-KOBOYO.txt`. The artwork is not offered in Bhoo's reusable icon API or icon picker. Changes: rasterization and a purple rounded background. No ownership or registered-trademark claim is made for the artwork.

## Android scroll physics reference

The fling spline constants and distance/duration formulas in
`codes/libs/ui/libbhoo/src/scroll.rs` are adapted into Rust from AOSP
`core/java/android/widget/OverScroller.java`, Copyright (C) 2010 The Android Open
Source Project, licensed under Apache License 2.0. The implementation uses
logical-pixel density and Bhoo's own velocity filtering and elastic edge policy.
The Apache-2.0 text is bundled in `licenses/Apache-2.0.txt` and the source archive.
Reference: https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/android/widget/OverScroller.java

## Delta Chat Android UI reference

The separate fictional Bhoo chat demo is informed by the Delta Chat Android UI
and layout sources at revision `6d85e8eadf1072170002de5deb4b84abe605e6f5`.
Upstream: https://github.com/deltachat/deltachat-android
License: GPL-3.0 (upstream LICENSE); copyright belongs to its respective authors.
The simplified XML/Rust demo is distributed under this project's GPL-3.0-only
license. It is not an official Delta Chat product, contains no real messages or
accounts, and does not connect to a messaging service. No upstream icon/logo
assets are copied; controls use the separately attributed Material Symbols.

## Android application integration fixtures

The four Bhoo offline app tests include adapted XML resources from Fossify Messages and Delta Chat (GPL-3.0), Thunderbird Android and Google Sunflower Views (Apache-2.0). Original XML, pinned revisions, license files and explicit adaptation reports are retained in `codes/apps/bhoo-web/android-app-tests/`. See `docs/TDD/libbhoo/android-app-tests.md` for source links and scope. These are unofficial test applications; upstream application logos and remote photographs are not included.

## Material Icons rating star

The filled rating star geometry in `codes/libs/ui/libbhoo/src/lib.rs` is derived from Google's [Material Icons grade SVG](https://github.com/google/material-design-icons/blob/master/src/action/grade/materialicons/24px.svg), Copyright Google LLC, licensed under Apache-2.0. The Apache-2.0 license accompanies this distribution. It is used as a vector approximation of the Android framework rating star.

The AnalogClock dial, hour hand and minute hand meshes in `codes/libs/ui/libbhoo/src/android_clock_shapes.rs` are derived from Android Open Source Project framework vector resources at revision `1cdfff555f4a21f71ccc978290e2e212e2f8b168`, licensed under Apache-2.0 as declared in `core/res/Android.bp`. The original vectors, source location and license notice are retained in `codes/libs/ui/libbhoo/assets/android-analog-clock/`; `tools/generate-android-clock-shapes.py` samples and triangulates them.

The `unicode-segmentation` crate, version 1.13.3, is used to keep grapheme clusters intact during text width estimation, TextView wrapping and truncation. Copyright (c) 2015 The Rust Project Developers; used under the MIT license. The license is bundled as `licenses/Unicode-Segmentation-MIT.txt` and in the source archive.

The `unicode-linebreak` crate, version 0.1.5, by Axel Forsman implements Unicode line break opportunities and is used under Apache-2.0. Its license is bundled as `licenses/Apache-2.0.txt`. The crate uses Unicode 15.0 line-breaking data, copyright © 1991–2022 Unicode, Inc.; the Unicode data license is bundled as `licenses/Unicode-DFS-2016.txt`. Source: https://github.com/axelf4/unicode-linebreak.

## Material 3 pull-to-refresh

The pull threshold, resistance, overshoot equation and circular pull-arrow geometry in `codes/libs/ui/libbhoo/src/pull_refresh.rs` are adapted from AndroidX Compose Material 3 `PullToRefresh.kt`, Copyright 2023 The Android Open Source Project, Apache-2.0. Source: https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/pulltorefresh/PullToRefresh.kt. The Apache-2.0 license is packaged in `licenses/Apache-2.0.txt`.

## Android ZoomControls drawables

The ZoomControls button images in `codes/libs/ui/libbhoo/assets/android-zoomcontrols` are derived from the Android framework's mdpi nine-patch PNGs, Copyright 2008 The Android Open Source Project, licensed under Apache-2.0. Source: https://github.com/aosp-mirror/platform_frameworks_base/tree/android-15.0.0_r1/core/res/res/drawable-mdpi. The generated RGBA assets strip the nine-patch metadata border; see `tools/generate-android-zoomcontrols.py`. The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

## Android RelativeLayout

The rule ordering, parent-edge behavior and gone-anchor traversal in `codes/libs/ui/libbhoo/src/relative_layout.rs` are adapted from Android 15 `RelativeLayout.java`, Copyright 2006 The Android Open Source Project, Apache-2.0. Source: https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/RelativeLayout.java. The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

## Android TableLayout and TableRow

The shared-column measurement in `codes/libs/ui/libbhoo/src/android_xml.rs` follows Android 15 TableLayout and TableRow, Copyright 2006 The Android Open Source Project, Apache-2.0. Sources: https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TableLayout.java and https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TableRow.java. The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

## Android TwoLineListItem

The XML mapping for TwoLineListItem follows Android 15 `TwoLineListItem.java`, Copyright 2006 The Android Open Source Project, Apache-2.0. Source: https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TwoLineListItem.java. The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

## Android GridLayout

The initial placement and default gap in `codes/libs/ui/libbhoo/src/android_xml.rs` refer to Android 15 `GridLayout.java` and `dimens.xml`, Copyright The Android Open Source Project, Apache-2.0. Sources: https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/GridLayout.java and https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/res/res/values/dimens.xml. The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

## Android Spinner behavior

The Spinner XML dropdown behavior is informed by Android 15 [Spinner.java](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/Spinner.java), Copyright 2006 The Android Open Source Project, Apache-2.0. The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

Stable-ID selection reconciliation is informed by Android 15 [AdapterView.java](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/AdapterView.java), Copyright 2006 The Android Open Source Project, Apache-2.0.

## AndroidX CardView content padding

MaterialCardView XML content-padding handling is informed by AndroidX CardView, Copyright 2017 The Android Open Source Project, Apache-2.0. Reference: [CardView.java](https://android.googlesource.com/platform/frameworks/support/+/4e4a7ffb298788f07a575003eba47bde163f1a67/cardview/src/main/java/androidx/cardview/widget/CardView.java). The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

Rounded Material card compat-inset calculations are informed by Material Components for Android's [MaterialCardViewHelper.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/card/MaterialCardViewHelper.java), Copyright 2018 The Android Open Source Project, Apache-2.0.

## Material Android checked-card icon

`codes/libs/ui/libbhoo/assets/material-android/ic_mtrl_checked_circle.xml` and its generated mesh in `material_card_icon.rs` are derived from Material Components for Android, Copyright 2019 The Android Open Source Project, Apache-2.0. The source is [ic_mtrl_checked_circle.xml](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/resources/res/drawable/ic_mtrl_checked_circle.xml). The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

Material card checked/dragged foreground and stroke-state choices follow Android's [m3_card_foreground_color.xml](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/card/res/color/m3_card_foreground_color.xml) and [m3_card_stroke_color.xml](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/card/res/color/m3_card_stroke_color.xml), Copyright 2021 The Android Open Source Project, Apache-2.0.

Material card default ripple-color state choices are informed by [m3_card_ripple_color.xml](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/card/res/color/m3_card_ripple_color.xml), Copyright 2021 The Android Open Source Project, Apache-2.0.

## Material bottom sheet gesture coordination

The initial downward flick threshold, projected hide distance, and nested-scroll direction and handoff in `codes/libs/ui/libbhoo/src/composition.rs` are informed by Material Components for Android `BottomSheetBehavior.java`, Copyright 2019 The Android Open Source Project, Apache-2.0. Source: https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomsheet/BottomSheetBehavior.java. The Apache-2.0 license is bundled as `licenses/Apache-2.0.txt`.

The `unicode-width` crate, version 0.2.2, is used for Unicode width-category fallback estimates. Copyright (c) 2015 The Rust Project Developers; used under the MIT license. The license is bundled as `licenses/Unicode-Width-MIT.txt`; its Unicode 17.0 data is subject to the Unicode license, bundled as `licenses/Unicode-License.txt`. Source: https://github.com/unicode-rs/unicode-width.

## Wisp remote video codec

Wisp development remote_control and the Rust remote viewer use `openh264` and
`openh264-sys2` 0.9.10 (Ralf Biedert and contributors, BSD-2-Clause),
https://github.com/ralfbiedert/openh264-rs. The bundled Cisco OpenH264 2.7 source
is Copyright (c) 2013 Cisco Systems, Inc., BSD-2-Clause; its full license and
disclaimer are preserved in
`codes/apps/wisp/docs/remote_control/licenses/OpenH264-BSD-2-Clause.txt`.
Upstream: https://github.com/cisco/openh264. This build compiles the source; it
does not download Cisco's separately licensed binary distribution.
