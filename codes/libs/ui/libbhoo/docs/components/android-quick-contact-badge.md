# Android XML QuickContactBadge

Status: Initial XML image badge and host activation. Android contacts provider integration and device parity remain open.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework widget |
| Priority | P1 |

## Scope and variants

QuickContactBadge uses ImageView sizing, scaling, image tint and alpha, including Wave 24A/24B sampled `<animated-vector>` `src`/`bg` keyframe playback (`RULE_ANIMATED_VECTOR = 309`, 8 sampled keyframes for numeric group/path transform, trim, and alpha properties without per-frame SVG `pathData` morphing; see [ImageView](android-imageview.md)). The host supplies an image and an optional URI, email or phone target, with lazy or immediate email/phone lookup. Assigning or clearing a target controls whether activation is available. The host can use Android 15’s default contact avatar. The enabled badge uses Android 15’s stateful light or dark nine-patch overlay, with pressed and focused states and RTL mirroring; the theme mapping still requires device comparison.

## State and interaction contract

Activation requests a host lookup for an email or phone, or asks the host to show a known URI. A lookup result can request showing a found contact or creating one when absent. Tokens reject stale results after reassignment. The embedding app performs lookups and platform UI actions; changing the target does not automatically change the image.

## Motion and animation

No specific badge animation is implemented beyond shared ImageView drawable playback (`RULE_ANIMATION_LIST = 285`, `RULE_DRAWABLE_DYNAMIC = 286`, `RULE_ANIMATED_SELECTOR = 293`, and Wave 24A/24B `RULE_ANIMATED_VECTOR = 309` with clip-gated `f.wake_at()` wakeups). The Android overlay changes immediately on press and focus.

**Reduced motion:** The badge has no movement; `<animated-vector>` drawables settle immediately on `len - 1` (`repeat == 0`) or frame `0` (`repeat == 1`) with zero wakeups.

## Acceptance criteria

- Import ImageView-compatible XML attributes and show a host-provided image.
- Activate only when a target is assigned and report that target.
- Implement an Android contacts provider bridge and device parity.

## References

- [Android 15 QuickContactBadge source](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/QuickContactBadge.java) (Apache-2.0).
