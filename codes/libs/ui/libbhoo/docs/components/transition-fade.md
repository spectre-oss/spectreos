# Fade transition

Status: Experimental Rust transition primitive; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | MUI-inspired transition |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

Fade a caller-supplied bounded subtree. Custom durations, easing, mount policies and transition callbacks remain open.

## State and interaction contract

Closing content immediately stops accepting input; visible children retain their caller-owned state. Fully hidden content is not built.

## Motion and animation

The default uses the MUI ease-in-out curve with 225 ms enter and 195 ms exit. Reversals start from the current opacity. **Reduced motion:** snap to the target.

## Acceptance criteria

- Fade reaches its endpoint after the correct default duration.
- Closing children cannot activate.

## References

- [MUI Fade source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Fade)
- [MUI transition tokens](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/styles/createTransitions.js)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
