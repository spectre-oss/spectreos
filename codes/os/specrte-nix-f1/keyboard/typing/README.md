# Adaptive touch model

`spectre-touch.h` ports the supplied `TouchModel.js`: plateau distances over the
central 20% of each key, Gaussian widths of 38% of the key dimensions, and the
same `0.7 * log(prior)` adjustment from the same demo vocabulary. Commands are
selected by their hit rectangle before any character scoring. The artwork and
native key hit regions are separate; selecting a pixel still emits real input.

This is the user's prototype calibration and demonstration word list. It is
not a trained Gboard model, a full autocorrect dictionary, or swipe decoding.
Persian benefits from distance scoring; the English demonstration vocabulary
does not provide Persian word priors.

Stevia uses text before the cursor when the client supplies it, and updates a
bounded current-word prefix during virtual-keyboard input. It resets that state
on focus/purpose changes, disables priors for password and PIN fields, and does
not persist the text. Set `SPECTRE_ADAPTIVE_TYPING=0` in the OSK service environment
to disable priors while retaining distance scoring.

Native press/release handling retains the initial key decision, matching the
QML model. Touch cancellation emits no character. Language tap switches between
English/Persian; holding language or comma opens emoji; holding space opens the
language menu. Long-press alternatives and caps lock still use native Stevia.

Run `uv run --with quickjs python tests/test_keyboard_touch.py` to compare the
C scorer against the supplied JavaScript on 3,000 deterministic random samples,
and check command keys, bounded prefixes and invalid coordinates. Device input
and password/focus tests are separate from the mathematical parity test.

The supplied `erase()` cluster rule is also ported. With surrounding text from
a normal input-method client, Backspace removes emoji joiner sequences, skin
tones, flag pairs and the prototype's combining-mark ranges together. Selections,
password/PIN fields, word-deletion mode and clients without surrounding text use
the native Backspace path. This mirrors the supplied prototype's cluster rule;
it does not claim complete Unicode grapheme-segmentation conformance.
