# Avatar group

Status: Partial Rust renderer with reference-based max and total counting, mixed avatar content and layout options.

| Field | Decision |
| --- | --- |
| Category | Data and application patterns |
| Kind | MUI-inspired display component |
| Priority | P1 |
| Motion profile | None |

## Scope and variants

`Frame::avatar_group_with_options` draws caller-provided images, initials or person fallbacks with a surplus count. Its maximum includes the surplus slot; when the total exactly matches the maximum, all supplied avatars can appear. An optional total accounts for contributors absent from the supplied items. Max values below two are clamped to two; a total of zero follows the MUI reference's default-to-child-count behavior. A 2 dp border uses the background color, with 8 dp default overlap or caller-selected overlap. Circle, rounded and square shapes, a uniform size and logical RTL placement are supported. Images are caller-owned RGBA resources; loading remains a host responsibility. The legacy `avatar_group` API retains its earlier visible-contributor-count behavior.

## State and interaction contract

This presentation has no focus targets. Applications must provide accessible names for contributors separately.

Only avatars intersecting the current clip generate drawing commands; surplus counts still use the complete input. The user-facing example can change the maximum, total, overlap, shape and direction.

## Motion and animation

There is no default animation. **Reduced motion:** unchanged.

## Acceptance criteria

- Visible avatars overlap consistently and hidden contributors produce a surplus count.
- Decorative avatars do not add focus targets.
- Verify reference max/total cases, image content, clipping and RTL placement.

## Validation

Rust avatar tests cover the pinned MUI max/total cases, mixed content, shape, overlap, RTL and bounded drawing for 100,000 inputs. The browser example is checked with a phone-sized canvas. Exact MUI tokens, arbitrary child sizes, custom surplus renderers and device/framework pixel parity remain open.

## Guidelines and visual references

- [MUI AvatarGroup examples](https://mui.com/material-ui/react-avatar/#grouped)

No upstream screenshot has been independently verified for this example; the reference page is a design guide, not a Bhoo parity claim.

## Downloaded reference implementation

The [pinned MUI AvatarGroup source](../../../../context/material-ui/packages/mui-material/src/AvatarGroup/AvatarGroup.js) and its [tests](../../../../context/material-ui/packages/mui-material/src/AvatarGroup/AvatarGroup.test.js) are in the local checkout at revision `0c82795c732a0ae809fd51f9537ee70747ef20a1`, under the [MIT license](../../../../context/material-ui/LICENSE). They informed counting, border and spacing choices. Bhoo's person fallback is the separately attributed Apache-2.0 Google Material Symbol; see [licenses and attribution](../licenses-and-attribution.md).

## References

- [MUI AvatarGroup source, pinned](https://github.com/mui/material-ui/tree/0c82795c732a0ae809fd51f9537ee70747ef20a1/packages/mui-material/src/AvatarGroup)
- [MUI Avatar documentation](https://mui.com/material-ui/react-avatar/)
- [MUI MIT license](https://github.com/mui/material-ui/blob/0c82795c732a0ae809fd51f9537ee70747ef20a1/LICENSE)
