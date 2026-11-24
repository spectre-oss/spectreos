# Android XML ExpandableListView

Status: Host-backed groups and children with initial XML row templates; full adapter behavior and Android device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

The importer accepts ExpandableListView, including solid rectangular group and child divider drawables and divider height. A host supplies groups with stable IDs and children with IDs and selectable states. Expanded groups insert their children into a virtualized list; group and child positions map to flat positions without copying every visible row. Expansion is retained by stable group ID when adapter data changes. A configurable maximum expanded count collapses the first expanded group in adapter order, as Android's connector does. Adapter callbacks are queued in order, including the automatic collapse before the new expansion.

Rows use the Android 15 framework group indicator assets. Group and child XML templates can be set separately; their labels bind to a TextView ID. Template row heights are measured as rows become visible. Per-item text, image and template bindings follow stable group and child IDs. Linear and frame row templates can contain independently interactive checkbox, switch and button views with stable row-local state; other interactive views remain open. Fixed XML header and footer views can be supplied with optional selection. XML group and child indicators support PNG and static VectorDrawable resources, @null, and ordered selectors with expanded/empty group states or the last-child state; in Wave 21A/21B, `android:groupIndicator` additionally supports timed `<animation-list>` drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`, `"indicator:<oneshot:0|1>|<img_key_0>:<dur_ms_0>,..."`, including `<inset>` unwrapping and cyclic-reference rejection via `resolve_animation_list_drawable` + `decode_animation_list_frames`, preserving frame-0 static fallback `kind: 131`, and advancing frames via `select_animation_list_frame` using `f.time()` and clip-gated `f.wake_at()` in both LTR and RTL). XML indicator left/right and start/end bounds apply to groups and children, with child bounds inheriting the group by default. Fixed headers and footers support checkbox, switch and button children. Timed `<animation-list>` on `android:childIndicator`, `<animated-vector>`/`<animated-selector>` indicators, additional vector features, other drawable types, and theme fidelity remain open.

## State and interaction contract

A host can intercept a group click by stable ID before the default toggle. Tapping an unintercepted group toggles expansion; tapping a selectable child reports its group and child positions. Presses use stable group and child IDs, preventing a reordered row from activating a different item. Nonselectable children do not activate. Touch scrolling uses the shared list viewport. The host receives the view ID and can inspect the latest expansion, collapse or child event.

## Motion and animation

Group expansion changes rows immediately, then scrolls the children into view using measured and estimated row heights over 200 ms while keeping the group visible. Touching or scrolling interrupts the movement. Scrolling uses the shared touch fling. The host can request the same scroll when expanding a group programmatically; pending requests follow stable group IDs if the adapter is reordered. Timed `<animation-list>` `android:groupIndicator` drawables (`RULE_COMPOUND_ANIMATION_LIST = 295`) advance by frame duration using `f.time()` and clip-gated `f.wake_at()` wakeups (looping or `oneshot`-clamping; see [motion.md](../motion.md#scheduling-and-lifecycle)).

**Reduced motion:** Scrolling fling is disabled, and timed `<animation-list>` `android:groupIndicator` drawables freeze strictly on frame `0` with zero wakeup.

## Acceptance criteria

- Expand and collapse groups, activate children, and scroll a large data set.
- Verify stable-ID updates and maximum expanded-group behavior.
- Verify timed `<animation-list>` `android:groupIndicator` playback (`RULE_COMPOUND_ANIMATION_LIST = 295`) across LTR/RTL placement, clip-gated wakeups, and reduced-motion frame-0 freeze.
- Implement remaining custom adapter and indicator variants, then compare appearance and gestures with Android.

## Validation and visual status

- **Python XML importer tests ([`tools/test-android-layout.py`](../../../../tools/test-android-layout.py), `248` passing tests, `0` diffs across `167` existing layouts):** Wave 21B `test_wave21b_compound_animation_list_rule_295` verifies `RULE_COMPOUND_ANIMATION_LIST = 295` (`"indicator:..."`) emission alongside frame-0 static fallback `kind: 131` on `ExpandableListView` `android:groupIndicator` (both direct PNG and `<inset>`-wrapped `<animation-list>`) and cyclic-reference rejection.
- **Rust runtime tests ([`tests/android_xml.rs`](../../tests/android_xml.rs), `320` tests within `915` default / `921` all-features Rust tests across `76` suites):** Wave 21A `wave21a_compound_animation_list_all_slots_rtl_clip_and_reduced_motion` verifies timed `indicator` frame playback, `f.wake_at()` scheduling, LTR left / RTL right placement, offscreen clip wakeup suppression, and reduced-motion frame-0 freeze. Android device visual and gesture parity remain open.

## References

- [Android 15 ExpandableListView](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/ExpandableListView.java)
- [Android 15 ExpandableListConnector](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/ExpandableListConnector.java)
- [Android 15 BaseExpandableListAdapter](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/BaseExpandableListAdapter.java)
