## ADDED Requirements

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
state their replacement. Shift on a pad SHALL remain full velocity, not a command.

#### Scenario: Shipped modified chords have one role
- **WHEN** the shipped map is checked across screens and edit layers
- **THEN** every Ctrl/Alt chord has the same action suffix or an allowed role with a reason
- **AND** no Ctrl+Shift+letter command or plain/Shift pad-block command is present

#### Scenario: Shift H is a distinct command in both input forms
- **WHEN** Chop receives H and Shift+H as extended key events or legacy lowercase and
  uppercase characters in either keyboard layout
- **THEN** H resolves to `chop.hits` and Shift+H to `chop.hits_back`
- **AND** Shift+Tab, existing Ctrl/Alt chords and full-velocity pads keep their behavior

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
