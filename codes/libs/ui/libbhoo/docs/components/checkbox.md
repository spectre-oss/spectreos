# Checkbox

Status: Partial implementation — binary/tri-state state handling, keyboard activation, mark/fill motion, and Android-inspired independent error colors; platform accessibility bridging and full visual parity remain open. See the [implementation record](../implementation-progress.md).

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | Material/framework-inspired component family |
| Priority | P1 |
| Motion profile | [Selection](../motion.md#motion-profiles) |

## Scope and variants

Checked, unchecked, indeterminate. `checkbox_with_error` adds an independent error visual state to each tri-state value; `checkbox_bool_with_error` provides the same state for a binary value without introducing a mixed state. As in the pinned Android source, disabled styling takes precedence over error styling. The error flag is caller-owned and does not alter the selection cycle. Unchecked outlines leave their center transparent, so the surface and interaction layer remain visible beneath them. The semantic snapshot exposes `invalid` independently of the value, including when disabled. It does not carry an error description, and platform accessibility bridges remain open; callers must provide an accessible explanation separately. The checkbox demo includes a separately labeled Android error variant with a switch for showing or clearing the error and a visible explanation, without claiming it matches the Angular reference example.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Represent unchecked, checked and mixed states; Space changes the value according to the documented cycle.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

The binary checkbox uses the Angular reference’s 90 ms fill transition and delayed checkmark path reveal, with the SVG path scaled into the 14 px content area inside its 18 px border. Interrupted transitions retarget from their current presentation and reduced motion snaps. A deterministic Rust test checks the reveal delay and 14 px mark geometry. The browser focus-layer check passes with the shared Angular palette; the checked-icon crop has zero pixels above threshold 16 out of 400 after using a butt-capped, miter-joined path. The native renderer uses the same path geometry and a one-physical-pixel antialiasing fringe. Wayland/GLES smoke tests verify checked and mixed marks, their filled boxes, and intermediate checkmark edge colors at scales 1 and 2; full native visual and motion parity still needs validation. Mixed-state transitions and precise visual parity remain open; Material Web uses a distinct scale-based animation.

Use the shared [selection profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Commit logical selection immediately and snap its visual indicator to the target; keep direct manipulation immediate.

### Angular comparison focus layer

The Angular profile keeps the 40 px focus state layer after pointer or keyboard
focus, until focus leaves. Checked focus uses primary at 12% opacity; unchecked
and mixed focus use on-surface. Hover uses 8% and only the checkbox's 40 px
control area; disabled controls do not show a state layer. The browser regression
`node tools/test-bhoo-checkbox-focus.mjs` samples the focused checked layer in
both implementations. This check does not establish complete checkbox animation
or accessibility parity. See [Angular checkbox state layer](https://github.com/angular/components/blob/main/src/material/checkbox/_checkbox-common.scss).

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Verify the mixed-state transition and announce the logical value once, independently of animation.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [Material Web component guide](../material-design/upstream/docs/components/checkbox.md)
- [Material design guidance linked by upstream](https://m3.material.io/components/checkbox)
- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/checkbox/)

- [Local upstream visual example](../material-design/upstream/docs/components/images/checkbox/hero.webp)

Visuals illustrate the upstream design or the explicitly labeled related pattern; they are not screenshots of a completed Bhoo component.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Source-level references for this family. Some files supply shared behavior or a variant; this is not a claim of equivalent feature coverage across toolkits.

| Toolkit | Pinned implementation | Local source snapshot |
| --- | --- | --- |
| Material Web | [checkbox.ts](https://github.com/material-components/material-web/blob/d872455c4be56022da2095349b30ef46e0b55bc3/checkbox/internal/checkbox.ts) | [Downloaded file](../../../../context/reference-sources/material-web/checkbox/internal/checkbox.ts) |
| Material Components for Android | [MaterialCheckBox.java](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/checkbox/MaterialCheckBox.java) | [Downloaded file](../../../../context/reference-sources/material-android/lib/java/com/google/android/material/checkbox/MaterialCheckBox.java) |
| Compose Material 3 | [Checkbox.kt](https://github.com/androidx/androidx/blob/1448d64974931a04c9fba96ac630cbd6b96cda36/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Checkbox.kt) | [Downloaded file](../../../../context/reference-sources/compose-material3/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Checkbox.kt) |
| Angular Material | [checkbox.ts](https://github.com/angular/components/blob/249b3ee5c8b334dcb6b8d9e035380318e4438c2c/src/material/checkbox/checkbox.ts) | [Downloaded file](../../../../context/reference-sources/angular-material/src/material/checkbox/checkbox.ts) |
| Flutter material_ui | [checkbox.dart](https://github.com/flutter/packages/blob/565c6573861992fbd225ca138bbc2d3b8aeb5d00/packages/material_ui/lib/src/checkbox.dart) | [Downloaded file](../../../../context/reference-sources/flutter-material/packages/material_ui/lib/src/checkbox.dart) |

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Material Web: checkbox.md](../material-design/upstream/docs/components/checkbox.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

### Browser morph comparison

Run `node tools/test-bhoo-checkbox-morph.mjs` with the comparison server and a Chromium DevTools endpoint running. It captures simultaneous 22 × 22 icon crops during the fixture’s mixed-to-checked toggle and reports sampling time and pixel differences; the snapshots are evidence, not a parity assertion. The fixture initializes its mixed state after Angular creates the input, and the test checks that Angular assigns `mdc-checkbox--anim-indeterminate-checked` to that input. In the pinned Angular reference, the class is assigned to the input but the keyframe selectors expect an ancestor, so the browser applies fallback CSS transitions instead. Bhoo’s Angular comparison profile approximates those transitions; early sampled frames still differ, and capture latency prevents a claim of exact parity. The reported timestamps precede screenshot capture and do not account for capture latency.
