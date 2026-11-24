# Android framework widget inventory

Generated from the public top-level `android.widget` subclasses of `android.view.View` in Android SDK API 35 (Android 15, platform revision 2).
The platform `android.jar` SHA-256 is `4566663c3876e022b4fa4ced8c8697c4ab1688267f090114fd92d027b32e619b`. Regenerate with `python3 tools/inventory-android-widgets.py /path/to/android-35/android.jar`.

This is an inventory of framework View classes, including abstract and deprecated classes. It is not a list of only recommended modern widgets. XML status identifies explicit importer tags; partial means the tag is accepted, not that its full Android appearance, attributes, behavior or motion are implemented.

This scan does not cover AndroidX, Material Components, Jetpack Compose, custom views, non-View UI APIs such as dialogs and popups, or nested classes. Those require separate inventories; see [Material catalog](components.md) and [Android XML support](android-xml.md).

| Android widget | Class | XML importer |
| --- | --- | --- |
| [AbsListView](https://developer.android.com/reference/android/widget/AbsListView) | abstract | Not supported |
| [AbsSeekBar](https://developer.android.com/reference/android/widget/AbsSeekBar) | abstract | Not supported |
| [AbsSpinner](https://developer.android.com/reference/android/widget/AbsSpinner) | abstract | Not supported |
| [AbsoluteLayout](https://developer.android.com/reference/android/widget/AbsoluteLayout) | concrete, deprecated | Partial |
| [ActionMenuView](https://developer.android.com/reference/android/widget/ActionMenuView) | concrete | Initial XML tag, host-supplied actions and overflow; exact platform layout remains open |
| [AdapterView](https://developer.android.com/reference/android/widget/AdapterView) | abstract | Not supported |
| [AdapterViewAnimator](https://developer.android.com/reference/android/widget/AdapterViewAnimator) | abstract | Not supported |
| [AdapterViewFlipper](https://developer.android.com/reference/android/widget/AdapterViewFlipper) | concrete | Initial XML timing, adapter rows and default crossfade; full animator parity remains open |
| [AnalogClock](https://developer.android.com/reference/android/widget/AnalogClock) | concrete, deprecated | Partial |
| [AutoCompleteTextView](https://developer.android.com/reference/android/widget/AutoCompleteTextView) | concrete | Partial: threshold, filtering and suggestions |
| [Button](https://developer.android.com/reference/android/widget/Button) | concrete | Partial: Material/M3 theme defaults (raised/pill chrome, uppercase wrap measurement, custom ripple), centered/RTL-aware label, compound shape/vector-metric drawables and shape/gradient/layer-list/ripple/selector backgrounds; full drawable and Android parity remain open |
| [CalendarView](https://developer.android.com/reference/android/widget/CalendarView) | concrete | Initial month grid, date limits and arrow navigation; parity remains open |
| [CheckBox](https://developer.android.com/reference/android/widget/CheckBox) | concrete | Partial: checked state, Material/M3 theme accents, stateful button tint and solid-shape drawable selectors; other drawables and Android parity remain open |
| [CheckedTextView](https://developer.android.com/reference/android/widget/CheckedTextView) | concrete | Partial: host-controlled state, mark gravity, stateful tint and solid-shape drawable selectors; other drawable types and Android parity remain open |
| [Chronometer](https://developer.android.com/reference/android/widget/Chronometer) | concrete | Partial |
| [CompoundButton](https://developer.android.com/reference/android/widget/CompoundButton) | abstract | Not supported |
| [DatePicker](https://developer.android.com/reference/android/widget/DatePicker) | concrete | Initial explicit calendar and spinner modes; parity remains open |
| [DialerFilter](https://developer.android.com/reference/android/widget/DialerFilter) | concrete, deprecated | Initial QWERTY mode state, XML child rendering and web text input; exact IME and Android parity remain open |
| [DigitalClock](https://developer.android.com/reference/android/widget/DigitalClock) | concrete, deprecated | Partial |
| [EditText](https://developer.android.com/reference/android/widget/EditText) | concrete | Partial: text input, hints, Material underline and M3 filled/outlined theme defaults; full IME, shaping and Android parity remain open |
| [ExpandableListView](https://developer.android.com/reference/android/widget/ExpandableListView) | concrete | Initial virtualized group and child list; custom adapters remain open |
| [FrameLayout](https://developer.android.com/reference/android/widget/FrameLayout) | concrete | Partial |
| [Gallery](https://developer.android.com/reference/android/widget/Gallery) | concrete, deprecated | Initial centered adapter items, bidirectional layout, velocity fling and XML spacing/opacity/duration; exact platform motion and device parity remain open |
| [GridLayout](https://developer.android.com/reference/android/widget/GridLayout) | concrete | Initial automatic and explicit placement, spans, weights and margins; complete constraints and Android parity remain open |
| [GridView](https://developer.android.com/reference/android/widget/GridView) | concrete | Partial: columns, spacing, adapter choices and virtualized scrolling |
| [HorizontalScrollView](https://developer.android.com/reference/android/widget/HorizontalScrollView) | concrete | Partial: one child, fill viewport, independent touch/fling scrolling and RTL start/direction; device physics and parity remain open |
| [ImageButton](https://developer.android.com/reference/android/widget/ImageButton) | concrete | Partial |
| [ImageSwitcher](https://developer.android.com/reference/android/widget/ImageSwitcher) | concrete | Partial |
| [ImageView](https://developer.android.com/reference/android/widget/ImageView) | concrete | Partial |
| [LinearLayout](https://developer.android.com/reference/android/widget/LinearLayout) | concrete | Partial: weights, gravity, dividers, direction, horizontal text baselines, XML opt-out and indexed child baselines; exact measurement and Android parity remain open |
| [ListView](https://developer.android.com/reference/android/widget/ListView) | concrete | Partial: string entries, choice modes, dividers and virtualized scrolling |
| [MediaController](https://developer.android.com/reference/android/widget/MediaController) | concrete | Initial XML child controls and host player actions; floating controller and device parity remain open |
| [MultiAutoCompleteTextView](https://developer.android.com/reference/android/widget/MultiAutoCompleteTextView) | concrete | Initial comma-tokenizer and cursor-based suggestions |
| [NumberPicker](https://developer.android.com/reference/android/widget/NumberPicker) | concrete | Initial three-row XML wheel with fling and editing; Android parity remains open |
| [ProgressBar](https://developer.android.com/reference/android/widget/ProgressBar) | concrete | Partial: horizontal/circular determinate and indeterminate modes, Material/M3 theme accents and M3 track colors, stateful tints; custom drawables and Android parity remain open |
| [QuickContactBadge](https://developer.android.com/reference/android/widget/QuickContactBadge) | concrete | Initial XML image and host contact activation; platform contacts integration and styling remain open |
| [RadioButton](https://developer.android.com/reference/android/widget/RadioButton) | concrete | Partial: group selection, RTL, Material/M3 theme accents, stateful button tint and solid-shape drawable selectors; other drawables and Android parity remain open |
| [RadioGroup](https://developer.android.com/reference/android/widget/RadioGroup) | concrete | Partial |
| [RatingBar](https://developer.android.com/reference/android/widget/RatingBar) | concrete | Partial: fractional ratings, indicator, RTL and stateful progress-layer tints; drawable and Android parity remain open |
| [RelativeLayout](https://developer.android.com/reference/android/widget/RelativeLayout) | concrete | Partial |
| [ScrollView](https://developer.android.com/reference/android/widget/ScrollView) | concrete | Partial |
| [SearchView](https://developer.android.com/reference/android/widget/SearchView) | concrete | Initial query, iconified state, submit behavior and local string suggestions; platform providers and parity remain open |
| [SeekBar](https://developer.android.com/reference/android/widget/SeekBar) | concrete | Partial: range, RTL, Material/M3 theme accents and M3 track colors, stateful tints, solid-shape thumbs/ticks and split track; other drawables and Android parity remain open |
| [SlidingDrawer](https://developer.android.com/reference/android/widget/SlidingDrawer) | concrete, deprecated | Initial XML handle/content, tap and drag; platform physics and parity remain open |
| [Space](https://developer.android.com/reference/android/widget/Space) | concrete | Partial |
| [Spinner](https://developer.android.com/reference/android/widget/Spinner) | concrete | Initial dropdown and dialog modes and string-array entries; full platform parity remains open |
| [StackView](https://developer.android.com/reference/android/widget/StackView) | concrete | Initial adapter-backed stack and vertical swipes; platform motion and parity remain open |
| [Switch](https://developer.android.com/reference/android/widget/Switch) | concrete | Partial: checked state, RTL, Material/M3 theme accents, state text and stateful thumb/track tints; arbitrary drawables and Android parity remain open |
| [TabHost](https://developer.android.com/reference/android/widget/TabHost) | concrete, deprecated | Initial mixed XML view-ID and lazy XML factory tabs; activities and Android parity remain open |
| [TabWidget](https://developer.android.com/reference/android/widget/TabWidget) | concrete, deprecated | Initial XML child tabs, selection and RTL; platform styles and TabHost integration remain open |
| [TableLayout](https://developer.android.com/reference/android/widget/TableLayout) | concrete | Initial shared columns, stretching, shrinking and collapsing; Android parity remains open |
| [TableRow](https://developer.android.com/reference/android/widget/TableRow) | concrete | Initial column indices and spans within TableLayout; Android parity remains open |
| [TextClock](https://developer.android.com/reference/android/widget/TextClock) | concrete | Partial |
| [TextSwitcher](https://developer.android.com/reference/android/widget/TextSwitcher) | concrete | Partial |
| [TextView](https://developer.android.com/reference/android/widget/TextView) | concrete | Partial: text layout, colors, local text appearance and solid compound drawables; full typography and Android parity remain open |
| [TimePicker](https://developer.android.com/reference/android/widget/TimePicker) | concrete | Initial explicit spinner and radial clock modes; parity remains open |
| [ToggleButton](https://developer.android.com/reference/android/widget/ToggleButton) | concrete | Partial: state labels, text styling and disabled indicator opacity; drawable and Android parity remain open |
| [Toolbar](https://developer.android.com/reference/android/widget/Toolbar) | concrete | Initial XML title, navigation (including rasterized vector drawables), Material elevated / M3 flat theme surfaces, custom children and menu actions; full menu and Android parity remain open |
| [TwoLineListItem](https://developer.android.com/reference/android/widget/TwoLineListItem) | concrete, deprecated | Partial: RelativeLayout-based XML and standard text IDs |
| [VideoView](https://developer.android.com/reference/android/widget/VideoView) | concrete | Initial XML surface, linked controller and browser-hosted offline decoding; native decoding and Android parity remain open |
| [ViewAnimator](https://developer.android.com/reference/android/widget/ViewAnimator) | concrete | Partial |
| [ViewFlipper](https://developer.android.com/reference/android/widget/ViewFlipper) | concrete | Partial |
| [ViewSwitcher](https://developer.android.com/reference/android/widget/ViewSwitcher) | concrete | Partial |
| [ZoomButton](https://developer.android.com/reference/android/widget/ZoomButton) | concrete, deprecated | Partial |
| [ZoomControls](https://developer.android.com/reference/android/widget/ZoomControls) | concrete, deprecated | Partial |

The base [`android.view.View`](https://developer.android.com/reference/android/view/View) is also accepted partially by the XML importer. XML support for a superclass does not imply support for its subclasses.
