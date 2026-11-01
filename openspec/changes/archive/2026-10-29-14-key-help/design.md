# Design — 14

## Key caps

`tui::key` draws a key as ` K `: bright text (`#E6EAF0`) on the border gray (`#2A303A`) — the owner found dark text on a bright cap too contrasty; a group of keys of one item (`↑ ↓ ← →`) is one cap. No new color: the palette in `docs/ui/STYLE.md` stays.

## Held modifiers

The program already asks the terminal for `REPORT_ALL_KEYS_AS_ESCAPE_CODES` and `REPORT_EVENT_TYPES`, so pressing and releasing Ctrl or Shift alone arrives as `KeyCode::Modifier` events. The app keeps `held_mod: Option<Ctrl | Shift>`:

- set on a modifier press, cleared on its release;
- also cleared by any key event without that modifier, so a release lost to a focus change does not leave the Ctrl view stuck;
- ignored while the text input line is open.

The Ctrl view is built from `Keymap::bindings(screen)`: every Ctrl chord of the screen with its short text; pairs (`Ctrl+,` `Ctrl+.`) share one item.

## Pairs in F1

Actions of one kind map to one row label and an order: value −/+, row up/down, bank prev/next, nudge left/right (coarse and fine), cursor left/right, slice prev/next, pad select, browser up/down, volume −/+, master volume −/+. F1 joins their keys in that order (`PgUp PgDn`, not alphabetical). The panel width comes from the widest key column plus the widest text, clamped to the screen.

## Open questions

1. **Key caps color.** Decided after the owner saw it: bright text on the border gray, not a bright cap.
2. **Ctrl combos only behind Ctrl.** Decided by the owner: without a held modifier the line has only keys without modifiers, no pointer to Ctrl either; F1 still lists everything.
