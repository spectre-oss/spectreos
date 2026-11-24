# Image media

Status: Experimental image renderer inspired by Beer CSS media; video and full parity remain open.

| Field | Decision |
| --- | --- |
| Category | Data and application patterns |
| Kind | Beer CSS-inspired media element |
| Priority | P1 |
| Motion profile | Feedback |

## Scope and variants

Caller-owned RGBA8 images, centered cover and contain fit, rectangular and rounded clipping. The native renderer uses a bounded 1024×1024 RGBA atlas; individual dimensions are limited to 1022px, and an entire frame must fit in the atlas. The browser uses a 16 MiB bounded decoded-image cache. Decoding, file/network loading, atlas paging, arbitrary-size images and video remain open.

## State and interaction contract

Images are immutable shared resources; applications own their lifetime and loading state. The drawing primitive is decorative and does not create a focus target. Applications must provide accessible descriptions until a media semantics API exists.

## Motion and animation

Image drawing is static. **Reduced motion:** unchanged. Beer CSS transform and corner-radius transitions remain open.

## Acceptance criteria

- Cover crops and contain preserves the aspect ratio.
- Browser and GLES render caller-owned RGBA pixels and rounded clipping.
- Resource sizes and caches are bounded; invalid pixel buffers are rejected.

## References

- [Beer CSS Media guide](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/docs/MEDIA.md)
- [Beer CSS Media styles](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/src/cdn/elements/media.css)
- [Beer CSS MIT license](https://github.com/beercss/beercss/blob/47592ff1a644df437ebdf08c8dbdc967ed2e68db/LICENSE)
