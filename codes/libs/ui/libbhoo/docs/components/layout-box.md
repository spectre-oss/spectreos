# Box layout

Status: Experimental Rust layout primitive; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Surfaces and containment |
| Kind | MUI-inspired layout primitive |
| Priority | P1 |
| Motion profile | None |

## Scope and variants

A styled rectangular region with background, radius, padding and clipped child layout. Responsive properties and the MUI sx system remain open.

## State and interaction contract

Box owns no input or selection state; children manage their own behavior.

## Motion and animation

Layout is immediate. **Reduced motion:** unchanged.

## Acceptance criteria

- Child content receives configured padding and clipping.

## References

- [MUI Box source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Box)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
