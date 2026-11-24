# Expressive shapes

Status: Experimental Rust shapes with wavy asset motion; complete Beer CSS fidelity remains open.

| Field | Decision |
| --- | --- |
| Category | Visual and interaction foundations |
| Kind | Beer CSS expressive shape library |
| Priority | P1 |
| Motion profile | None |

## Scope and variants

The Rust library contains 37 SVG-derived silhouettes from Beer CSS, including animated wavy and wavy-circle assets. The separately animated loading-indicator SVG, shape morphs and image masks are not implemented here. Curves are sampled into compact polygon meshes; this is an approximation, not exact SVG rendering.

## State and interaction contract

Shapes are decorative drawing primitives, do not take focus, and leave accessible labels to the parent. Callers supply bounds and color.

## Motion and animation

The wavy asset translates over one second; wavy-circle rotates counterclockwise over eight seconds. The wave repeats sampled geometry, so exact SVG raster and clipping parity are not yet verified. **Reduced motion:** show both at their initial geometry without continuous frames. Shape morphing remains open.

## Acceptance criteria

- Every static asset has finite geometry, valid tessellation and bounded rendering.
- Concave silhouettes and holes render in browser and GLES backends.
- Shapes do not create input targets.

## References

- [Beer CSS Shape guide and SVG credits](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/docs/SHAPE.md)
- [Beer CSS SVG assets](https://github.com/beercss/beercss/tree/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/shapes)
- [Beer CSS MIT license](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/LICENSE)

Beer CSS credits Daria Dororoshchuk for the SVG shapes and KTibow for the loading indicator. Geometry is derived from those assets under the MIT license; attribution and the complete license are included in THIRD_PARTY_NOTICES.md.
