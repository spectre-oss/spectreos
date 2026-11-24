# Link

Status: Experimental Rust widget; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | MUI-inspired text link |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

Text link with underline, hover, keyboard activation and a caller-owned navigation response. Native browser anchors, visited states and configurable underline remain open.

## State and interaction contract

Activation returns a response; the host application chooses how to navigate. Disabled links do not activate. The core exposes a link semantic role.

## Motion and animation

Underline feedback is immediate. **Reduced motion:** unchanged.

## Acceptance criteria

- Pointer and keyboard activation return a response.
- Disabled links cannot activate.

## References

- [MUI Link source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Link)
- [MUI Link documentation](https://mui.com/material-ui/react-link/)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
