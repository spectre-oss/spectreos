# Snackbar

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | Communication and feedback |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Overlay](../motion.md#motion-profiles) |

## Scope and variants

Transient message with optional action and timeout.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Own message, optional action, timeout and dismissal; define queue policy and accessible reading-time behavior.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Enter/exit with overlay motion; timeout is a deadline, not a continuous animation.

Use the shared [overlay profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Present or dismiss at final geometry immediately with no scale/translation; retain all modality and focus behavior.

### Angular comparison implementation

The example uses the reusable host to draw a 48 px high bottom overlay, inset 8 px, using inverse-surface,
inverse-on-surface and inverse-primary colors with a trailing Undo action. Its
entry scales from 0.8 while fading over 150 ms; exit fades over 75 ms. The
example dismisses six seconds after entry or when Undo is activated. Click the
archive button twice to queue a longer message and three times to also see an
action on a new line; the fourth notice includes a separate close button, and
the fifth uses right-to-left placement. Reduced
motion snaps to the final state. Hover and focus pause the deadline; a platform
recommended accessible reading-time policy remains an open production requirement.
See [Angular snackbar styles](https://github.com/angular/components/blob/main/src/material/snack-bar/snack-bar-container.scss)
and [tokens](https://github.com/angular/components/blob/main/src/material/snack-bar/_m3-snack-bar.scss).

`node tools/test-bhoo-snackbar.mjs` checks placement, light/dark colors and Undo;
`node tools/test-bhoo-snackbar-motion.mjs` compares 14 timed samples and checks
timeout and reduced motion. `node tools/test-bhoo-snackbar-queue.mjs` exercises
five queued notices, live-region DOM updates, hover pause, actions, dismissal
and mirrored control placement in the browser. These checks do
not establish screen-reader speech, every viewport or native rendering parity. Bhoo applies
opacity to individual draw commands, so overlap can differ during fades.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

### Reusable lifecycle

`bhoo::snackbar::Host` owns a FIFO of notices and reports action, dismissal, and
timeout results by ID. Short and long timeouts are 4 and 10 seconds, following
the Apache-2.0 Compose Material 3 `SnackbarHost` reference; callers can use an
indefinite or custom duration, including a platform-recommended accessibility
timeout. Call `update` on input changes and on the returned monotonic deadline,
passing whether the notice is hovered or focused. This preserves the remaining
time during interaction and does not require continuous frames. `Frame::snackbar_host`
draws the notice, schedules its deadline, pauses for pointer or action focus,
dispatches the action and animates entry/exit; reduced motion snaps immediately.
Each notice can provide a localized `dismiss_label` for an independent close
button; pressing it returns `Outcome::Dismissed`, not the action result. Focus
on either control pauses a finite timeout.
`Presentation::right_to_left` mirrors action and dismiss placement and aligns
the message to the right; this is geometry support, not bidirectional text shaping.
The caller owns the queue, presentation state, result handling and platform
accessibility integration. A non-focusable `Status` semantic carries the full
message; the playground and single-canvas browser hosts translate it into a
polite atomic live region once per visible notice. Screen-reader speech and
native accessibility announcements are not yet verified. The Angular comparison scene uses this host with a
6.15-second duration to include its 150 ms entrance.

`cargo test --manifest-path codes/libs/ui/libbhoo/Cargo.toml --test snackbar`
checks queue ordering, action and dismissal outcomes, duplicate IDs, deadlines,
pause/resume, visual draw commands, pointer activation and reduced motion.
Message lines wrap using the current Bhoo text metrics, with a 68 dp minimum
for two lines; long unbroken words break at character boundaries and the text
is clipped to its available region. The action can be placed below the message
via `Presentation::action_on_new_line`; Bhoo also uses that layout when the
inline message region would be narrower than 96 dp. Very long action labels are
clipped to the available action region. This initial layout does not yet implement
full Unicode line breaking, bidirectional shaping or scrolling for messages
taller than the viewport. Native rendering and native live-region integration
remain to be verified or implemented.

## Acceptance criteria

- Focus the action near timeout and verify reading/action time is not cut short by dismissal.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Android implementation and anatomy guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Snackbar.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/snackbar/overview)

- [Upstream anatomy / visual example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/snackbar/snackbar-overview.png)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Components for Android | [Snackbar.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/snackbar/Snackbar.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/snackbar/Snackbar.java) |
| Compose Material 3 | [Snackbar.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Snackbar.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Snackbar.kt) |
| Angular Material | [snack-bar.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/snack-bar/snack-bar.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/snack-bar/snack-bar.ts) |
| Flutter material_ui | [snack_bar.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/snack_bar.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/snack_bar.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Android: Snackbar.md](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Snackbar.md)
- [Angular Material: snack-bar/](https://github.com/angular/components/tree/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/snack-bar/)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Snackbar preview](http://127.0.0.1:8766/#/snackbar).

Demonstrates: Transient archive notice, Undo, deadline and focus/hover timeout pause.

Remaining scope: Native live announcements, screen-reader verification and recommended accessibility
timeouts, full Unicode layout, and native and browser validation
of multiline rendering and motion against upstream.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.

## Android XML `SnackbarContentLayout` support (`RULE_SNACKBAR = 265`, Wave 14C / Wave 15A / Wave 16A)

- `tools/import-android-layout.py` lowers `com.google.android.material.snackbar.SnackbarContentLayout` to horizontal `Kind::Linear` emitting `RULE_SNACKBAR = 265` (`anchor: ""`) and validating `app:animationMode` (`slide`/`fade`) and `app:maxActionInlineWidth`.
- At runtime in [`android_xml.rs`](../../src/systems/android/xml/mod.rs), `RULE_SNACKBAR` renders `4dp`-rounded `inverseSurface` container chrome, defaults child message `TextView` labels to `inverseOnSurface`, and defaults child action `Button` labels to `inversePrimary`.
- **`CoordinatorLayout` Snackbar-aware FAB dodge (Wave 16A):** When a visible bottom-docked `RULE_SNACKBAR = 265` child is present inside a `CoordinatorLayout` (`Kind::Frame`), pass 3 of `compute_coordinator_rects` shifts bottom-aligned `ExtendedFloatingActionButton` (`RULE_EXTENDED_FAB = 242`) and elevated bottom-aligned `FloatingActionButton` / `Button` / `Icon` siblings upward above the snackbar (`snackbar_height + vertical_margins + 8dp` gap) so FABs never overlap the snackbar.
- **Validation:** Rust runtime tests `wave15a_alert_dialog_and_snackbar_chrome` and `wave16a_coordinator_snackbar_aware_fab_dodge` in [`tests/android_xml.rs`](../../tests/android_xml.rs), Python importer test `test_wave14c_alert_dialog_snackbar_tab_slider_text_input_ext_chip_group_rules_264_to_269` in [`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), and static Light/Dark Mesa GLES Page-3 captures (`context/android-xml-comparison/bhoo-m3-showcase-page3.png`, `bhoo-m3-showcase-page3-dark.png`).
- **Remaining Android XML limitations:** Declarative slide/fade show-dismiss animation lifecycle, swipe-to-dismiss gesture, and automatic `maxActionInlineWidth` stacked-action reflow in XML remain open (the native Rust `bhoo::snackbar::Host` covers FIFO queueing, deadlines, and stacked actions separately). See [Android XML import](../android-xml.md).
