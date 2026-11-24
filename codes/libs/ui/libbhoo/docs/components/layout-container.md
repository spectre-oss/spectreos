# Container layout

Status: Experimental Rust layout primitive; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | MUI-inspired layout primitive |
| Priority | P1 |
| Motion profile | None |

## Scope and variants

Centered content region with caller-specified maximum width. Responsive breakpoints and gutters remain open.

## State and interaction contract

Layout owns no input or selection state; children manage their own behavior.

## Motion and animation

Layout is immediate. **Reduced motion:** unchanged.

## Acceptance criteria

- Children fit the configured dimensions and spacing.

## References

- [MUI Container source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Container)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
