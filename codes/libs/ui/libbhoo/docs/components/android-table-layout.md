# Android XML TableLayout

Status: Initial shared-column implementation.

| Field | Decision |
| --- | --- |
| Category | Android XML views |
| Kind | Android framework layout |
| Priority | P1 |

## Scope and variants

TableLayout arranges TableRow children vertically and shares the widest natural cell width in each column. The initial implementation supports sequential cells, explicit column indices and spans, stretchable, shrinkable, and collapsible columns, horizontal and vertical cell gravity, text baseline alignment with a row opt-out, row backgrounds, alpha, foreground, visible/invisible/gone rows, margins, gone cells, match-parent cell height and basic wrap-content height, actions, right-to-left column order, and per-row direction overrides. Standalone TableRow supports horizontal layout, margins, text baseline alignment, actions, and RTL; its virtual-column behavior still needs parity work. Table and row child clipping and clip-to-padding settings are applied during rendering and hit testing. Non-row children span the available table width.

## State and interaction contract

Cells retain their own interaction state and can dispatch their usual actions. Baseline accuracy for other view types, some clipping edge cases and Android's detailed measurement rules are not implemented; unsupported XML attributes fail compilation. Android device comparison remains open.

## Motion and animation

TableLayout positions cells directly; cell controls provide their own interaction motion.

**Reduced motion:** Placement remains direct.

## Acceptance criteria

- Share each column's largest non-spanning cell width between rows; position explicit gaps and spanning cells.
- Preserve cell actions and reverse column order in RTL.
- Implement and verify the remaining column variants against Android.

## References

- [Android TableLayout source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TableLayout.java)
- [Android TableRow source (Apache-2.0)](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-15.0.0_r1/core/java/android/widget/TableRow.java)

## Wrap-content shared-column measurement

TableLayout measures supported wrap-content children with the active renderer text metrics and current state before finding each shared column's widest cell. Cell gravity and row baseline calculation use the same measured width. A focused example changes a Button label and checks that both rows' second columns move together in LTR and RTL. Android device typography, theme and complete TableLayout measurement remain unverified. The Apache-2.0 Android 15 sources above are stored at `context/reference-sources/android-framework/TableLayout.java` and `context/reference-sources/android-framework/TableRow.java`.

## Nested wrap-content width

A wrap-content TableLayout calculates its own width from its shared columns using the active renderer text metrics and current child state. Supported non-row children also contribute to this width. A focused nested example checks the resized cell, next column and following outer sibling in LTR and RTL; Rust coverage includes non-default text metrics. Complete Android table constraints, font/theme measurement and device parity remain unverified.
