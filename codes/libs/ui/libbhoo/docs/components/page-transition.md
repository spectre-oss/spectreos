# Page transition

Status: Experimental Rust page primitive inspired by Beer CSS; full parity remains open.

| Field | Decision |
| --- | --- |
| Category | Navigation and toolbars |
| Kind | Beer CSS-inspired page transition |
| Priority | P1 |
| Motion profile | Navigation |

## Scope and variants

Caller-owned active page with neutral, left, right, top or bottom entry. Exactly one active page is selected by the application. Nested routing, navigation history and focus handoff remain open.

## State and interaction contract

Inactive page content is not built and has no input targets. Page selection is application-owned.

## Motion and animation

Beer CSS uses a 400ms CSS ease entry from zero opacity and, for directional pages, a 4rem translation. Bhoo uses a 64 logical-pixel offset with the CSS ease curve. **Reduced motion:** snap to the active page.

## Acceptance criteria

- Switching pages hides the previous page and reveals the chosen page.
- Entry fades and follows the requested direction, then settles.
- Reduced motion snaps and hidden pages have no active controls.

## References

- [Beer CSS Page guide](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/docs/PAGE.md)
- [Beer CSS Page styles](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/elements/page.css)
- [Beer CSS keyframes](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/settings/global.css)
- [Beer CSS MIT license](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/LICENSE)
