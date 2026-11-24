# Collapse transition

Status: Experimental Rust transition primitive; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | MUI-inspired transition |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

Collapse a bounded subtree vertically while moving following content. Horizontal orientation, custom durations, easing, mount policies and transition callbacks remain open.

## State and interaction contract

Closing content immediately stops accepting input; visible children retain their caller-owned state. Fully hidden content is not built.

## Motion and animation

The default uses the MUI ease-in-out curve and its auto-height duration formula. Reversals start from the current size. **Reduced motion:** snap to the target.

## Acceptance criteria

- Collapse reaches its endpoint after the correct default duration.
- Closing children cannot activate.

## References

- [MUI Fade source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Collapse)
- [MUI transition tokens](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/styles/createTransitions.js)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
