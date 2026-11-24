# Four Android XML integration examples

These are unofficial, offline Bhoo integration examples, not ports of the complete Android applications. They exercise the Rust layout, input, scrolling, navigation and storage boundary in an application-sized flow. No real messages are sent and no accounts are connected.

| Example | Executed original XML | Local flows | Source license |
| --- | --- | --- | --- |
| Fossify Messages | `item_conversation.xml` | Conversation list, search, contact picker, conversation, local send, draft persistence | GPL-3.0 |
| Delta Chat | `conversation_list_item_view.xml` | Chat list, search, contact picker, conversation, local send, draft persistence | GPL-3.0 |
| Thunderbird/K-9 | `message_list_item.xml` | Inbox, starred/archive folders, message detail, star/unstar, archive/restore, reply, compose, local send | Apache-2.0 |
| Sunflower | `list_item_plant.xml` | Plant catalog, search, detail, add/remove garden, saved garden | Apache-2.0 |

## Provenance and reproducibility

Original files are preserved under `codes/apps/bhoo-web/android-app-tests/<app>/layout`, together with upstream value resources and licenses. `manifest.json` records the pinned revision and downloaded reference inventory; only selected files from that inventory are bundled here. `ADAPTERS.json` records explicit custom-view substitutions, layout overrides, external dimension aliases and unresolved theme references. Original screen XML is retained for comparison, but the executed XML in this version is the row/card template listed above. Activities and custom Android controllers are reimplemented as local Rust controllers.

- [Fossify Messages, d9510f44](https://github.com/FossifyOrg/Messages/tree/d9510f44b03c82869856d89a0fcd30b880df9688)
- [Delta Chat, d5bc5cf3](https://github.com/deltachat/deltachat-android/tree/d5bc5cf38aae16ee057e68b9c42d0b9aae351604)
- [Thunderbird, 3542b3e7](https://github.com/thunderbird/thunderbird-android/tree/3542b3e75bef57405ef0ce0a4d6c4655d7bb8eb5)
- [Sunflower Views, 99eb5941](https://github.com/android/sunflower/tree/99eb59417bf8a95999d2334eb5dc320627b9304e)

`tools/import-android-app-tests.py` compiles the original XML to static Rust `NativeNode` trees. It is a fixture importer, not a general Android runtime. Build with `python3 tools/build-bhoo-web.py --rust-only`, then `python3 tools/build-bhoo-app-tests.py`. No reference download is required to rebuild the fixtures. Each webxdc includes the corresponding source archive and license notices.

## Runtime

Rust owns application data, routes, search, form values, list culling, hit testing, state layers and scroll physics. One canvas paints the Rust command packet. The host provides browser text editing/IME and a localStorage snapshot. Touch scrolling uses the shared Bhoo Android-inspired spline fling and spring edge return. Only visible rows/cards are drawn. The canvas uses the device pixel ratio where possible, bounded to 8,388,608 pixels (32 MiB for one RGBA surface) and 8,192 pixels per dimension. Larger surfaces use a lower ratio to stay within those limits. Animation frames stop when the scene is idle or hidden.

Snapshots are versioned, app-scoped UTF-8 records with hex-encoded fields. Restore validates the header, field encoding and row limits before replacing state. Corrupt data does not partially overwrite the active store. Source data and snapshots are local, not webxdc status updates. Send means append locally. The four archives do not share storage. To bound this test fixture’s storage, it retains at most 32 mail records and the newest 2 KiB of each conversation; subjects are limited to 256 UTF-8 bytes and recipients to 128. These are demo limits, not production retention policies.

## Fidelity boundaries

These builds are integration tests, not pixel-identical replicas. They do not execute Kotlin, Android services, arbitrary custom views, Compose, Android resource qualifiers or the complete ConstraintLayout solver. Their headers, conversation bubbles, compose/detail views and navigation are Bhoo compositions. Theme selectors and icon drawables use Bhoo's Material icons and colors. Sunflower uses locally drawn placeholder plant illustrations and fictional growing advice rather than remote photographs or production botanical data.

The bridge supports the bounded sibling constraints used in these selected templates; it is not suitable for loading arbitrary untrusted layouts. XML data binding is replaced by explicit Rust ID bindings. Per-fixture overrides are reviewable in the adapter reports. Android screenshots should be compared separately before making any visual parity claim.

## Licenses

The GPL-3.0 Bhoo application source and generated adaptations are distributed with their corresponding source. Original Apache-2.0 headers and license text are preserved for Thunderbird and Sunflower. Original GPL license text is preserved for Fossify and Delta Chat. No upstream app logos are used. The shared Bhoo ghost artwork retains its separately documented Koboyo terms in the existing third-party notices; it is not relicensed as GPL.

## Validation

The browser regression exercises all four fixtures at a phone viewport, navigation, searches, local sends, multi-field mail composition, archive/star state, garden mutations, persistence after reload, touch scrolling without accidental activation, information dialogs, desktop layout and network isolation. Rust tests cover snapshot round trips, Unicode, app scoping and malformed records. Screenshot artifacts are saved under ignored `context/bhoo-app-tests`.

## Toolbar and page motion

The Thunderbird fixture opens an animated modal folder drawer from its toolbar menu. Folder choices use the existing local inbox, starred and archive state; the scrim and Escape dismiss it. The Sunflower source home layout has a centered title and a ViewPager/TabLayout, not a navigation drawer. The fixture removes the extra menu button and presents the two destinations as tabs. Page transitions use directional incoming motion, with a 200 ms duration drawn from the pinned Sunflower `values/anim.xml` and `anim/slide_in_right.xml`; reduced-motion preference snaps to the destination. Thunderbird uses a short directional entrance for its local screens. These are Bhoo adaptations and do not reproduce all native Android transition choreography.

References: [Sunflower home layout](https://github.com/android/sunflower/blob/99eb59417bf8a95999d2334eb5dc320627b9304e/app/src/main/res/layout/fragment_view_pager.xml), [Sunflower slide](https://github.com/android/sunflower/blob/99eb59417bf8a95999d2334eb5dc320627b9304e/app/src/main/res/anim/slide_in_right.xml), [Thunderbird drawer layout](https://github.com/thunderbird/thunderbird-android/blob/3542b3e75bef57405ef0ce0a4d6c4655d7bb8eb5/legacy/ui/legacy/src/main/res/layout/message_list.xml). `tools/test-bhoo-app-motion.mjs` checks the moving and reduced-motion positions.

Thunderbird's search action replaces the app bar title with a focused local search field, back action, and clear action. This mirrors the pinned upstream [SearchView menu item](https://github.com/thunderbird/thunderbird-android/blob/3542b3e75bef57405ef0ce0a4d6c4655d7bb8eb5/legacy/ui/legacy/src/main/res/menu/new_message_list_option_menu.xml); it searches only demo data. The bottom destinations use the corresponding Material Symbols and confine their state layer to the icon indicator.

Thunderbird’s three-dot action opens an anchored popup menu, based on the pinned [`message_list_option_menu.xml`](https://github.com/thunderbird/thunderbird-android/blob/3542b3e75bef57405ef0ce0a4d6c4655d7bb8eb5/legacy/ui/legacy/src/main/res/menu/message_list_option_menu.xml) and the Material Android [menu guidance](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/Menu.md). The demo implements local sort by date, subject or sender, marking messages in the current folder read, and its existing About dialog. Server actions and unsupported original menu entries are omitted. The popup uses a Bhoo approximation of native opening motion.

The Thunderbird popup is positioned beneath the app bar, with a surface-container color, 3dp elevation, 48dp rows and 16sp body text in line with the pinned [Android overflow-menu example](https://github.com/material-components/material-components-android/blob/66b0eeb53afeb6025ffbbd37734b08e5bce197ed/docs/components/assets/menu/menus-overflow.png). A trailing chevron identifies the local sort submenu and a check marks the current sort. Hover color is not retained for touch input. The transition is still a Bhoo approximation, not a measured Android animation match.

The Thunderbird popup retains a visible pressed state while a menu item is held, clears it after cancellation, and activates on release. Reduced motion retains static press feedback. The host keeps the touch pointer type across animation frames and clears it when touch ends; this avoids incorrectly applying a persistent mouse hover after touch. Android ripple fidelity has not been measured against a physical Android reference.
