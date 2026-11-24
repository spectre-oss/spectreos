# Breadcrumb

Status: Experimental Rust browser composition available; the full component specification remains incomplete.

| Field | Decision |
| --- | --- |
| Category | SpectreOS desktop additions |
| Kind | SpectreOS desktop proposal |
| Priority | P1 |
| Motion profile | [Feedback](../motion.md#motion-profiles) |

## Scope and variants

Hierarchical paths and navigation.

This is one family specification; variants share this document. The names describe proposed design scope, not new exported Rust APIs or completed upstream parity.

## State and interaction contract

Represent a hierarchical path with named ancestor actions and a current location; collapse overflow predictably.

The proposed public contract should expose the value/state and action results described above. The caller owns application data. Preserve stable IDs and prevent stale callbacks after removal. Final Rust signatures remain open until the supporting input, layout and semantic infrastructure is available.

## Motion and animation

Use normal action feedback; overflow changes do not animate the entire path on each resize.

Use the shared [feedback profile](../motion.md#motion-profiles). Motion here is a Bhoo proposal, not a claim to reproduce official Material timing or physics. Functional state changes must not wait for visual completion. Reversals start from the current presented state; remove animation state when its owner disappears.

**Reduced motion:** Snap to the final color/visibility state; replace spatial press/ripple effects with a static state layer.

## Accessibility and RTL

Apply the semantic roles, names, focus and input requirements in the [shared component contract](README.md#shared-component-contract) where relevant. Decorative primitives must not create extra focus targets. Labels and editable text require correct shaping and bidirectional handling; interpret leading/trailing placement logically. Preserve the specific focus rules in this document when content becomes hidden or disabled.

## Performance and lifecycle

Apply the [motion scheduler and cache rules](../motion.md#scheduling-and-lifecycle). Avoid independent per-component timers. Static or settled instances must stop requesting frames; invisible indeterminate effects must pause. Restrict layout/geometry work to affected visible content and use bounded caches appropriate to this family's data.

## Acceptance criteria

- Resize a long RTL path and preserve the current location and keyboard access to hidden ancestors.
- Verify normal and reduced-motion paths, interruption/reversal, disable/removal and final settled state where applicable.
- Verify pointer and applicable keyboard/semantic behavior independently of animation completion.
- Apply only relevant shared checks; document exceptions rather than inventing interactions for decorative primitives.

## Guidelines and visual references

- [WAI interaction and accessibility pattern](https://www.w3.org/WAI/ARIA/apg/patterns/breadcrumb/)

No dedicated image was verified for this entry in the reference snapshot. The linked guides provide behavior and implementation references; a Bhoo visual specification remains to be produced.

For desktop proposals, web/framework interaction guidance is a reference to adapt to native semantics, not a claim that this is an official Material component. See [shared motion guidance](../motion.md) for animation requirements.

## Downloaded reference implementations

Related building blocks only; this is a SpectreOS-specific proposal, not an upstream component implementation.

No matching implementation file was selected in the small source snapshots. Use the component-specific guidelines above; an exact reference remains an open research item. This is not evidence that no implementation exists elsewhere.

The snapshots omit dependencies, tests, assets and build configuration, so they are for reading rather than standalone builds. Local links resolve after downloading the ignored `context/reference-sources` folder; pinned upstream links remain usable without it. See [source inventory and attribution](../reference-implementations.md) and the [license review](../licenses-and-attribution.md) before adapting code.

## References

- [Bhoo catalog and reference implementations](../components.md)
- [Shared motion design](../motion.md)
- [Catalog](../components.md)

## Browser demonstration

[Open the live Breadcrumb preview](http://127.0.0.1:8766/#/breadcrumb).

Demonstrates: Three navigable path levels with selected content.

Remaining scope: Overflow/collapse, current-page semantics and arbitrary hierarchical model.

See [demo coverage and motion baseline](../demo-coverage.md). The state/motion contracts above describe the target design; they are not all established by this preview.
