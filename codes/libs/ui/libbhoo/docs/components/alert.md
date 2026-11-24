# Alert

Status: Experimental Rust widget; not full MUI parity.

| Field | Decision |
| --- | --- |
| Category | Communication and feedback |
| Kind | MUI-inspired inline feedback |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

Inline success, info, warning and error messages with a dismiss action. Filled and outlined variants, custom actions and exact MUI tokens remain open.

## State and interaction contract

The caller controls visibility. Activating the close control returns a response so the caller can remove the message. Accessible announcement roles remain future work.

## Motion and animation

Dismissal is immediate; the close control uses shared pointer feedback. **Reduced motion:** use static feedback.

## Acceptance criteria

- Each severity is distinguishable and can be dismissed independently.
- Disabled state prevents close activation.

## References

- [MUI Alert source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/Alert)
- [MUI Alert documentation](https://mui.com/material-ui/react-alert/)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
