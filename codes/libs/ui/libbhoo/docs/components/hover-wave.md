# Hover wave

Status: Experimental Rust feedback primitive inspired by Beer CSS; exact raster parity remains open.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | Beer CSS-inspired hover feedback |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

A centered hard-edged radial state layer that expands on pointer hover or keyboard-visible focus. Beer CSS custom background sizing and exact gradient rasterization remain open.

## State and interaction contract

The wave is attached to a caller-owned target. Pointer press, pointer leave and disabled state remove it immediately; activation remains the target's responsibility.

## Motion and animation

The default Beer CSS helper transitions background size over 200 ms linearly and uses a 10% state layer. Bhoo approximates that gradient as an expanding clipped circle. **Reduced motion:** snap to the static hover state.

## Acceptance criteria

- Hover starts a 200 ms linear expansion and leave or press clears it.
- Disabled controls do not draw feedback.

## References

- [Beer CSS wave helper](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/helpers/wave.css)
- [Beer CSS timing tokens](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/settings/global.css)
- [Beer CSS MIT license](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/LICENSE)
