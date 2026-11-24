# Grid layout

Status: Experimental Rust layout primitive; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | MUI-inspired layout primitive |
| Priority | P1 |
| Motion profile | None |

## Scope and variants

Equal-width columns with caller-specified item count, row height and gap. Responsive breakpoints, spans, offsets and inherited nested layout remain open.

## State and interaction contract

The grid owns no selection state; children manage their own input and semantics. Unused cells do not build children.

## Motion and animation

Layout is immediate. **Reduced motion:** unchanged.

## Acceptance criteria

- Grid invokes its builder once per item and positions items in equal columns.
- Zero items or columns produce no children.

## References

- [MUI Grid source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Grid)
- [MUI Grid documentation](https://mui.com/material-ui/react-grid/)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
