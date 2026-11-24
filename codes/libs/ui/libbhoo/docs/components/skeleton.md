# Skeleton

Status: Experimental Rust rendering primitive; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Communication and feedback |
| Kind | MUI-inspired loading placeholder |
| Priority | P1 |
| Motion profile | Loading |

## Scope and variants

Circular, text and rounded placeholders communicate content that is still loading. Geometry is supplied by the caller.

## State and interaction contract

The skeleton is noninteractive and does not expose unnecessary focus targets. Applications own loading status and accessible announcements.

## Motion and animation

The pulse uses a two-second ease-in-out opacity cycle with a half-second initial delay, following the pinned MUI source. **Reduced motion:** show a stationary placeholder. Wave animation remains open.

## Acceptance criteria

- Pulse animates while visible and stops under reduced motion.
- Shapes retain their geometry during the pulse.

## References

- [MUI Skeleton source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Skeleton)
- [MUI Skeleton documentation](https://mui.com/material-ui/react-skeleton/)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
