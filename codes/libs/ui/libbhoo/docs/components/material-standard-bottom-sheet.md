# Material standard bottom sheet

Status: Initial non-modal collapsed and expanded detents.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material standard bottom sheet |
| Priority | P1 |

## Scope and variants

A standard bottom sheet remains alongside the page without a scrim. It starts at a configurable peek height and can be dragged by its handle between collapsed, optional half-expanded, and expanded positions. A directional release follows Material Android's handle-release rules; a stationary or mostly horizontal release uses its position. The host can open or close it independently.

## State and interaction contract

The sheet exposes its target expanded state. Cancelled drags return to the previous detent; dragging can interrupt settling. Changes to the target or peek height retarget an open sheet from its current position, including during settling; changes during an active drag take effect on release. The sheet supports touch scrolling in a caller-defined content region. Upward motion first expands the sheet and then scrolls content; downward motion first returns content to its top and then collapses the sheet. On touch release the sheet chooses a detent using the last scroll direction, following Material Android's nested-scroll policy. A subsequent non-touch content fling does not move the sheet. Content uses Bhoo spline fling physics. Half-expanded position uses a configurable viewport-height ratio. Detailed Android velocity tracking and device parity remain open.

## Motion and animation

The handle follows pointer movement; release settles using a bounded critically damped curve. Reduced motion snaps settling.

**Reduced motion:** Snap to the chosen detent.

## Acceptance criteria

- Drag between collapsed, half-expanded and expanded states; cancel without changing the target.
- Keep the page available without a modal scrim.
- Verify content-to-sheet gesture handoff and compare motion and detents with Android on a device.

## References

- [Material Android bottom sheet guide (Apache-2.0)](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomSheet.md)
- [Material Android bottom sheet behavior (Apache-2.0)](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomsheet/BottomSheetBehavior.java)
