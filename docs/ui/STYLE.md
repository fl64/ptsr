# UI style (draft)

Cyberpunk: an almost black background, three state accents, everything else muted. The screen sketches in `docs/ui/` are a mood guideline, not pixel-exact. A terminal has one font size, no rounded corners and no semi-transparency.

## Colors

| Role | Color | Where |
| --- | --- | --- |
| Background | `#07080A` | whole screen |
| Panel | `#11141A` | pads, tiles |
| Frame | `#2A303A` | frames, inactive items |
| Text | `#C9D1D9` | main |
| Bright text | `#E6EAF0` | headings, values |
| Muted | `#8A929E` | labels, help |
| Cyan | `#33E1FF` | playing, selected, on |
| Magenta | `#FF5FAE` | cut markers, waiting, recording, warnings |
| White | `#F2F4F7` | hit flash, playhead |
| Yellow | `#FFE03A`, dark fill `#3D3305` | PadMode badges, mode frames, corner marks, marked pads and bank marks |

Use only these state accents. Bank tints and meters keep their separate meanings.
Yellow identifies modes where pads do more than ordinary playback: placement, copying, swapping, assignment, marking and learning.
Marking still sounds. A title badge uses dark text on its accent background. Marked pads retain their dark yellow fill.

The exception is the bank colors. Each of the 8 banks has its own muted tint. This tint colors the bank number and fill in the bank row, the pad frames and the key letters of the open bank (the open bank is bold in brackets). It shows membership, not state: the selected pad stays cyan, the pad in hand stays magenta, a marked pad is yellow, a hit is a white flash. Assign mode on the chop screen colors the bank row in its own way: the planned bank is magenta, an occupied bank is bright text, an empty bank is muted.

| Bank | Color |
| --- | --- |
| 1 | `#D98880` coral |
| 2 | `#D9A066` amber |
| 3 | `#C9C46E` olive |
| 4 | `#86C47E` green |
| 5 | `#6FC2B4` turquoise |
| 6 | `#7FA3E0` blue |
| 7 | `#A88BE0` violet |
| 8 | `#D08CC4` orchid |

## Rules

- The screen shows only what is necessary now. Details are in F1 or in pop-up values.
- Cyan means playback, selection or On. Magenta means attention, cuts or recording. Yellow means PadMode.
- Mode badges always name their state. Color alone does not distinguish copy, swap, assignment or learning.
- The screen name is an inverse badge at the top left (`F3 CHOP`).
- The help at the bottom: groups (`LISTEN`, `CUT`, `ENV`, …) → key button (light text on the gray frame color `#2A303A`) → action. The help shows a toggle state only if the screen does not show it anywhere else (the pad mode and pad volume are already in the pad panel).
- With no modifier held, the help shows only keys without modifiers. With Ctrl held: only the Ctrl combinations of the screen. With Shift held: only "pads at full velocity". In a terminal without the kitty protocol, the Ctrl combinations stay in the usual line.
- In the sketches `docs/ui/*.md`, a key in brackets `[F1]` means such a button.
- The hint line for the next step changes with the situation.
- UI texts are in English and short. Russian labels in the sketches `docs/ui/*.md` give the meaning, not the final text.

## Title badges

Each screen uses the same optional badge composition beside its fixed screen identity.
F2 PLAY and F3 LISTEN keep their ten-cell slots and existing active and idle styles.
Show at most three optional badges. An active PadMode comes first, then ASK, recording and ordinary On states.
Combine overlapping file, copy and swap states in one badge. Keep the existing keyboard and MIDI priorities.
When space is short, remove lower-priority optional badges before using compact labels.
Compact marking and assignment retain their counts. Compact EDIT retains its page, such as `EDIT FILTER` or `EDIT BOUNDS`.
Retain the filename and slice count on Chop. Badges do not move the strip, grid, bank row, right column or help band.

## Pad mode feedback

During marking, placement, copy, swap, assignment or learning, ordinary pad frames are yellow.
Hit flashes, selection, marked fill and pads in hand keep their existing styles above this mode color.
Idle key labels keep the bank tint. Retained marks alone do not make every frame yellow.
The grid keeps its existing cells, rows and spacing.

Corner marks sit at the top right of each cell. The key and pad name stay readable.
`◎` identifies an available placement target. `!` identifies a replacement that already requires confirmation.
Occupied swaps use `◎`, not a new question. Assignment shows the planned slice number, including numbers with three digits.
Copy cancellation, loading and identical whole-file placement retain their existing rules.
For overlapping operations, `!` means that at least one operation requires confirmation. It does not select the keyboard or MIDI priority.
Screen learning keeps its control labels. Chop listen mode does not show pending placement targets.

Hidden grids keep their pad line and title badge. Mode marks cannot open the grid.
Assignment keeps its existing exception: it shows the grid until assignment ends.

## Notices

The existing one-row notice line keeps its position above help. Each transient result has a level and a non-color cue.

| Level | Cue | Style | Visible time |
| --- | --- | --- | --- |
| Info | `·` | Muted | 3 seconds |
| Done | `✓` | Cyan | 3 seconds, or 5 for manual save success |
| Warn | `!` | Magenta | 6 seconds |
| Error | `✗` | Bold magenta | Until a new owner action acknowledges the result shown by a frame |

Higher levels interrupt lower levels. Equal levels replace the active result. Pending results retain their remaining visible time.
Editor prompts and ongoing recording keep their existing text and style. Hidden notices do not consume their lifetime.
Startup notices keep their existing keyboard acknowledgement. Keymap warnings retain their separate visible-warning queue.
History records each production once, including repeated requests. It does not record each frame or timer update.
Persistent `Save failed`, loading progress and fallback hints are not transient results.

## Shared status and learning

Keep the top status visible on every screen, including during screen learning.
Show recording seconds, KB, connected input labels, audio and the project when space permits.
An arrow identifies the active input. Cyan emphasis supplements it. Its one-second deadline starts at original receipt, not delayed delivery.
Compact input labels distinguish actual connected ports. A reconnect cannot inherit old activity.
Remove optional project detail first, then audio detail, then input detail.
Retain unsaved or failed-save state, recording, audio loss, nonzero xruns and the full no-key-releases warning at supported widths.
Cached text must not freeze timers, meters, recording seconds, playback or hints.

Screen learning uses the existing three-row help band, not the top status.
Use yellow for context, magenta or muted text for detail, and cyan for working controls.
Shorten labels before losing next-step controls. Do not show unavailable keys or a parameter walk on Files.

## Applied-value panel

Place the bounded panel below the title at the upper right. Name the original object, parameter and applied value.
Numeric values have a range bar. Categorical choices keep words. Configuration choices must not imply an unconfirmed live-server result.
The panel lasts one elapsed second. Higher-priority editors can hide it without pausing that clock.
Unchanged values, limits, pickup waiting and refusals do not renew it. This differs from notice time, which counts visibility.

## Questions and pictures

Questions retain ASK, the target summary, safety text and effective keyboard answers.
Show controller answers only for connected bindings that really answer the current question.
Keep keyboard answers before controller rows. Count omitted rows instead of losing safety text or answers.
Drawing must not replace the stored target used for a second confirmation check.

Remove intersecting kitty images before drawing text overlays. Restore current wave data when an overlay ends.
Keep nonintersecting images unchanged. Graphics-off mode uses readable Braille and text without picture commands.
Real terminal composition needs a separate manual check.

## Offline catalogue

`task screens` checks the registered screens and applicable modes at 120x40 and 80x26.
It writes deterministic `<screen>-<mode>-<width>x<height>.ansi` files under ignored `renders/screens/`.
A separate required-name matrix catches missing modes or sizes. Required labels and cell boundaries have drawing assertions.
New screens or modes need their applicable fixtures. Long Unicode labels and 128-slice Chop remain in the matrix.
These are offline text fixtures, not hardware, listening or kitty-composition acceptance.
See [Live checks, section 87](../LIVE-CHECKS.md#87-visual-feedback-and-screen-catalogue) for selected files and separate owner procedures.

## Keyboard

- Pads: `1 2 3 4 / Q W E R / A S D F / Z X C V`, always, on all screens, except when a text input line is open (`/` opens it, Enter or Esc closes it).
- Commands: right hand, arrows, F1–F12 (screens), Space, Enter, Esc, Tab, Backspace, Ctrl+letter.
- Shift is only pad hit velocity.
