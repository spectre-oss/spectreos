# Stack layout

Status: Experimental Rust layout primitive; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | MUI-inspired layout primitive |
| Priority | P1 |
| Motion profile | None |

## Scope and variants

Vertical stack with caller-specified spacing. Horizontal and responsive direction and dividers remain open.

## State and interaction contract

Layout owns no input or selection state; children manage their own behavior.

## Motion and animation

Layout is immediate. **Reduced motion:** unchanged.

## Acceptance criteria

- Children fit the configured dimensions and spacing.

## References

- [MUI Stack source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Stack)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
