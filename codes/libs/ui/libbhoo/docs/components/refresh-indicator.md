# Refresh indicator

Status: Initial pull-to-refresh gesture, integrated scrolling and indicator; full nested-scroll and exact platform parity remain incomplete.

| Field | Decision |
| --- | --- |
| Category | Data and application patterns |
| Kind | Material/framework-inspired component family |
| Priority | P2 |
| Motion profile | [Continuous](../motion.md#motion-profiles) |

## Scope and variants

Pull-to-refresh feedback; touch-oriented interaction.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

The caller owns refreshing state and receives a single action when a touch pull is released beyond the threshold. The initial reusable `Frame::pull_to_refresh` takes an `at_top` flag supplied by the host. It tracks pull, arm, release and cancellation, and animates the indicator toward the refreshing or hidden state. The pull indicator uses the reference circular arc and arrow dimensions and progress geometry; busy motion is an approximation. `Frame::pull_refresh_scroll` also coordinates a Bhoo vertical viewport: downward drag moves content to the top first, then accumulates pull; an upward reversal consumes the pull first. An explicit refresh button is included in the demo.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Follow pull distance directly, animate only during visible refresh, and stop on completion.

Use the shared [continuous profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Stop rotation/morphing and show a static busy indicator with accessible status; determinate values still update.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Cancel before the threshold and confirm no refresh action or residual animation runs.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Flutter RefreshIndicator API and samples](https://api.flutter.dev/flutter/material/RefreshIndicator-class.html)

No dedicated image was verified for this entry in the reference snapshot. The linked guides provide behavior and implementation references; a Bhoo visual specification remains to be produced.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Compose Material 3 | [PullToRefresh.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/pulltorefresh/PullToRefresh.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/pulltorefresh/PullToRefresh.kt) |
| Flutter material_ui | [refresh_indicator.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/refresh_indicator.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/refresh_indicator.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Flutter Material API](https://api.flutter.dev/flutter/material/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Refresh indicator preview](http://127.0.0.1:8766/#/refresh-indicator).

Demonstrates: A scrollable list with pull-to-refresh, an explicit refresh action, visible busy feedback, timed completion and refresh count.

Remaining scope: Arbitrary nested child scroll coordination, matched fling consumption, busy indicator parity, exact transition timing and device comparison.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML `SwipeRefreshLayout` support (`RULE_SWIPE_REFRESH = 246` / `RULE_SWIPE_REFRESH_EXT = 303`, Wave 8A / Wave 8B / Wave 23A / Wave 23B)

- **XML importer ([`tools/import-android-layout.py`](../../../../tools/import-android-layout.py), Wave 8A / Wave 23B):** Lowers `androidx.swiperefreshlayout.widget.SwipeRefreshLayout` and (Wave 23B) legacy `android.support.v4.widget.SwipeRefreshLayout` to `Kind::Frame` emitting `RULE_SWIPE_REFRESH = 246`. Wave 23B adds `SWIPE_REFRESH_ATTRS` (`swipeRefreshLayoutProgressSpinnerBackgroundColor`, `swipeRefreshProgressColor`, `swipeRefreshSize`, `swipeRefreshOffsetStart`, `swipeRefreshOffsetEnd`), resolving spinner background color, color scheme (single color, comma/list, `@array/`, or `@string/`), indicator size (`default`/`large`), and start/end offsets (`-40`, `64` dp defaults) into `RULE_SWIPE_REFRESH_EXT = 303` (`"<bg_rgba>|<scheme_rgba_list>|<size>|<start_off_dp>,<end_off_dp>"`) whenever set.
- **Runtime ([`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../../src/systems/android/xml/mod.rs), Wave 8B / Wave 23A):** Independently implements `RULE_SWIPE_REFRESH = 246` and `RULE_SWIPE_REFRESH_EXT = 303` on `Kind::Frame` against the public Apache-2.0 AndroidX `SwipeRefreshLayout` contract (without vendoring AndroidX Java sources; see [licenses and attribution](../licenses-and-attribution.md)):
  - **Interactive pull-to-refresh physics:** When descendant scrollable views are at top (`offset <= 0.5`), downward pointer/touch dragging applies `0.45×` elastic resistance, cancels child clicks past `8dp` drag slop, and translates an elevated `40dp` (`default`) or `56dp` (`large`) circular spinner filled with `bg_rgba` (default M3 `surface_high`) and a rotating/sweeping arc cycling through `scheme_rgba_list` (default `primary`).
  - **Trigger & retraction:** Releasing past `end_off_dp` sets `LayoutState.refreshing[id] = true` and holds the spinning indicator at the refresh offset; releasing below the threshold or clearing `LayoutState.refreshing[id]` smoothly retracts the indicator over `0.22s` ease-out via bounded `LayoutState.swipe_refresh_motion` (`64`-entry cap), snapping immediately under `MotionPreference::Reduced` per [motion.md](../motion.md#motion-profiles).
- **Validation evidence:**
  - Rust runtime tests in [`tests/android_xml.rs`](../../tests/android_xml.rs) (`328` passing tests within `923` default / `929` all-features Rust tests across `76` suites): `swipe_refresh_rule_paints_elevated_indicator_over_content_when_refreshing` and Wave 23A `wave23a_swipe_refresh_ext_pull_physics_colors_size_offsets_retract_and_reduced_motion`.
  - Python XML importer tests in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py) (`256` passing tests, `0` diffs across `167` existing layouts): `test_wave8a_swipe_refresh_and_drawer_layout_rules_246_247` and Wave 23B `test_wave23b_swipe_refresh_ext_rule_303`.
  - Visual showcase & Gmail captures: active refreshing indicator rendered in Page-2 Light/Dark showcases (`context/android-xml-comparison/bhoo-m3-showcase-page2{,-dark}.png`) and Gmail Inbox (`gmail-xml-inbox.png`) under headless Weston + Mesa software GLES (static renders, not side-by-side Waydroid pixel-parity proof).
- **Remaining Android XML limitations:** Custom `slingshotDistance` overscroll physics, nested fling consumption across arbitrary nested scroll chains, programmatic scale-in/scale-out indicator animation (`setProgressViewOffset(scale=true)`), and Android device visual/gesture parity remain open. See [Android XML import](../android-xml.md) and [FrameLayout](android-frame-layout.md).
