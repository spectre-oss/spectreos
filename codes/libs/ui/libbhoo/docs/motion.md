# Motion and Animation Design

Status: initial effects engine implemented. The [implementation record](implementation-progress.md#motion-implementation) describes the current analytic critical-damping solver, bounded property store, configured reduced-motion behavior and tooltip deadline scheduling. The Standard spatial spring (damping 0.9, stiffness 700) is now used by experimental disclosure and overlay compositions. Standard/Expressive spatial schemes, timed menu/dialog choreography, radial ink, generated loading morphs, Android 12/13 `EdgeStretch` overscroll, GPU-scaled collapsing toolbar titles, and touch-hold ripple cancellation are now implemented; see [the effects implementation](effects-implementation.md). System preference adapters and full component qualification remain open.

The sections below remain the wider design contract. In particular, the duration table is an earlier proposal, not the current runtime configuration: implemented effects use AndroidX standard critical-damping parameters instead. Current tests cover deterministic sampling and lifecycle behavior, not complete visual parity with Material.

## Goals

Motion should explain feedback, state changes, spatial relationships and ongoing work. Input and logical application state remain responsive while visuals transition. Idle and settled interfaces return to waiting; decorative motion must not create a permanent render loop.

This design is inspired by Material but defines Bhoo's own proposed behavior. Current [Material motion guidance](https://m3.material.io/styles/motion) includes expressive physics-based motion. The [Android motion implementation guide](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/theming/Motion.md) is a useful implementation reference. The initial timing proposals below are not copied Material tokens and do not establish Material conformance.

## Motion profiles

Each [component specification](components/README.md) names one primary profile. A complex component may use another profile for a child: a split button uses feedback on the trigger and overlay motion on its menu.

| Profile | Intended behavior | Reduced-motion behavior |
| --- | --- | --- |
| Feedback | Short color, opacity or state-layer changes; fixed layout | Snap to the final appearance; replace spatial effects with static feedback |
| Selection | Interpolate a selected indicator or toggle geometry within bounds | Snap indicator to the selected state |
| Layout | Deliberate expansion, collapse or visible keyed reflow | Apply final layout immediately while preserving focus and identity |
| Overlay | Coordinated surface/scrim entrance and exit near an anchor or edge | Show/hide at final geometry immediately |
| Continuous | Visible busy-state rotation/morphing or measured progress updates | Static busy treatment; actual progress values still change |
| Direct manipulation | Position/value follows the pointer, wheel or keyboard without delayed trailing | Preserve direct control; disable optional decorative/inertial travel |
| Static | No intrinsic motion or animation clock | Same static behavior |

Avoid scale/position animation for every hover, per-character typing animations, automatic carousel playback, and decorative looping on empty/error states by default. Components may intentionally have no animation.

## Initial timing proposals

Tokens are semantic planning names; none currently exist in the Rust API. Validate these desktop starting values using real interactions and hardware before accepting them as stable defaults.

| Proposed token | Initial value | Use |
| --- | --- | --- |
| `motion.feedback` | 100 ms | Hover/press color and state-layer feedback |
| `motion.selection` | 160 ms | Check, switch and selected-indicator transitions |
| `motion.enter` | 180 ms | Small overlay appearance |
| `motion.exit` | 120 ms | Small overlay dismissal |
| `motion.layout` | 220 ms | Deliberate expansion or container changes |

Use smooth monotonic interpolation for state changes; an initial candidate is smoothstep, `3t² - 2t³`, on clamped normalized elapsed time. The runtime now uses pinned AndroidX StandardMotionTokens; this table remains an earlier proposal. If a spring model is added, define its units, bounded-value policy, settling thresholds, interruption rules and integration stability before assigning it to components.

Tooltip reveal delays, snackbar lifetimes and input debouncing are deadlines, not these animation durations. They need their own semantic settings and accessibility policy. Busy indicator cycle lengths remain component-specific work; they should not inherit a short feedback token accidentally.

## Proposed state model

Store transient motion state separately from application values, keyed by a stable widget ID and property key. A record needs the current presentation value, target, interpolation model, timestamps and lifecycle identity. Springs additionally need velocity. Final data types and public Rust signatures are undecided.

Application state commits immediately. A selected radio button becomes selected logically before the indicator finishes changing. Pointer hit testing and accessibility must describe the active logical control, not a stale or decorative intermediate surface.

For geometry-changing transitions, explicitly define hit testing: direct manipulation uses presented geometry; exiting overlays stop accepting fresh input immediately; layout changes must keep interaction bounds consistent with the visible content. Do not use visual interpolation to hide an inconsistent layout/input model.

## Interruption and retargeting

- Sample the current presentation value at the event timestamp before replacing a target. Start the new transition there instead of replaying from the old endpoint.
- With springs, preserve meaningful velocity unless a bounded property or discontinuous state change requires resetting it.
- Cancel pointer feedback on cancellation, focus loss or disabled/removal events according to the component contract.
- Discrete logical changes, callbacks and accessibility announcements happen once. Reversing a visual transition must not emit another action.
- Deleting a widget cancels its motion records. Reusing an ID for unrelated content must not inherit old animation state.
- Finishing a transition snaps to the exact target and removes it from active scheduling. Numeric tolerances must not leave a perpetual almost-settled state.

## Scheduling and lifecycle

Use one window-owned scheduler with a monotonic clock and elapsed-time sampling. Avoid frame-count-based progress and per-widget polling timers. A planned frame sequence is:

1. Process input and update logical targets.
2. Sample active motion at the current time.
3. Build/layout/paint affected visible content and present.
4. Request another paced redraw only while visible motion remains active.
5. Otherwise wait until an input event or the earliest semantic deadline.

The current runner uses `ControlFlow::Wait`, Wayland redraw notifications and EGL swap interval one. A future scheduler must extend this with earliest-deadline wakeups, background update notifications, and measured fallback pacing where necessary. It must not assume requesting redraw alone proves correct frame pacing.

When a window is suspended or content is hidden, pause continuous effects and discard obsolete one-shot motion. Visibility information must come from the platform/layout integration; it is not currently wired into Bhoo. On resume, avoid replaying a backlog or simulating every missed tick. Reconcile against current logical state and cap any solver timestep if integration is used.

Virtualized rows and tiles must not keep animation records for every offscreen item. Preserve meaningful keyed selection state, clear recycled presentation state, and bound any animation cache. Pending tooltip/snackbar deadlines need lifecycle tokens so removed/replaced content cannot reappear through a stale timer.

## Reduced motion and accessibility

Provide an application-level motion preference first, with a future adapter for desktop preferences. Do not assume there is one universal Wayland reduced-motion protocol. The system-setting integration remains an open decision.

Changing the preference at runtime should immediately reconcile active transitions to their reduced-motion presentation and stop continuous decorative effects. Busy state remains visible and semantically available even when its spinner stops moving. Animation duration zero must work without division-by-zero or an extra looping frame.

Opacity zero is not enough to hide a dismissed interactive surface: input routing and semantic visibility must change too. Retain focus and content through legitimate presentation transitions, restore focus when closing, and provide stable announcements without narrating every animation tick. Never make a completed animation a prerequisite for an action, dismissal, or assistive-technology update.

## Rendering and performance

Prefer color, opacity and bounded transforms when they express the change correctly. Layout animation is allowed for meaningful expansion/reflow, but it must not rebuild an entire large model each tick. Cache text shaping and rasterization across purely visual transitions. Avoid rerasterizing fonts for decorative scaling.

The current renderer rebuilds the full requested frame and uploads one vertex stream; animation does not imply damage tracking or compositor-managed transforms. Benchmark this actual path before promising cheap animation. Measure CPU build cost, driver submission, GPU/presentation behavior, allocations and frame-time percentiles separately.

Do not add an external animation crate until a prototype demonstrates the required interpolation and scheduling behavior. If one is selected, document why it is needed and retain the same ownership, cancellation and reduced-motion contracts.

## Validation requirements

Use an injectable clock for future animation tests. Assert initial, intermediate and exact settled values at timestamps without real sleeps. Test retargeting/reversal, long pauses, zero duration, removed/disabled owners, runtime preference changes and pending-deadline cancellation.

For interactive controls, assert callbacks and semantic values independently of visual completion. For overlays, test focus restoration and input rejection during exit. For virtualization, test recycled IDs and bounded active-state counts. For continuous effects, test visibility changes and completion returning the runner to waiting.

Add visual/framebuffer checks for representative intermediate and final frames at different scales when motion exists. Existing Weston tests exercise static presentation and scaling; they do not verify animation, pacing or idle CPU consumption.

## Implementation order

1. Deterministic clock, monotonic interpolation, property identities and lifecycle cleanup.
2. Shared preference/token resolution and reduced-motion paths.
3. Small button/state-layer feedback prototype, with scheduling and settle-to-idle tests.
4. Checkbox/switch selection and anchored overlay transitions.
5. Deadline wakeups for tooltips and transient feedback.
6. Layout/scroll motion and optional spring experiments, after profiling.

Every stage requires code and validation before its component documents can change from planned to implemented.

## Browser composition baseline

The [all-family preview matrix](demo-coverage.md) records actual behavior. Effects use critical damping (1.0, stiffness 1600); bounded geometry uses Standard spatial damping 0.9, stiffness 700. The analytic solver supports these two profiles with deterministic sampling and velocity-preserving retargeting. Geometry is clamped to normalized bounds. Reduced motion snaps both profiles. This is the Compose Standard scheme, not a reproduction of Material Web’s duration-based menu/dialog choreography or the Expressive loading polygon morph sequence.

The [effect implementation record](effects-implementation.md) supersedes earlier statements that radial ink, Expressive schemes, loading shape geometry or Web overlay choreography were unimplemented. Earlier proposed timing tables remain historical design notes, not the selected runtime parameters.

## Android 12/13 scroll stretch, collapsing title, and touch-hold ripple cancel

- **EdgeEffect stretch overscroll (`EdgeStretch`)**: Implemented in [`codes/libs/ui/libbhoo/src/scroll.rs`](../src/scroll.rs) (`EdgeStretch`, `Frame::scroll_stretch`, `Frame::with_edge_stretch`) using AOSP `EdgeEffect.java` (`TYPE_STRETCH`, Apache-2.0; pinned local reference `context/reference-sources/android-scroll/EdgeEffect.java`) constants (`NATURAL_FREQUENCY = 24.657`, `DAMPING_RATIO = 0.98`, `LINEAR_STRETCH_INTENSITY = 0.016`, `EXP_STRETCH_INTENSITY = 0.016`, `SCROLL_DIST_AFFECTED_BY_EXP_STRETCH = 0.33`). Pulling past start/end bounds stretches content anchored at the active edge via `push_transform`; reverse pull consumes stretch before scrolling; release runs the analytic damped harmonic oscillator without crossing zero; fling impact calls `absorb` for velocity-proportional bounce; reduced motion snaps immediately to rest. Wired into `ScrollPhysics` and [`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../src/systems/android/xml/mod.rs) (`ScrollView`, `HorizontalScrollView`, `RecyclerView`).
- **CollapsingToolbarLayout GPU title scale**: `CollapsingTitleMode::Scale` in [`codes/libs/ui/libbhoo/src/systems/android/xml/mod.rs`](../src/systems/android/xml/mod.rs) interpolates active scroll frames (`0.001 < collapse_t < 0.999`) by GPU-scaling the expanded single-line title (`push_transform` scale `title_sp / expanded_sp`) matching AOSP `CollapsingTextHelper` without discrete glyph-size stepping, while keeping static `0.0`/`1.0` endpoints bit-identical. In Wisp Launcher Settings ([`codes/apps/wisp/src/settings.rs`](../../../../apps/wisp/src/settings.rs)), sub-pages collapse smoothly from `136dp` to `56dp` (`32sp` to `20sp` diagonally gliding title) while splitting top vs. bottom `EdgeStretch` so the pinned toolbar stays anchored during bottom overscroll.
- **Touch-hold ripple + drag-cancel (`Frame::cancel_pointer_input`)**: Added `Frame::cancel_pointer_input` in [`codes/libs/ui/libbhoo/src/systems/material/mod.rs`](../src/systems/material/mod.rs) and wired it into scrollable parents (`RecyclerView`, `ScrollView`, and launcher scroll containers). When a parent scroll/drag intercepts after a pointer hold, `active` is cleared and `input.cancelled` is set so click callbacks never fire, pending touch ripples (`< 150ms`) are cancelled before flashing, and already-bloomed ripples smoothly fade out (`FADE_OUT = 375ms` in [`codes/libs/ui/libbhoo/src/ripple.rs`](../src/ripple.rs)) while attached to the scrolling item.
- **Launcher System UI choreography** ([`codes/apps/wisp/src/lib.rs`](../../../../apps/wisp/src/lib.rs)):
  - **`OverviewGlide`**: Velocity-matched C1 cubic Hermite deceleration (`decel_rate = 7.2`, duration clamped `0.28..0.72s`, `0.16s` for `< 3dp` micro-settle, monotone tangent clamp `2.55 * abs_d` forward / `0.45 * abs_d` settle-back) paired with horizontal `EdgeStretch`, letting short slow drags settle back to the current card, normal swipes advance one card, and fast flicks coast smoothly across multiple cards. Snaps directly under reduced motion.
  - **Quick Settings tile editor circular reveal**: Radial wave reveal (`280ms` open / `240ms` close) anchored at the edit pencil button origin (`qs_edit_origin`) with per-element radial distance masking.
  - **Notification swipe-to-dismiss & shade close**: Horizontal card detachment with rounded corners, neighboring card top-corner rounding, linear fade, and `dismiss_anim` lower-card height collapse (springing back on cancelled swipes). Shade close (`260ms`) slides and fades notifications upward first, tucks QS row 2 beneath row 1, and fades the full-height backdrop last.
  - **Edge-back indicator**: `render_edge_back_indicator` renders a bare folding `|` ↔ `<` / `>` chevron floating `~58dp` above the finger with adaptive contrast, unfolding proportionally with inward drag and folding back on cancellation.
- **Validation**: Covered by `scroll::tests::edge_stretch_pull_release_and_absorb_spring_recoil` (`libbhoo --lib`: 30/30 passed, 0 warnings) and `launcher_demo::drag_tests` (`settings_about_phone_top_and_bottom_edge_stretch_overscroll`, `settings_one_second_hold_then_drag_down_shows_ripple_without_selecting`, `overview_variable_speed_gradual_deceleration_short_settles_back_fast_coasts_multicard`; 14/14 passed), plus 60fps real-phone filmstrip references (`context/system-ui-flow/animations/strip-v5-*.png`) and native Mesa GLES captures (`codes/libs/ui/libbhoo/examples/android_launcher_capture.rs`).
