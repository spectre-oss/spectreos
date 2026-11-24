# Reference Source Licenses and Attribution

Review scope: the selected upstream source snapshots and the Material Web documentation snapshot used by Bhoo. Checked 2026-11-08. This is a source-provenance review, not a complete license audit of SpectreOS, Cargo dependencies, or upstream transitive dependencies.

## Findings

| Reference | Project-level license | Evidence retained |
| --- | --- | --- |
| Material Web | Apache-2.0 | [Original LICENSE](reference-licenses/material-web/LICENSE) and original source headers |
| Material Components for Android | Apache-2.0 | [Original LICENSE](reference-licenses/material-android/LICENSE) and original source headers |
| Compose Material 3 / AndroidX | Apache-2.0 | [Original LICENSE.txt](reference-licenses/compose-material3/LICENSE.txt) and Apache notices in source files |
| Angular Material | MIT | [Original LICENSE](reference-licenses/angular-material/LICENSE), including the Google LLC copyright notice |
| Flutter material_ui | BSD-3-Clause | [Repository LICENSE](reference-licenses/flutter-material/LICENSE), [package LICENSE](reference-licenses/flutter-material/packages/material_ui/LICENSE), and original source headers |

The [source inventory](reference-implementations.md) records the exact commit for each finding. The [machine-readable reference index](reference-sources.json) lists the legal files actually found and downloaded, including additional notices where present. Original texts are copied without rewriting; their exact copyright notices are authoritative.

These project-level findings are the starting point for source reuse. They are not a claim that every file, asset, dependency, font or design kit associated with each project has identical terms. Review the particular files being ported and their relevant notices. Fonts, Figma kits and unrelated media were not downloaded as part of these source snapshots.

The legal-file collection also retained AndroidX's `license-baseline.txt` as source-review metadata. It lists build dependency coordinates and is not an additional license grant or a replacement for dependency notices. No root `NOTICE` file was present in the inspected repository-root inventories; any later file selection still needs its own applicable-notice check.

A header scan of the selected source files found Apache notices throughout the Material Android and Compose selections, Apache SPDX notices in most Material Web files, MIT-style headers in many Angular files, and BSD-style headers in the Flutter selection. Some generated/style files omit a license name in their initial header; retain the project license and review those files individually before copying them. The scan is supporting evidence, not a license classifier for every file.

## What is currently included in SpectreOS

The five downloaded implementation snapshots live under ignored `context/reference-sources`. They remain research material rather than vendored runtime libraries. The subsequent control implementation adapts standard AndroidX effects token values in `motion.rs`; its origin, retained notice, changes and license are recorded in [implementation status](implementation-progress.md#source-references-and-reuse-record) and the repository third-party notices. No upstream GUI framework dependency was added to the Rust core. The optional Angular browser reference described below is a separate demo runtime.

The Material Web documentation and illustrations already copied into `codes/libs/ui/libbhoo/docs/material-design/upstream` are distributed repository content. Their [upstream license](material-design/upstream/LICENSE), source revision and original files are retained. The repository-level [third-party notices](../../../../../THIRD_PARTY_NOTICES.md) identify this snapshot separately from SpectreOS's own GPL license.

Copies of the five reviewed upstream license files are also retained in this TDD so the review remains readable without the ignored source downloads. Merely linking to a screenshot does not add that image to Bhoo's application assets.

## Conditions when adapting or distributing source

### Apache-2.0 references

Retain the applicable copyright, patent, trademark and attribution notices, provide the Apache license text with redistributed work, and add prominent notices to files changed from upstream. If the relevant upstream distribution includes a `NOTICE` file, reproduce its applicable attribution notices in the form permitted by Apache section 4. Do not remove individual-file notices because a repository has a top-level license.

Apache-2.0 includes a conditional patent grant and does not grant general trademark rights. Naming a reference toolkit does not imply its authors endorse Bhoo. See the [official Apache license, sections 3, 4 and 6](https://www.apache.org/licenses/LICENSE-2.0).

### MIT reference

Include the original copyright and permission notice in copies or substantial portions of Angular Material code. Preserve its warranty disclaimer as part of the license text. A Rust port of substantial implementation logic still needs source provenance and the relevant notice; changing the programming language is not a basis for dropping it.

### BSD-3-Clause reference

For copied Flutter source, retain the copyright notice, conditions and disclaimer. Binary redistribution must reproduce those materials in the accompanying documentation or other materials. Do not use the upstream names to endorse or promote a derived product without the permission specified by the license. Preserve both package and repository notices that apply to the selected files.

### Relationship to Bhoo's GPL-3.0-only license

Bhoo currently declares `GPL-3.0-only`. Apache's official [GPL compatibility explanation](https://apache.org/licenses/GPL-compatibility.html) confirms Apache-2.0 source can be incorporated into GPLv3 projects under the applicable conditions. The permissive MIT and BSD-3-Clause notices must likewise remain with material reused under their terms. A project-level GPL declaration does not erase upstream license or attribution obligations.

Do not describe untouched upstream files as exclusively authored by SpectreOS. Keep upstream license texts and applicable source headers, mark our modifications, and document the combined project's licensing separately from the provenance of individual pieces.

## Attribution record for an actual port

Before committing adapted code, record:

| Field | Required information |
| --- | --- |
| Origin | Repository URL, exact commit, and source path(s) |
| License evidence | Applicable license file, individual-file notices, and relevant NOTICE entries |
| Destination | Bhoo path(s) receiving the implementation or port |
| Changes | A concise description of the translation or modifications |
| Distribution | Where required license and attribution text accompanies source and binaries |

Update the repository notices and the component's reference section with this record. Preserve notices next to vendored source or in a clearly referenced attribution file as the applicable license permits. An inspiration link is useful provenance but does not replace required copyright/license text when implementation expression is reused.

## Limits and future checks

The downloads intentionally omit full build trees and dependency installations, so this review does not resolve the licenses of dependencies that might be needed to build the upstream frameworks. Directory selection also cannot establish that all optional assets or generated code in the complete repositories are covered.

Before importing additional source, assets, icons or fonts, inspect their own notices and update this record. The existing image-reference documents label related visuals and should not be read as permission to bundle those visuals into a shipping application.

## Implemented motion and derived shape data

The runtime now independently implements Material Web ripple/curve/menu/dialog behavior and adapts AndroidX progress/loading tokens and seven loading shape recipes. `loading_shapes.rs` contains generated derived curve data; `tools/BhooShapes.java` preserves Apache-2.0/AOSP attribution for the adapted recipes. The root notice records source paths, pinned revisions, generation libraries and modifications. The small JVM tools are generation-only dependencies, not shipped runtime libraries. See [effects provenance](effects-implementation.md) and the updated [third-party notices](../../../../../THIRD_PARTY_NOTICES.md).

## Optional Angular comparison reference

The [comparison lab](angular-comparison.md) now distributes an actual Angular reference bundle, separate from libbhoo and the older research snapshots. Material/CDK 22.2.6 and Angular 22.2.5 are MIT; RxJS 7.8.2 is Apache-2.0 and tslib 2.9.1 is 0BSD. The lockfile pins the complete build graph. The builder inventories every bundled package and retains its original license/notice/copyright files, including tslib's separate Microsoft copyright notice, under `site/angular/THIRD_PARTY_LICENSES.txt`. Keep that file, `versions.json`, and esbuild's legal-comments sidecar with any distribution of the comparison bundle. Independently written fixture/host source remains GPL-3.0-only and is included in the corresponding-source download. Native Rust and the original standalone playground do not load Angular.

### Avatar example references

MUI's [Avatar.js](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Avatar/Avatar.js) in `context/material-ui` is MIT licensed. It informed avatar image cover, fallback order and shape variants; Bhoo implements these independently. The fallback icon is Google's [Material Symbols Outlined person](https://github.com/google/material-design-icons/blob/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/symbols/web/person/materialsymbolsoutlined/person_24px.svg), Apache-2.0, included with its source and license in `codes/libs/ui/libbhoo/assets/material-symbols`. The sample image is original procedural Bhoo code.

### Material Symbols Outlined (40 bundled icons)

The bundled Material Symbols Outlined icon set was expanded from 25 to 40 icons by adding 15 SVGs (`delete`, `mark_email_unread`, `mail`, `chat_bubble`, `videocam`, `edit`, `sell`, `attach_file`, `send`, `reply`, `more_horiz`, `chevron_left`, `image`, `sentiment_satisfied`, and `description`) from the same pinned Google repository revision:

| Field | Record |
| --- | --- |
| Origin | [`google/material-design-icons`](https://github.com/google/material-design-icons/tree/10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7/symbols/web) at commit `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`, paths `symbols/web/<name>/materialsymbolsoutlined/<name>_24px.svg` (40 icons listed in `codes/libs/ui/libbhoo/assets/material-symbols/SOURCE.json`) |
| License evidence | Apache-2.0, Copyright Google LLC; original license text preserved at `codes/libs/ui/libbhoo/assets/material-symbols/LICENSE.txt` |
| Destination | `codes/libs/ui/libbhoo/assets/material-symbols/` (40 SVG originals) and `codes/libs/ui/libbhoo/src/icons/material_symbols.rs` (triangulated Rust meshes exposed via `MaterialIcon` in `codes/libs/ui/libbhoo/src/systems/material/mod.rs`) |
| Changes | `tools/generate-material-symbols.py` normalizes each 24×24 SVG viewBox and samples/triangulates vector contours into static Rust `ShapeData` geometry; no icon font or network dependency is required |
| Distribution | Apache-2.0 license and attribution accompany source and binary/webxdc distributions via `codes/libs/ui/libbhoo/assets/material-symbols/LICENSE.txt` and [THIRD_PARTY_NOTICES.md](../../../../../THIRD_PARTY_NOTICES.md) |

### AndroidX ConstraintLayout reference

Bhoo's initial `Kind::ConstraintLayout` axis solver in `codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs` is an independent Rust implementation informed by the public AndroidX ConstraintLayout API contract (upstream [`androidx/constraintlayout`](https://github.com/androidx/constraintlayout), Apache-2.0) and Bhoo's relative-layout axis structure informed by Android 15 `RelativeLayout.java` (`context/reference-sources/android-framework/RelativeLayout.java`, Apache-2.0). No upstream `ConstraintLayout.java` source snapshot is currently vendored or copied into `libbhoo`. See [ConstraintLayout](components/android-constraint-layout.md).

### Android framework progress reference

Android 15 `TextView.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for text appearance and direct attribute precedence. Bhoo resolves its supported local styles independently.

Android 15 `ToggleButton.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for state-label selection and disabled indicator opacity. Bhoo's rendering is independently implemented.

Android 15 `Switch.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for independent thumb and track tint attributes. Bhoo applies these colors to its own switch geometry.

Android 15 `CompoundButton.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for checkbox button tint and state. Bhoo applies colors independently to its own rendering.

Android 15 `CheckedTextView.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for checked state, check-mark gravity and tint. Bhoo implements this subset independently with its existing Material icon.

Android 15 `RatingBar.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for fractional rating secondary-progress behavior. Bhoo renders and colors its own star vector independently.

Android 15 `AbsSeekBar.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for SeekBar thumb tint and state behavior. Bhoo implements the tint application independently.

The Android 15 `ProgressBar.java`, `RadioGroup.java`, `ToggleButton.java`, `FrameLayout.java`, `LinearLayout.java`, `ScrollView.java` and `HorizontalScrollView.java` source snapshots in `context/reference-sources/android-framework/` is an Apache-2.0 reference with its original copyright and license header. The Bhoo secondary-progress rendering is an independent implementation; Android drawable and theme fidelity remains unverified.

### Rating star vector

The filled rating star in `libbhoo` is derived from Google's [Material Icons grade SVG](https://github.com/google/material-design-icons/blob/master/src/action/grade/materialicons/24px.svg), licensed Apache-2.0. Android's `ratingbar_material.xml` references a filled star bitmap; Bhoo uses this vector approximation and has not yet established pixel parity.

### Android density selection reference

The Android 15 `ResourceTypes.cpp` snapshot in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for density-bucket selection. Bhoo supports a subset of Android resource qualifiers and performs selection at import time.

Android 15 `ListView.java` and `AbsListView.java` in `context/reference-sources/android-framework/` retain their Apache-2.0 headers; they are used as behavior references for the independently implemented Bhoo XML ListView.

Android 15 `simple_list_item_1.xml`, `themes_material.xml`, `styles_material.xml` and `dimens_material.xml` snapshots under `context/reference-sources/android-framework/res/` retain Apache-2.0 source headers and were consulted for ListView row measurements.

Android 15 `GridView.java` under `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for column and spacing calculations.

Android 15 `AutoCompleteTextView.java`, `MultiAutoCompleteTextView.java` and `ArrayAdapter.java` in `context/reference-sources/android-framework/` retain their Apache-2.0 headers and were consulted for filtering and popup behavior.

Android 15 `ActionMenuView.java` and `ActionMenuItemView.java` in `context/reference-sources/android-framework/` retain their Apache-2.0 headers and were consulted for inline menu cells and icon-and-text visibility.

Android 15 `StackView.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for initial stack perspective and swipe thresholds.

Android 15 `CalendarView.java`, `CalendarViewMaterialDelegate.java`, `DayPickerView.java`, `SimpleMonthView.java`, and the internal `ViewPager.java` under `context/reference-sources/android-framework/` retain their Apache-2.0 headers and are consulted for the CalendarView range, month paging, and day-grid behavior.

The Android 15 calendar layout and dimension resources under `context/reference-sources/android-framework/res/` retain their Apache-2.0 headers and were consulted for CalendarView geometry.

Android 15 `DatePicker.java`, `DatePickerCalendarDelegate.java`, `DatePickerSpinnerDelegate.java`, `YearPickerView.java` and the date picker layout resources in `context/reference-sources/android-framework/` retain their Apache-2.0 headers and were consulted for the DatePicker calendar and year selection.

Android 15 DatePicker fade resources under `context/reference-sources/android-framework/res/anim/` retain their Apache-2.0 headers and informed the day/year transition.

Android 15 `NumberPicker.java` under `context/reference-sources/android-framework/` retains its Apache-2.0 header and informs the NumberPicker range, wrapping, displayed values and spinner-mode work.

The Android 15 `date_picker_legacy.xml` resource in `context/reference-sources/android-framework/res/layout/` retains its Apache-2.0 header and informed the combined spinner/calendar arrangement.

Android 15 `TimePicker.java`, `TimePickerSpinnerDelegate.java`, `TimePickerClockDelegate.java` and `res/layout/time_picker_legacy.xml` in `context/reference-sources/android-framework/` retain their Apache-2.0 headers and informed TimePicker behavior.

Android 15 `RadialTimePickerView.java` under `context/reference-sources/android-framework/` retains its Apache-2.0 header and informed radial hour and minute selection.

Android 15 `TextInputTimePickerView.java`, `time_picker_material.xml`, `btn_keyboard_key_material.xml` and `btn_clock_material.xml` under `context/reference-sources/android-framework/` retain their Apache-2.0 headers and informed the clock/keyboard mode.

The TimePicker keyboard and clock icons are generated from the Android 15 Apache-2.0 vector assets in `codes/libs/ui/libbhoo/assets/android-time-picker/`; their original license and notice are bundled beside them.

Android 15 `SearchView.java` and `res/layout/search_view.xml` under `context/reference-sources/android-framework/` retain their Apache-2.0 headers and informed the SearchView interaction model.

The SearchView search, clear and submit icons are generated from the Android 15 Apache-2.0 vector assets in `codes/libs/ui/libbhoo/assets/android-search-view/`; their original license and notice are bundled beside them.

Android 15 `ExpandableListView.java`, `ExpandableListConnector.java`, `BaseExpandableListAdapter.java` and `res/drawable/expander_group_material.xml` under `context/reference-sources/android-framework/` retain their Apache-2.0 headers and informed expandable list behavior.

Android 15 framework `expander_open_mtrl_alpha.9.png` and `expander_close_mtrl_alpha.9.png` in `codes/libs/ui/libbhoo/assets/android-expandable-list/` are Android Open Source Project assets under Apache-2.0; see the adjacent `SOURCE.md` for provenance.

Android 15 framework `Toolbar.java` in `context/reference-sources/android-framework/` is from the Android Open Source Project under Apache-2.0. The `toolbar_back.png` sample icon is from [Google Material Design Icons](https://github.com/google/material-design-icons), Apache-2.0.

Android 15 `MenuInflater.java` in `context/reference-sources/android-framework/` is from the Android Open Source Project under Apache-2.0.

Android 15 `MenuBuilder.java` in `context/reference-sources/android-framework/` is from the Android Open Source Project under Apache-2.0.

Android 15 `MenuItemImpl.java` and `ListMenuItemView.java` in `context/reference-sources/android-framework/` are from the Android Open Source Project under Apache-2.0.

Android 15 `ActionMenuPresenter.java` in `context/reference-sources/android-framework/` is from the Android Open Source Project under Apache-2.0.

The `toolbar_search.png` sample icon is from [Google Material Design Icons](https://github.com/google/material-design-icons), Apache-2.0.

The path in `toolbar_vector_search.xml` is from [Google Material Design Icons](https://github.com/google/material-design-icons/blob/master/src/action/search/materialicons/24px.svg), Apache-2.0.

Android 15 `VectorDrawable.java` and `VectorDrawable.cpp` in `context/reference-sources/android-framework/` are from the Android Open Source Project under Apache-2.0.

Android 15 `AdapterViewFlipper.java` and `AdapterViewAnimator.java` in `context/reference-sources/android-framework/` are from the Android Open Source Project under Apache-2.0.

Android 15 `QuickContactBadge.java` in `context/reference-sources/android-framework/` is from the Android Open Source Project under Apache-2.0.

Android 15 QuickContactBadge default avatar and light/dark overlay nine-patches in `codes/libs/ui/libbhoo/assets/android-quick-contact/` derive from Android Open Source Project framework resources under Apache-2.0. Original source images and selectors are retained under `context/reference-sources/android-framework/res/`.

Android 15 `MediaController.java` and `media_controller.xml` in `context/reference-sources/android-framework/` are from the Android Open Source Project under Apache-2.0.

Android 15 `ic_media_play`, `ic_media_pause`, `ic_media_rew`, `ic_media_ff`, `ic_media_previous` and `ic_media_next` PNGs in `codes/apps/bhoo-web/android-examples/res/drawable/` derive from Android Open Source Project framework resources under Apache-2.0; original files are retained in `context/reference-sources/android-framework/res/drawable-mdpi/`.

Android 15 `VideoView.java` in `context/reference-sources/android-framework/` is from the Android Open Source Project under Apache-2.0.

The silent `bhoo-test-video.webm` is a locally generated FFmpeg test pattern, not third-party footage. Its generation command is recorded in `tools/generate-bhoo-test-video.py`.

Android 15 `SlidingDrawer.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and was consulted for initial drawer behavior.

Android 15 `Gallery.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and informed the initial Gallery interaction.

Android 15 `Scroller.java` in `context/reference-sources/android-framework/` retains its Apache-2.0 header and informed Gallery fling distance and duration.

Android 15 `TabWidget.java` and `TabHost.java` in `context/reference-sources/android-framework/` retain their Apache-2.0 headers and were consulted for TabWidget and TabHost behavior.

Android 15 `DialerFilter.java` and `DialerKeyListener.java` in `context/reference-sources/android-framework/` retain their Apache-2.0 headers and informed the initial filter state.

## Text width dependency

`unicode-width` 0.2.2 supplies Unicode 17.0 width categories for the text-width fallback. It is used under the MIT option; its license and the Unicode data license are in [reference-licenses/unicode-width](reference-licenses/unicode-width). See [third-party notices](../../../../../THIRD_PARTY_NOTICES.md). Terminal width categories are only a heuristic for proportional fonts.

## GLES text shaping, rasterization, and MSDF icon generation

Under the `gles` feature, `libbhoo` uses pure-Rust crates for text shaping and glyph rasterization:
- [`harfrust`](https://github.com/harfbuzz/harfrust) (`0.16`, MIT / Apache-2.0): OpenType GSUB/GPOS shaping (kerning, ligatures, Persian/Arabic contextual joining).
- [`skrifa`](https://github.com/googlefonts/fontations) (`0.50`, MIT / Apache-2.0): TrueType/CFF outline extraction and smooth vertical hinting.
- [`zeno`](https://github.com/dfrg/zeno) (`0.3`, MIT / Apache-2.0): antialiased coverage mask rasterization with 4x horizontal subpixel phases.
- [`unicode-bidi`](https://github.com/servo/unicode-bidi) (`0.3`, MIT / Apache-2.0): UAX #9 bidirectional visual run ordering.

The 68 Material Symbols Outlined system icons (`codes/libs/ui/libbhoo/assets/material-system/*_24px.svg`, Apache-2.0, pinned rev `10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7`) are compiled offline into `codes/libs/ui/libbhoo/assets/material-system/icons.msdf` by `tools/msdf-icon-gen` using [`fdsm`](https://crates.io/crates/fdsm) (`0.8`, MIT / Apache-2.0; build-time generator only, not linked into runtime binaries).
