# keyboard Specification

## Purpose
TBD - created by archiving change 64-user-keymap. Update Purpose after archive.

## Requirements

### Requirement: Shared FX screen keys

The system SHALL open the shared FX screen with `F4`. On that screen `Tab` and
`Shift+Tab` SHALL select the next and previous shared effect block. `I` and `K` SHALL
select a parameter, and `J` and `L` SHALL change it. The pad keys SHALL play pads as on
other screens. The bottom help and F1 SHALL name the controls.

#### Scenario: Open the shared FX screen
- **WHEN** the owner presses F4
- **THEN** the shared FX screen opens and the selected block is clear

#### Scenario: Release a pad after the screen changes
- **WHEN** a MIDI control starts a gate pad, the owner opens F4 and releases that control
- **THEN** the pad stops even if that control has a different action on F4

#### Scenario: Play pads on the FX screen
- **WHEN** the shared FX screen is open and the owner hits a pad
- **THEN** the pad plays as it does on the pads screen

### Requirement: The owner's keymap
The system SHALL build the shipped keymap into the program and read the owner's keys from
`keymap.toml` in the user config folder (`~/.config/ptsr/`) over it, whatever
folder the program starts in. A key bound in a section of the user file replaces the
shipped binding of that key in that section; the action `none` takes a key away. A pad key
bound to a command, alone or with Shift, is refused as in the shipped keymap and named on
the status line; with `Ctrl` (`Ctrl+A`) it is a command key. The old
action name `pad.scope_next` is read as `pad.mark_bank`, in the keymap and in the learned
MIDI bindings. A file that cannot be read is named on the status line with its line, and
the program runs with the shipped keys.

#### Scenario: A key of my own
- **WHEN** the user keymap has `"Ctrl+G" = "pad.mark_bank"` under `[screens.pads]`
- **THEN** `Ctrl+G` marks the open bank on the pads screen, and `Ctrl+A` still does too

#### Scenario: An old action name
- **WHEN** the user keymap has `"Ctrl+G" = "pad.scope_next"` under `[screens.pads]`
- **THEN** `Ctrl+G` marks the open bank, and the status line names no error

#### Scenario: A key taken away
- **WHEN** the user keymap has `"Ctrl+A" = "none"` under `[screens.pads]`
- **THEN** `Ctrl+A` does nothing on the pads screen

#### Scenario: A broken file
- **WHEN** the user keymap has a line that cannot be read
- **THEN** the status line names the file and the line, and every shipped key works

#### Scenario: From any folder
- **WHEN** the program starts from the home folder and the user keymap binds `Ctrl+G`
- **THEN** `Ctrl+G` works, and the shipped keys work as when started from the project folder

#### Scenario: Pad keys stay pads
- **WHEN** the user keymap binds `Q` to a command
- **THEN** `Q` still plays its pad, and the status line says a pad key cannot be a command

### Requirement: Key meaning is consistent

The system SHALL keep the shipped pad block (`1234 QWER ASDF ZXCV`) for pads, alone and
with Shift. Ctrl/Alt chords SHALL mean the same action or action role on every screen
where they are bound. A shipped-map test SHALL compare their action-name suffixes and
allow only named exceptions with reasons. I/K and Up/Down SHALL move the screen's
selection, row or zoom; Enter SHALL be its main action, Esc back, Backspace removal,
M marking or cutting, and bound Tab/Shift+Tab the next/previous page or group. Where
no previous-page action is bound, the existing Shift+Tab fallback SHALL stay. On Pads
and FX, J/L and Left/Right SHALL change the selected parameter with edit open or closed.
Alt+R SHALL record the microphone, Ctrl+T the output; Ctrl+R SHALL stay reserved for
pattern recording until change 97. A bound Ctrl+D SHALL mean duplicate or double.
The shipped map SHALL NOT use Ctrl+Shift+letter chords. New command chords SHALL work
in English and Russian layouts, with and without the extended keyboard protocol, or
state their replacement. Shift on a pad SHALL remain full velocity, not a command. An
unbound Shift command chord SHALL keep acting as its plain key, in both input forms. An
explicit `none` on the chord SHALL block that fallback and run nothing, also for the Tab
fallback of `Shift+Tab`.

#### Scenario: Shipped modified chords have one role
- **WHEN** the shipped map is checked across screens and edit layers
- **THEN** every Ctrl/Alt chord has the same action suffix or an allowed role with a reason
- **AND** no Ctrl+Shift+letter command or plain/Shift pad-block command is present

#### Scenario: Shift H is a distinct command in both input forms
- **WHEN** Chop receives H and Shift+H as extended key events or legacy lowercase and
  uppercase characters in either keyboard layout
- **THEN** H resolves to `chop.hits` and Shift+H to `chop.hits_back`
- **AND** Shift+Tab, existing Ctrl/Alt chords and full-velocity pads keep their behavior

#### Scenario: An unbound Shift chord acts as its plain key
- **WHEN** a Shift command chord is not bound on the screen and its plain key is
- **THEN** the shifted press runs the plain key's action in both input forms and both layouts
- **AND** a Shift chord of the pad block stays a full-velocity pad, not a command

#### Scenario: An explicit none on a Shift chord blocks the fallback
- **WHEN** the owner sets a Shift command chord to `none` and presses it in either input form
- **THEN** no action runs for the shifted press
- **AND** the plain key without Shift keeps its own action

#### Scenario: Unbound Shift Tab keeps its fallback
- **WHEN** a screen has a Tab action but no explicit Shift+Tab action
- **THEN** Shift+Tab keeps its existing Tab fallback; no new navigation action is added

#### Scenario: Ctrl C copies instead of quitting
- **WHEN** Pads receives Ctrl+C in raw mode, with or without the extended keyboard protocol
- **THEN** `pad.copy` is resolved, no quit is requested and Ctrl+Q remains the quit command

### Requirement: Reserved keyboard chords

The shipped map SHALL leave Ctrl+R free until pattern recording in 97 and Ctrl+Y and
Ctrl+. free until redo and panic in 93a. They SHALL NOT run future placeholder actions.
The moved-key notice for Ctrl+R and Chop Ctrl+Y MAY explain their former roles. Ctrl+B
SHALL keep the Files favorite action; bake SHALL NOT take it in this change. Owner
bindings SHALL still be allowed to use reserved chords.

#### Scenario: Future commands stay unbound
- **WHEN** the default map is loaded before transport, redo and panic ship
- **THEN** Ctrl+R, Ctrl+Y and Ctrl+. execute no action, except a moved-key notice where defined
- **AND** Ctrl+B still changes the favorite on Files

### Requirement: Moved key notices

The system SHALL keep an App table for global Ctrl+R -> Alt+R (microphone), Chop
Ctrl+N -> Ctrl+G (normalize), Chop Ctrl+Y -> H (hits), and Pads J/L -> U/O (banks).
An old press SHALL show an Info flash naming the new key only if no applicable shipped
binding or explicit owner entry consumes that chord. It SHALL NOT run the former
action. A bound J/L SHALL change a value, not show a bank-move flash. An explicit owner
entry, including `none`, SHALL win. Text input, questions and learn SHALL keep their
input priority; moved-key flashes SHALL NOT replace their instructions. Notices SHALL
use the current flash until the new notice model exists. The Ctrl+R table row SHALL
leave when 97 binds pattern recording; the remaining table SHALL leave in the first
change after version 0.2.

#### Scenario: Old microphone key names its replacement
- **WHEN** Ctrl+R is unbound and the owner presses it outside text, questions and learn
- **THEN** the flash says `microphone moved to Alt+R` and no recording starts

#### Scenario: Old normalize key names its replacement
- **WHEN** Ctrl+N is unbound on Chop and the owner presses it
- **THEN** the flash says `normalize moved to Ctrl+G` and no pad gain changes
- **AND** Ctrl+N on Pads still opens the pad name input

#### Scenario: Old hit chop key names its replacement
- **WHEN** Ctrl+Y is unbound on Chop and the owner presses it
- **THEN** the flash says `chop by hits moved to H` and no cuts change

#### Scenario: Rebound bank keys change values without a move notice
- **WHEN** Pads uses the default J/L bindings and the owner presses J or L
- **THEN** the selected parameter changes and no moved-bank notice appears

#### Scenario: Owner old key wins over the move notice
- **WHEN** the owner maps Ctrl+R to `record.mic` and presses it
- **THEN** microphone recording follows that binding and no moved-key notice appears

#### Scenario: Disabled old key stays silent
- **WHEN** the owner explicitly sets Chop Ctrl+Y to `none` and presses it
- **THEN** no action or moved-key notice occurs

#### Scenario: Move notices do not interrupt other input modes
- **WHEN** an old chord is received in text input, an open question or screen learn
- **THEN** that mode keeps its existing input rules and no moved-key flash replaces its text

### Requirement: Owner keymap shadow warnings

At start, the system SHALL warn once per chord and screen when an owner keymap entry
hides a new or changed shipped binding from this cleanup. This includes Alt+R,
Chop Ctrl+G, H and Shift+H, and Pads U/O, Ctrl+C, J/L and Left/Right. A warning SHALL
name `keymap.toml`, the chord, screen and hidden action, for example
`keymap.toml: H on chop hides chop.hits`. The owner's action or explicit removal SHALL
win and the file SHALL NOT be rewritten. An entry for the same action SHALL NOT warn.
Multiple startup warnings SHALL be delivered individually, not silently overwritten.
Keymap warnings SHALL use their own queue on the current notice line. A new key press
SHALL dismiss only the warning that is visible, without consuming the key's normal
action. Repeats, releases and modifier-only events SHALL NOT dismiss a warning. A warning
hidden by text input, questions, learn or a higher-priority notice SHALL stay pending.
Other startup notices, including the terminal warning, SHALL keep their current dismissal
rules. Only a new or changed binding that is present in the shipped map SHALL produce a
shadow warning; planned bindings SHALL NOT produce warnings before they ship.

#### Scenario: An owner key hides a new default
- **WHEN** the owner maps H on Chop to another action
- **THEN** startup warns once that H on Chop hides `chop.hits`, and H keeps the owner's action

#### Scenario: Matching owner keys do not warn
- **WHEN** the owner's Chop H binding is already `chop.hits`
- **THEN** no shadow warning is shown for H

#### Scenario: All changed default conflicts are reported
- **WHEN** the owner has several entries that hide changed defaults, including `none`
- **THEN** each conflicting chord and screen is named once through startup notices
- **AND** every owner entry stays effective and their file stays unchanged

#### Scenario: Only a visible keymap warning is dismissed
- **WHEN** several keymap warnings wait and the owner presses a key while one is visible
- **THEN** only that warning leaves the queue and the key keeps its normal action
- **AND** repeats, releases and modifier-only events do not dismiss the next warning

#### Scenario: Hidden keymap warnings stay pending
- **WHEN** a keymap warning is hidden by text input, a question, learn or a higher-priority notice
- **THEN** keyboard events do not dismiss that warning
- **AND** it is shown when the blocking state ends, without changing other startup notices

#### Scenario: Planned defaults do not produce shadow warnings
- **WHEN** an owner entry uses a cleanup chord whose new shipped binding is not present yet
- **THEN** no shadow warning is shown for that planned binding
