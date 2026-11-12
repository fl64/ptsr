# keyboard Specification

## Purpose
Define keyboard actions, owner key overrides and input priorities. Keep pad keys reserved for play, commands consistent across screens and keymap warnings visible.

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
The system SHALL embed shipped keys and overlay `keymap.toml` from the user config folder (`~/.config/ptsr/`), regardless of starting folder.
Each owner entry SHALL replace that section's shipped binding, with `none` removing it.
`pad.scope_next` SHALL mean `pad.mark_bank` in keymaps and learned MIDI bindings.
An unreadable file SHALL be named with its line in status, with shipped keys used instead.

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

### Requirement: Owner keymaps preserve pad keys

As in the shipped keymap, owner commands on pad keys alone or with Shift SHALL be refused and named in status.
A pad key with `Ctrl`, such as `Ctrl+A`, SHALL be a command key.

#### Scenario: Pad keys stay pads
- **WHEN** the user keymap binds `Q` to a command
- **THEN** `Q` still plays its pad, and the status line says a pad key cannot be a command

### Requirement: Key meaning is consistent

The shipped pad block (`1234 QWER ASDF ZXCV`) SHALL remain pads, alone or with Shift.
Shift on a pad SHALL give full velocity, not a command.
Bound Ctrl/Alt chords SHALL have the same action or role across screens.
A shipped-map test SHALL compare action-name suffixes and allow only named exceptions with reasons.
The shipped map SHALL NOT use Ctrl+Shift+letter chords.

#### Scenario: Shipped modified chords have one role
- **WHEN** the shipped map is checked across screens and edit layers
- **THEN** every Ctrl/Alt chord has the same action suffix or an allowed role with a reason
- **AND** no Ctrl+Shift+letter command or plain/Shift pad-block command is present

### Requirement: Command key roles across screens

I/K and Up/Down SHALL move selection, row or zoom.
Enter SHALL be the main action, Esc back, Backspace removal, M marking or cutting, and bound Tab/Shift+Tab next/previous page or group.
On Pads and FX, J/L and Left/Right SHALL change the selected parameter with edit open or closed.
A bound Ctrl+D SHALL mean duplicate or double.

#### Scenario: Shift H is a distinct command in both input forms
- **WHEN** Chop receives H and Shift+H as extended key events or legacy lowercase and
  uppercase characters in either keyboard layout
- **THEN** H resolves to `chop.hits` and Shift+H to `chop.hits_back`
- **AND** Shift+Tab, existing Ctrl/Alt chords and full-velocity pads keep their behavior

### Requirement: Command layouts and Shift fallback

New command chords SHALL work in English and Russian layouts, with and without the extended keyboard protocol, or state their replacement.
An unbound Shift command chord SHALL act as its plain key in both input forms.
Without a previous-page binding, Shift+Tab SHALL keep its existing Tab fallback.
An explicit `none` SHALL block either fallback and run nothing.

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

### Requirement: Recording key roles

Alt+R SHALL record the microphone and Ctrl+T the output. Ctrl+R SHALL toggle pattern recording through `pattern.record` on every current screen. The commands SHALL remain distinct and explicit owner bindings SHALL retain priority.

#### Scenario: Ctrl C copies instead of quitting
- **WHEN** Pads receives Ctrl+C in raw mode, with or without the extended keyboard protocol
- **THEN** `pad.copy` is resolved, no quit is requested and Ctrl+Q remains the quit command

#### Scenario: Ctrl R records the pattern on every screen
- **WHEN** default Ctrl+R arrives on Pads, Files, Chop, FX or Settings outside a blocking input mode
- **THEN** it starts count-in when stopped, starts overdub when playing or ends the current recording
- **AND** no microphone recording starts and no screen changes

### Requirement: Reserved keyboard chords

The shipped map SHALL leave Ctrl+Y and Ctrl+. free until redo and panic in 93a. Ctrl+R SHALL run pattern recording in 97. Chop Ctrl+Y MAY explain its former role. Ctrl+B SHALL keep the Files favorite action. Bake SHALL NOT take it in this change. Owner bindings SHALL remain allowed to use reserved chords.

#### Scenario: Future commands stay unbound
- **WHEN** the default map is loaded after pattern recording ships but before redo and panic
- **THEN** Ctrl+Y and Ctrl+. execute no action, except a moved-key notice where defined
- **AND** Ctrl+R records patterns and Ctrl+B still changes the favorite on Files

### Requirement: Moved key notices

The App SHALL keep Chop Ctrl+N to Ctrl+G, Chop Ctrl+Y to H, and Pads J/L to U/O move notices. Only an old press without an applicable shipped binding or explicit owner entry SHALL show its Info notice. It SHALL NOT run the former action. Bound J/L SHALL change a value without a move notice. Ctrl+R SHALL no longer produce the microphone move notice.

#### Scenario: Old microphone key names its replacement
- **WHEN** Ctrl+R is unbound through an explicit owner removal after pattern recording ships
- **THEN** no recording or microphone move notice appears

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

### Requirement: Moved-key notice priority and lifetime

Explicit owner entries, including `none`, SHALL win over moved-key notices.
Text, questions and learn SHALL keep priority without notices replacing their instructions.
Notices SHALL use the current flash until the new notice model exists.
The Ctrl+R row SHALL leave when 97 binds pattern recording.
The remaining table SHALL leave in the first change after version 0.2.

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

At start, owner entries hiding new or changed shipped cleanup bindings SHALL warn once per chord and screen.
These include Alt+R, Chop Ctrl+G, H, Shift+H, and Pads U/O, Ctrl+C, J/L, Left/Right.
Warnings SHALL name `keymap.toml`, chord, screen and hidden action, for example `keymap.toml: H on chop hides chop.hits`.

#### Scenario: An owner key hides a new default
- **WHEN** the owner maps H on Chop to another action
- **THEN** startup warns once that H on Chop hides `chop.hits`, and H keeps the owner's action

### Requirement: Shadow warnings preserve owner entries

The owner's action or explicit removal SHALL win, without rewriting the file.
An owner entry for the same action SHALL NOT warn.
Multiple startup warnings SHALL be delivered individually, without silently overwriting each other.

#### Scenario: Matching owner keys do not warn
- **WHEN** the owner's Chop H binding is already `chop.hits`
- **THEN** no shadow warning is shown for H

#### Scenario: All changed default conflicts are reported
- **WHEN** the owner has several entries that hide changed defaults, including `none`
- **THEN** each conflicting chord and screen is named once through startup notices
- **AND** every owner entry stays effective and their file stays unchanged

### Requirement: Keymap warning queue and dismissal

Keymap warnings SHALL use their own queue on the current notice line.
A new press SHALL dismiss only the visible warning without consuming its normal action.
Repeats, releases and modifier-only events SHALL NOT dismiss warnings.
Warnings hidden by text, questions, learn or higher-priority notices SHALL stay pending.
Other startup notices, including terminal warnings, SHALL keep their dismissal rules.

#### Scenario: Only a visible keymap warning is dismissed
- **WHEN** several keymap warnings wait and the owner presses a key while one is visible
- **THEN** only that warning leaves the queue and the key keeps its normal action
- **AND** repeats, releases and modifier-only events do not dismiss the next warning

#### Scenario: Hidden keymap warnings stay pending
- **WHEN** a keymap warning is hidden by text input, a question, learn or a higher-priority notice
- **THEN** keyboard events do not dismiss that warning
- **AND** it is shown when the blocking state ends, without changing other startup notices

### Requirement: Shadow warnings require shipped bindings

Only new or changed bindings present in the shipped map SHALL cause shadow warnings.
Planned bindings SHALL NOT warn before they ship.

#### Scenario: Planned defaults do not produce shadow warnings
- **WHEN** an owner entry uses a cleanup chord whose new shipped binding is not present yet
- **THEN** no shadow warning is shown for that planned binding

### Requirement: Transport keyboard controls

Shipped Space SHALL toggle transport on Pads, FX and Settings through `transport.toggle`. Chop and Files Space SHALL keep preview playback. Ctrl+Up/Down SHALL change tempo by 1 BPM. Ctrl+Shift+Up/Down SHALL change it by 0.1 BPM. Alt+M SHALL toggle the metronome. These new controls SHALL work in English and Russian layouts. Commands SHALL NOT occupy plain or Shift pad keys.

#### Scenario: Space controls transport without replacing previews
- **WHEN** the owner presses Space on Pads, FX or Settings and then on Chop and Files
- **THEN** the first three screens toggle transport and the preview screens retain their previous Space actions

#### Scenario: Tempo and metronome keys work in both layouts
- **WHEN** keyboard events use Ctrl+Up/Down, Ctrl+Shift+Up/Down and Alt+M in either supported layout and input protocol
- **THEN** tempo changes by the specified amount or the metronome toggles once
- **AND** tempo holds at 40.0 and 300.0 BPM without wrapping

#### Scenario: Holding Space does not repeatedly toggle transport
- **WHEN** the owner presses Space, holds it through keyboard repeats and releases it
- **THEN** transport toggles only once

### Requirement: Transport keys preserve owner assignments

Existing owner keymap entries and `none` SHALL retain priority. The old `transport.play_stop` action SHALL keep its Chop preview meaning. `metronome.click` SHALL remain available without a new shipped key. New defaults hidden by owner entries SHALL use the existing startup warning queue without rewriting the keymap.

#### Scenario: Previous click and preview bindings retain their meaning
- **WHEN** an owner keymap binds Space to `metronome.click` or binds another key to `transport.play_stop`
- **THEN** those bindings keep the test click and Chop preview behavior without becoming general transport toggles

#### Scenario: Owner overrides hide new transport defaults with a warning
- **WHEN** owner entries change or remove the shipped Space, tempo or metronome bindings
- **THEN** their entries remain effective and startup warnings name the hidden defaults once per chord and affected screen

### Requirement: Transport help and input priority

Bottom hints and F1 SHALL name effective transport, tempo and metronome controls where applicable. The edit band SHALL keep its EDIT-only layout, with transport keys in F1. Text inputs, questions, pickers, screen learn and MIDI learn SHALL keep their existing input priorities. Selecting a transport command in screen learn SHALL NOT execute it.

#### Scenario: Transport input respects existing overlays
- **WHEN** transport keys arrive while text, a question, picker or learning mode owns input
- **THEN** that mode retains its existing behavior without starting transport or changing tempo through the underlying screen
- **AND** screen learn can select the effective transport command without executing it

#### Scenario: Help follows effective transport bindings
- **WHEN** shipped or owner transport keys are effective on the current screen
- **THEN** the bottom hints and F1 name those controls without replacing the screen's preview hints

### Requirement: Pitch Gate keyboard toggle

Shipped Alt+G on Pads SHALL run `pad.pitch_gate_toggle` with edit open or closed. It SHALL toggle once per press, not on repeat or release. It SHALL work in English and Russian layouts with extended and legacy keyboard input. Existing pad keys, pages, knobs, transport and preview bindings SHALL remain unchanged. Other screens SHALL gain no shipped Alt+G action.

#### Scenario: Pitch Gate keyboard toggle works in both layouts
- **WHEN** Pads receives Alt+G presses, repeats and releases in both supported layouts and input forms, with edit open or closed
- **THEN** each press changes Pitch Gate once for the existing scope and one undo restores that scope
- **AND** repeats and releases do not toggle it again or play a pad

### Requirement: Pitch Gate keys preserve input priorities

Text, questions, pickers and learning SHALL retain their existing input priorities. F10 SHALL select the effective Pitch Gate command without executing it. Owner keymap entries, including `none`, SHALL win without file rewriting. A hidden shipped Alt+G SHALL use the existing startup warning queue. Hints and F1 SHALL use effective bindings.

#### Scenario: Pitch Gate input cannot bypass overlays or owner keys
- **WHEN** Alt+G reaches text, a question, picker or learning, or an owner entry changes or removes its shipped action
- **THEN** existing input priority or that owner entry wins without an underlying Pitch Gate toggle
- **AND** applicable startup warnings name the hidden default without rewriting the keymap

#### Scenario: Pitch Gate help and screen learn follow effective keys
- **WHEN** the owner rebinds Pitch Gate on Pads, opens help or selects its command in F10
- **THEN** help names the effective key and screen learn selects the command without executing it
- **AND** existing EDIT-only bottom hints keep their layout with the command available in F1

### Requirement: Full help page controls preserve ordinary input

Alt+PageUp/PageDown SHALL page the F1 key table through GLOBAL `help.page_prev` and `help.page_next` commands.
The commands SHALL support learned buttons without factory assignments.
Presses and keyboard repeats SHALL navigate, but releases SHALL NOT.
Owner bindings, `none`, input ownership, ordinary controls and Messages SHALL retain priority.
Outside the key table, the commands SHALL NOT change the screen, bank, selection or recording state.

#### Scenario: Help page keys preserve ordinary roles
- **WHEN** default or owner page keys arrive on each current screen with either keyboard protocol and edit state
- **THEN** only the open F1 key table changes page on presses or repeats, and releases do not navigate
- **AND** owner bindings and `none` in the global or active edit layer retain priority, while ordinary bank keys, pads and Messages controls retain their behavior

#### Scenario: Help page commands keep first-frame order
- **WHEN** keyboard or learned page commands follow help opening before its first drawn frame
- **THEN** the first frame applies those commands in order with the current wrapped bounds
- **AND** a tab change preserves the key-table offset, while closing and reopening discards old page requests

#### Scenario: Learned help pages retain input ownership
- **WHEN** raw MIDI uses learned page buttons with F1 open, closed or blocked by text, questions, pickers or learning
- **THEN** only an unblocked F1 key table changes page, without changing recording or the underlying screen
- **AND** button releases and earlier pad releases retain their original ownership

### Requirement: Zoom while holding a bound modifier

Chop SHALL support repeating Ctrl/Alt+I/K zoom in either layout, with slice edit open or closed.
Releases SHALL NOT zoom or move bounds.
Tempo chords, other screens, blocking modes, explicit owner bindings and `none` SHALL retain their behavior.
Legacy Tab SHALL retain its action. Alt+I SHALL replace Ctrl+I when the terminal cannot distinguish Ctrl+I from Tab.

#### Scenario: Modified zoom keys preserve routing and owner overrides
- **WHEN** Chop receives Ctrl/Alt+I/K in either layout, including repeats and releases
- **THEN** it resolves zoom presses and repeats without changing tempo bindings or other screens
- **AND** owner overrides and `none` remain effective, while legacy Tab retains its previous action

### Requirement: Pattern control keys

Alt+N SHALL toggle compact pattern controls. I/K and Up/Down SHALL select a row. J/L and Left/Right SHALL change a value row and do nothing on Clear A1. Enter SHALL run Clear A1, and Enter on a value row, Esc or Alt+N SHALL close controls. Alt+E SHALL toggle erasure in a pass and on Pads outside a pass. Esc SHALL first turn off erasure, then finish Recording or cancel CountIn. Alt+Z SHALL undo the pattern pass globally. Ctrl+Z on Pads SHALL use common history. Pad keys SHALL remain pads.

#### Scenario: Adjust a pattern without opening a step grid
- **WHEN** Alt+N opens controls on any current screen and direction keys change each parameter
- **THEN** those keys edit the shown pattern parameter without changing underlying screen parameters
- **AND** closing controls restores the original screen, bank, selection and edit page

#### Scenario: Pattern keys work in both layouts and protocols
- **WHEN** recording, controls, erasure and pattern Undo keys arrive in English or Russian with extended or legacy key input
- **THEN** their logical actions agree and plain or Shift pad keys never execute those commands
- **AND** repeats or releases do not toggle recording, controls or erasure again

#### Scenario: Enter runs Clear A1 from pattern controls
- **WHEN** the owner selects Clear A1 in pattern controls, presses L, then presses Enter
- **THEN** L changes nothing, and Enter shows the Clear A1 question without clearing
- **AND** Enter on Bars closes controls

#### Scenario: Esc leaves erasure first
- **WHEN** erasure is on during Recording and the owner presses Esc twice
- **THEN** the first Esc turns off erasure and recording continues
- **AND** the second Esc finishes Recording

### Requirement: Pattern defaults preserve owner keys

Explicit owner entries and `none` SHALL override new defaults without rewriting files. Hidden recording defaults SHALL use existing startup shadow warnings. Owner Ctrl+R mapped to `record.mic` SHALL keep microphone hold or toggle behavior. Effective help SHALL name the actual recording keys or `none`.

#### Scenario: Owner recording overrides retain their old meaning
- **WHEN** an owner binds Ctrl+R to microphone or `none` and overrides another new pattern key
- **THEN** those entries remain effective and startup names each hidden default once
- **AND** the owner file stays unchanged and Ctrl+R shows no obsolete move notice

### Requirement: Pattern commands respect active input ownership

Text, pickers, questions and learning SHALL retain their existing source-specific priorities. Selecting a pattern command for learning SHALL NOT execute it. Pending placement, marking, Chop assignment and project restoration SHALL refuse recording start. Recording SHALL refuse entry into pad-consuming placement and marking modes. Earlier releases SHALL keep their original pad addresses.

#### Scenario: Pattern commands cannot escape a blocking input mode
- **WHEN** pattern commands arrive during text, pickers, questions, learning or project restoration
- **THEN** existing input ownership remains and no underlying recording command executes
- **AND** permitted original releases still finish their sounds

#### Scenario: Placement and recording cannot consume the same pad
- **WHEN** recording starts with placement, marking or Chop assignment active, or those modes are requested during recording
- **THEN** the incompatible request refuses with a next-step instruction
- **AND** no pad is replaced, marked or accidentally recorded by that request

#### Scenario: A recording pad release keeps its original bank and screen
- **WHEN** a held pad crosses a bank change, screen change, record end or erasure toggle before release
- **THEN** the release reaches its original sound ownership and cannot create a new onset
- **AND** neither the new bank nor a newer recording pass receives the old captured release

### Requirement: Pattern view keys

Pads and the A1 overview SHALL bind Alt+Up to `pattern.view_next` and Alt+Down to `pattern.view_prev`. Alt+Up SHALL select a finer view and Alt+Down a coarser view, and both SHALL stop at the ends. The keys SHALL change only the view. Pad parameters, raw rows, history, dirty state and sound routing SHALL stay unchanged. The keys SHALL work in both supported layouts and input protocols.

#### Scenario: Pattern view keys change only the view
- **WHEN** the owner presses Alt+Up and Alt+Down on Pads in extended and legacy input, then plays a pad in either layout
- **THEN** the view becomes finer or coarser and stops at 1/64 and 1/4
- **AND** raw rows, history and dirty state stay unchanged, and the pad press still plays its pad

#### Scenario: Overview view keys share the Pads division
- **WHEN** the owner changes the view on Pads, opens the overview and changes it again
- **THEN** both screens show the same division and the view stops at 1/64 and 1/4
- **AND** neither change edits the pattern or pad parameters

### Requirement: Pattern view keys preserve owner input rules

New view defaults SHALL keep explicit owner bindings, `none`, active edit-layer precedence and existing shadow warnings. Text, pickers, questions, F1 and learning SHALL keep their input priority. Outside Pads and the A1 overview, a view command SHALL explain that it needs one of those screens. Pad-block keys SHALL remain pads.

#### Scenario: Pattern view keys preserve owner input rules
- **WHEN** owner overrides hide or move a view key, the edit layer or help is open, or a view command arrives on another screen
- **THEN** effective bindings keep priority, a hidden default gives a startup warning, and blocked commands change nothing
- **AND** a view command outside Pads and the A1 overview gives an instruction to open one of them

### Requirement: Pattern controls own the edit controls

While pattern controls are open, keyboard commands that change a pad, a slice or an effect SHALL change nothing. These include the pad mode, Pitch Gate, pad clear, rename, marks and edit pages. Pad keys SHALL still play pads. Screen, bank, transport, recording, erasure and Undo keys SHALL keep their roles.

#### Scenario: Pad commands wait while pattern controls are open
- **WHEN** pattern controls are open on Pads and the owner presses H, Alt+G, Backspace and Tab, then a pad key
- **THEN** the pad mode, Pitch Gate, sound and panel page stay unchanged
- **AND** the pad key plays its pad

### Requirement: A1 overview has a direct screen key

F5 SHALL open the A1 overview from an ordinary screen. F2 and unclaimed Esc SHALL return to Pads. Other screen keys SHALL keep their existing screen roles. The overview SHALL work with absent, empty, stopped and playing A1. Opening it SHALL NOT create a pattern or move transport. Pad keys SHALL play and select pads as on Pads. Shift on a pad SHALL give full velocity. Releases SHALL keep their original sound owner across screen and bank changes.

#### Scenario: Overview opens without editing or stopping A1
- **WHEN** the owner opens the overview with F5 while A1 is absent, empty, stopped or playing
- **THEN** it shows the correct state without creating a pattern, restarting transport or changing recorded data
- **AND** F2 or Esc returns to Pads with the same bank and view division

#### Scenario: Overview pad input keeps sound ownership
- **WHEN** a keyboard or raw MIDI pad starts a Gate in the overview and the owner changes bank or screen before release
- **THEN** the pad plays with its original velocity and its release stops the original note
- **AND** no new-bank sound or pattern edit follows the old release

### Requirement: Overview can show empty banks from the keyboard

Alt+H SHALL switch the visibility of empty bank sections while the overview is open. The key SHALL have no effect on pad mode or A1. Other screens SHALL keep their existing Alt+H behavior. Owner overrides and `none` SHALL keep their existing priority rules.

#### Scenario: Overview empty-bank key keeps pad and pattern state
- **WHEN** the owner presses Alt+H in the overview with empty banks hidden, then presses it again
- **THEN** empty sections appear and disappear without changing the current bank, pad modes or A1
