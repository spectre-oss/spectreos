# Shape rotation

Status: Experimental Rust animation primitive inspired by Beer CSS; full parity remains open.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | Beer CSS-inspired shape animation |
| Priority | P1 |
| Motion profile | Continuous |

## Scope and variants

Continuously rotate a caller-selected shape. The Beer CSS slow, normal and fast helpers use 24, 12 and 6 seconds per revolution. Counter-rotating child content and arbitrary origins remain open.

## State and interaction contract

The caller owns the rotation clock. Offscreen or paused content does not consume frames and resumes without replaying hidden time. Shapes have no focus targets.

## Motion and animation

Rotation uses a linear full revolution at the chosen duration. **Reduced motion:** present a stationary shape and stop scheduling animation frames. The comparison fixture disables Beer CSS animation under reduced motion for this shared setting.

## Acceptance criteria

- A normal-speed rotation reaches 90 degrees in three seconds.
- Hidden and reduced-motion shapes do not request continuous frames.

## References

- [Beer CSS Shape rotation styles](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/elements/shape.css)
- [Beer CSS rotation keyframes](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/settings/global.css)
- [Beer CSS MIT license](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/LICENSE)
