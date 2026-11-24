# Rating

Status: Experimental Rust widget and browser demo; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Selection and input |
| Kind | MUI-inspired interactive component |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

Whole-star selection with pointer hover preview and a caller-owned value. This first implementation supports five stars and disabled controls; precision, read-only and custom icon variants remain open.

## State and interaction contract

Select a star by pointer or arrow keys; selecting the current star clears the value. Disabled controls cannot change. Radio semantics report the current choice.

## Motion and animation

Hover previews are immediate. **Reduced motion:** no animated preview is required.

## Acceptance criteria

- Pointer selection, preview, clearing and disabled state work.
- Arrow, Home and End keys change the value.

## References

- [MUI Rating source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Rating)
- [MUI Rating documentation](https://mui.com/material-ui/react-rating/)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
