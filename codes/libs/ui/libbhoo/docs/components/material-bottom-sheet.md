# Material bottom sheet

Status: Initial modal drag-to-dismiss implementation.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | Material modal bottom sheet |
| Priority | P1 |

## Scope and variants

The modal bottom sheet has a scrim, rounded surface, drag handle and actions. A downward drag follows the pointer directly; releasing past 40% of the sheet height or making a sufficiently fast downward flick dismisses it; a shorter slow drag settles back. Scrim and Escape dismissal and explicit actions remain available. The drag is opt-in for Material3 overlays.

## State and interaction contract

Dragging begins on the top handle area. Cancelled input settles back; a new drag can interrupt the settle. The initial implementation uses an approximate velocity tracker and dismissal threshold; standard-sheet detents, nested scrolling and exact Android motion parity remain open.

## Motion and animation

The sheet follows pointer movement and settles using a bounded critically damped curve. Reduced motion snaps the settle to its target. The exit continues from the dragged position.

**Reduced motion:** Present or dismiss without spatial animation and snap a released drag back.

## Acceptance criteria

- Short drags settle back; long drags dismiss; cancelled drags do not dismiss.
- Explicit actions and scrim dismissal work, including after an interrupted settle.
- Compare dimensions, drag velocity and motion against Android on a device.

## References

- [Material Components Android bottom sheet guide (Apache-2.0)](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/BottomSheet.md)
- [Android bottom sheet behavior (Apache-2.0)](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/lib/java/com/google/android/material/bottomsheet/BottomSheetBehavior.java)
