# 72 — Stale texts in the main specs

## Why

Later changes replaced some keys and some behavior, but the old texts stayed in the main
specs. Now the main specs contradict each other and the program. The owner (2026-11-01):
"fix the contradictions". Here the program is the truth: `config/keymap.toml`, the code
in `crates/` and its tests. The specs must describe the program.

The changes that replaced the old texts:

- 41: `Enter` on the files screen takes the file in hand, `Ctrl+Enter` loads it at once.
- 43: the snap `cuts` moves the cursor from cut to cut. `PgUp` `PgDn` turn the banks.
- 54: `Tab` and `Shift+Tab` turn a whole page of the edit. `I` `K` pick one parameter.
- 57: a binding of the learn mode goes to the group of the layer in sight.
- 70: `Tab` and `I` `K` also act with the pad edit closed. The pad panel folds in a short
  window.

## What changes

- pad-assignment, "Assigning a sample by hitting a pad": `Enter` takes the file in hand,
  `Ctrl+Enter` loads it onto the selected pad at once.
- pad-playback, "Pad pitch", "Pad envelope", "Pad edit mode": `Tab` turns a page, `I` `K`
  pick one parameter, `J` `L` change it in the edit. The scenarios "Pick the parameter" and
  "The page follows the picked parameter" use `K`.
- pad-playback, "Source page": the rows fold in a short window. The scenario "Every row in
  sight" asks for a window tall enough for every row.
- pad-playback, "Filter curve on the strip": the scenario "Back to the wave" turns the
  pages with `Tab` until TONE.
- pad-playback, "The pad panel in blocks": `Tab` turns the pages and `I` `K` walk the
  parameters, both in the page order.
- chopping, "Slice editing": the scenario "The cursor on a cut" moves the cursor with `L`
  in the snap `cuts`, not with `PgDn`.
- midi-learn, "Learn on the screen": `Enter` binds the control in the group of the layer,
  not always in the group of the screen.

No code, config, test or document changes. No scenario gets a new name.
