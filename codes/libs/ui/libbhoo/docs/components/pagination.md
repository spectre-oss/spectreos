# Pagination

Status: Experimental Rust widget; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Data and application patterns |
| Kind | MUI-inspired numbered pagination |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

Numbered pagination with one-based caller-owned page state and previous/next buttons. Large page counts are condensed using ellipses and fixed sibling and boundary counts. Configurable counts and first/last controls remain open.

## State and interaction contract

Numbered buttons select a page. Previous and next are disabled at their boundaries. Keyboard focus and activation use the common Bhoo button model.

## Motion and animation

Selection and disabled states update immediately; pointer feedback uses the shared state layer. **Reduced motion:** use static feedback.

## Acceptance criteria

- Numbered selection and previous/next update the page.
- Previous and next disable at their respective boundaries.

## References

- [MUI Pagination source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Pagination)
- [MUI Pagination documentation](https://mui.com/material-ui/react-pagination/)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
