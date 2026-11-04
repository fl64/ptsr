# chopping Specification

## Purpose
Cutting a sample into slices on the chop screen (F3) and laying the slices out on pads, from the keyboard and from controllers, without changing the source file.

## Requirements

### Requirement: Chop screen
Chop SHALL open the selected pad's sample and show its wave, cuts, selected slice and cursor.
Only owner actions (cursor keys, snap, slice selection) SHALL move the cursor.
It SHALL appear as `!` above the wave, with its time on the slice line.
Playback and cursor edits SHALL use it.
A thin playhead SHALL run while the sample plays and disappear when it stops, without moving the cursor.

#### Scenario: Open from a pad
- **WHEN** the selected pad has a sample and the owner presses F3
- **THEN** the chop screen shows the waveform of this sample, drawn with Braille characters

#### Scenario: Empty pad
- **WHEN** the selected pad has no sample and the owner presses F3
- **THEN** the screen does not change, the status says "pad is empty"

#### Scenario: The cursor is visible
- **WHEN** the chop screen is open
- **THEN** the `!` marks the cursor above the wave and the slice line names its time in the sample

#### Scenario: The playhead does not take the cursor
- **WHEN** the cursor stands at 1.0 s and the owner presses play and lets it run to 3.0 s
- **THEN** a thin line runs from 1.0 s to 3.0 s, the `!` and the slice line stay at 1.0 s

### Requirement: Equal-parts auto-chop
The system SHALL chop a sample into equal parts (2, 4, 8, 16, 32); one command switches to the next count.

#### Scenario: Eight parts
- **WHEN** the owner presses the auto-chop command until the count is 8
- **THEN** the sample has 7 markers at equal distances, the header says "8 slices"

### Requirement: Chopping in time
The system SHALL play the sample with a moving playhead and set a marker at the playhead on the "cut" command while it plays (at the cursor when it does not); the cut does not move the cursor.

#### Scenario: Cut while playing
- **WHEN** the sample plays and the owner presses "cut"
- **THEN** a marker appears where the playhead is, the number of slices grows by one, and the cursor stays where it was

### Requirement: Slice editing
The system SHALL support cursor movement, fine taps and accelerated holds to move the selected slice's start, cut deletion and undo without a mouse.
A cursor exactly on a cut SHALL select the slice to its right.
The slice under the cursor SHALL play when the cursor stops on it and stay selected while the cursor moves within it.

#### Scenario: The cursor selects
- **WHEN** the owner presses → and the cursor enters the next slice
- **THEN** that slice becomes selected and highlighted, and plays once

#### Scenario: Select a slice
- **WHEN** the owner presses the "next slice" action (from a controller)
- **THEN** the next slice is selected, highlighted on the waveform and plays once

#### Scenario: Coarse move
- **WHEN** the owner holds the key that moves the start of the selected slice
- **THEN** the first repeat moves the cut by 1/128 of the visible wave, the step grows over
  6 repeats to 1/32 per repeat and stays there: about a screen in 1.3 s at the system's 25
  repeats a second, at any zoom

#### Scenario: Fine adjustment
- **WHEN** the owner taps the key that moves the start of the selected slice
- **THEN** the cut moves by 1/128 of the visible wave, about one cell of the screen; a tap is
  the fine step, holding speeds up

#### Scenario: Trim the sample start
- **WHEN** the first slice is selected and the owner moves its start to the right
- **THEN** the sample starts later, the silence before the first hit is not played; the file does not change

### Requirement: Delete a slice bound

`Backspace` and `Ctrl+Backspace` SHALL delete the selected slice's start cut, and `Alt+Backspace` its end cut.
These SHALL match the bounds lit by held `Ctrl` and `Alt`.
A region edge SHALL NOT be deleted.
When the cursor is exactly on a cut, `Backspace` SHALL delete that cut.

#### Scenario: Undo a delete
- **WHEN** the owner deletes the cut at the start of a slice and then presses undo
- **THEN** the cut is back at its old position

#### Scenario: Delete the lit bound
- **WHEN** slice 3 is selected and the owner holds Alt (its end lights) and presses Backspace
- **THEN** the cut between slices 3 and 4 is gone, slices 3 and 4 are one slice, and undo brings the cut back

#### Scenario: The cursor on a cut
- **WHEN** the snap is `cuts`, the owner moves the cursor onto a cut with `L` (or with the
  "next cut" action of a controller) and presses Backspace
- **THEN** that cut is gone, and the slices on both sides of it are one slice

### Requirement: Slice boundaries from knobs
The system SHALL let a knob set the start and another knob the end of the selected slice, inside the space between the neighbouring cuts, with soft takeover for absolute knobs.

#### Scenario: Turn the start
- **WHEN** slice 3 is selected and the owner turns the "slice start" knob
- **THEN** the cut between slices 2 and 3 follows the knob, from just after the start of slice 2 to just before the end of slice 3

#### Scenario: Another slice selected
- **WHEN** the owner selects another slice and turns the knob from a position far from the new cut
- **THEN** the cut does not jump; it starts to follow once the knob reaches its position, and the status says which way to turn

### Requirement: Listen mode
In Chop listen mode, bank actions SHALL change banks.
Until a slice of this file lies on a pad, the grid SHALL act as a slice keyboard.
Key k SHALL play and select slice 16·b + k, where b is the selected bank.
The row above SHALL name the slice range (`slices 1–16`), and each cell SHALL draw its slice's wave.

#### Scenario: Before the layout
- **WHEN** a sample is cut into 8 slices, none of them lies on a pad, and the owner hits the third pad key on the chop screen
- **THEN** slice 3 plays and is selected, the grid names the cells `slice 1`…`slice 8`, and the row over it says `slices 1–8`

### Requirement: Listen with assigned slices

With slices of this file on pads, Chop listen SHALL show and play the current bank's real pads as Pads does.
A pad holding a slice of this file SHALL name it (`slice 3`), and other pads SHALL be dimmed.
If the current bank has none, Chop SHALL open on a bank holding them.
Unassigned slices SHALL remain audible through cursor arrows and jumps by cuts.

#### Scenario: Check a slice
- **WHEN** slices 1–8 of a break lie on pads 1–8 of bank 2, bank 1 is shown, the owner opens the break on the chop screen from the files screen and hits the third pad key
- **THEN** the grid shows bank 2, the third pad plays slice 3 from its start to its end without a click, and the dimmed pads of bank 2 play what they hold

### Requirement: Assign mode
The system SHALL, on the "assign" command, show the real pads of the current bank with what lies on them, let the owner change banks, and show a plan: which slice goes to which pad and which pads it replaces. The plan is made by hand, pad by pad, or all in order from a start pad the owner hits; it is applied by a separate command, and an application that would overwrite pads asks first, showing what lies there. Leaving the mode changes nothing.

#### Scenario: See what is taken
- **WHEN** the owner enters the assign mode
- **THEN** each pad of the bank shows its sample name or that it is empty, the planned pads show the slice number, the pads that would be replaced are highlighted, and a summary says how many slices go to which banks and how many pads are replaced

#### Scenario: By hand
- **WHEN** the owner selects slice 1 and hits pad 1, then hits pad 2
- **THEN** slice 1 is planned on pad 1, slice 2 on pad 2 (the selection moved on after the first hit); slices not planned are not placed

#### Scenario: Start the chain on another pad
- **WHEN** the owner switches to "all in order", turns to bank 3 and hits pad 5
- **THEN** slice 1 is planned on pad 5 of bank 3 and the rest follow from there; nothing is planned before a start pad is hit

#### Scenario: Nine slices
- **WHEN** a break is cut into 9 slices, the plan is all in order from pad 1 of an empty bank 3 and the owner applies it
- **THEN** the slices go to pads 1–9 of bank 3 at once, pads 10–16 are not touched, the status says "slices placed on bank 3"

#### Scenario: More slices than pads to the end of the bank
- **WHEN** 9 slices are applied in order starting from pad 13 of bank 1
- **THEN** slices 1–4 go to pads 13–16 of bank 1, slices 5–9 to pads 1–5 of bank 2, the status says "slices placed on banks 1–2"

#### Scenario: Overwrite only after a question
- **WHEN** the plan covers pads Q and W of bank 2 that hold "kick" and "amen 3" and the owner applies it
- **THEN** nothing changes yet; a question shows which pads would be overwritten and what lies on them ("bank 2, pad Q: kick → slice 1", "bank 2, pad W: amen 3 → slice 2"); confirming places the slices, declining leaves the pads and the plan as they were

#### Scenario: Out of pads
- **WHEN** the slices do not fit before the end of bank 8
- **THEN** the extra slices are not assigned, the summary and the status say how many slices did not fit

#### Scenario: Hear the plan
- **WHEN** the owner hits a pad in the assign mode
- **THEN** the slice planned for that pad plays

#### Scenario: Leave without changes
- **WHEN** the owner leaves the assign mode without applying
- **THEN** no pad changes

#### Scenario: Slice playing on a pad is replaced
- **WHEN** a pad plays a slice and a plan with another slice for this pad is applied
- **THEN** the playing sound continues or stops with a fade, without a click; the next hit plays the new slice

### Requirement: Slices in the session
The system SHALL save slices in the session and restore them at start; sessions from earlier versions SHALL open with whole-file pads.

#### Scenario: Restart
- **WHEN** slices are on pads and the program is restarted
- **THEN** each pad plays the same slice as before and is named after it, e.g. "break 5"

#### Scenario: Old session
- **WHEN** the session file was saved before slices existed
- **THEN** it opens, every pad plays its whole file as before

### Requirement: Chop actions are learnable
The system SHALL list all chop screen actions, including the slice start and end knobs and the assign mode commands, in MIDI learn under their own group, so any controller control can perform them.

#### Scenario: Learn the cut
- **WHEN** the owner opens MIDI learn and selects "chop.cut" in the CHOP group
- **THEN** the next pressed controller button performs the cut on the chop screen

### Requirement: Linked slices
The system SHALL keep the cuts of a file, also when none of its slices lies on a pad, and SHALL keep the pads with its slices linked to them, so a change of a cut changes these pads at once. A new cut inside a slice that lies on pads keeps these pads on the left part, the slice that still starts at their cut; the right part is a new slice on no pad.

#### Scenario: Fix a slice after the layout
- **WHEN** slices 1–8 of a break are on pads 1–8 and the owner opens pad 3 with F3 and moves the start of slice 3 by 20 ms to the right
- **THEN** pad 2 plays 20 ms longer and pad 3 starts 20 ms later, without laying the slices out again

#### Scenario: Open the whole break from a slice
- **WHEN** the owner selects a pad with slice 5 of a break and presses F3
- **THEN** the chop screen shows the whole break with all its cuts and slice 5 selected

#### Scenario: Join two slices
- **WHEN** pads 2 and 3 hold slices 2 and 3 and the owner joins slice 3 with slice 2
- **THEN** pads 2 and 3 both play the joined slice, and the pads with later slices keep playing the same sound under numbers one lower

#### Scenario: Chop again into equal parts
- **WHEN** pads hold slices of a break and the owner replaces its cuts with equal parts
- **THEN** these pads keep playing what they played and are no longer linked, the status says how many pads were unlinked

#### Scenario: Undo the new chop
- **WHEN** the owner has replaced the cuts with equal parts and presses undo
- **THEN** the old cuts are back and the pads are linked to them again

#### Scenario: Cut inside a slice on a pad
- **WHEN** slice 1 of a break lies on pad Q and the owner cuts inside slice 1
- **THEN** Q plays slice 1 from its start to the new cut and stays linked, the status says "pad 1-Q is shorter now; Ctrl+Z returns"; the right part is slice 2 on no pad; pads with later slices stay linked under numbers one higher

#### Scenario: The status says the pads changed
- **WHEN** slices are on pads and the owner moves, deletes or adds a cut or a region edge that bounds them
- **THEN** the status says how many pads changed and that Ctrl+Z returns them; after undo it says how many pads changed back

#### Scenario: Undo the cut
- **WHEN** the owner undoes the cut inside slice 1 of pad Q
- **THEN** the cut is gone and Q is linked to slice 1 again

#### Scenario: Restart
- **WHEN** the program is restarted
- **THEN** the cuts of each chopped file and the links of its pads are restored

#### Scenario: Chopped, nothing laid out
- **WHEN** the owner cuts a sample, puts no slice on a pad and restarts the program
- **THEN** the chop screen of that sample shows the same cuts

### Requirement: Chop from the files screen
The system SHALL open the selected file of the files screen on the chop screen without putting it on a pad first.

#### Scenario: Chop a found break
- **WHEN** the owner selects a file on the files screen and presses F3
- **THEN** the chop screen opens with this file, no pad changes until a plan is applied

### Requirement: Play from the selected slice
The system SHALL start playback on the chop screen from the cursor, every time, as the play mode says (see "Play modes"); stopping does not move the cursor, so the next play starts from the same place.

#### Scenario: Listen from the middle
- **WHEN** the cursor stands at slice 5 and the owner presses play
- **THEN** playback starts there and goes on to the end of the region, or of the sample when no region is set

#### Scenario: The cursor survives a pause
- **WHEN** the owner stops the playback
- **THEN** the cursor stays where the owner put it and the next play starts from it again

### Requirement: Pad hit selects the slice
The system SHALL, in the listen mode, make the slice of this file played by a pad hit the selected slice, so the next edit acts on it; a hit on a pad that plays no slice of this file selects nothing.

#### Scenario: Fix the slice just heard
- **WHEN** slice 6 lies on pad 6 of the bank shown and the owner hits it and then moves the slice start
- **THEN** slice 6 is selected and highlighted, and its start moves

### Requirement: Assign slice by slice
The system SHALL open the assign mode with an empty plan made by hand and the slice selected before it opened; each pad hit puts the selected slice on that pad and selects the next slice not yet planned, until all slices are planned.

#### Scenario: Tap pads in turn
- **WHEN** a sample is cut into 4 slices, slice 1 is selected, the owner presses P (was Enter before 21) and hits pads Q, 1, W, E
- **THEN** slice 1 is planned on Q, slice 2 on 1, slice 3 on W, slice 4 on E, and the status says that all 4 slices are placed and Enter applies

#### Scenario: Assign the slice just heard
- **WHEN** the owner hits the fifth pad key in the listen mode, presses P (was Enter before 21), hits pad Q and presses Enter again
- **THEN** slice 5 lies on Q and no other pad changed

#### Scenario: All placed
- **WHEN** all slices are planned and the owner hits another pad
- **THEN** nothing is planned there, the pad plays what is planned for it if anything

#### Scenario: Undo a tap
- **WHEN** the owner undoes the last tap
- **THEN** that slice is no longer planned and is selected again

### Requirement: Ask before overwriting pads
The system SHALL, before any step that loses what lies on a pad and cannot be undone, show in plain words which pads lose what and what takes its place, and SHALL do the step only after the owner confirms; declining changes nothing.

#### Scenario: Apply a plan over taken pads
- **WHEN** a plan puts slices 1 and 2 on pads Q and W of bank 2 that hold "kick" and "amen 3" and the owner applies it
- **THEN** a question lists "bank 2, pad Q: kick → slice 1" and "bank 2, pad W: amen 3 → slice 2", says that the files on disk stay, and offers Enter to replace and Esc to keep

#### Scenario: Put a file on a taken pad
- **WHEN** a file is in hand on the files screen and the owner hits a pad that holds "kick"
- **THEN** a question says that "kick" on that pad will be replaced by the file; Esc keeps "kick" and the file stays in hand

#### Scenario: Steps that can be undone
- **WHEN** a step can be taken back with undo (cuts, the plan itself)
- **THEN** it is done without a question

### Requirement: Chop screen readability
The system SHALL show on the chop screen, without cutting text: the plan summary in full, the selected slice both on the waveform and, while the pads are shown, on the grid of the listen mode, empty pads in the assign mode as only "empty", and in the overwrite question each pad as "bank B, pad K: what → new".

#### Scenario: Selected slice on the grid
- **WHEN** the pads are shown and the owner hits pad W in the listen mode
- **THEN** pad W is highlighted as selected on the grid

#### Scenario: Selected slice with the pads hidden
- **WHEN** the pads are hidden and the owner hits pad W in the listen mode
- **THEN** the pad line names pad W and its slice, and the slice is selected on the waveform

#### Scenario: Plain overwrite question
- **WHEN** the plan puts slice 3 on pad 3 of bank 1 that holds "kick" and the owner applies it
- **THEN** the question says "bank 1, pad 3: kick → slice 3"

### Requirement: Equal parts go on after a restart
The system SHALL step equal parts on from the current number of slices when it is one of 2, 4, 8, 16, 32, also for a chop restored from the session.

#### Scenario: Eight slices after a restart
- **WHEN** a sample was cut into 8 equal parts, the program was restarted and the owner presses the equal-parts command
- **THEN** the sample is cut into 16 parts

### Requirement: Zoom around the cursor
The system SHALL let the owner zoom the wave view twice in or out around the cursor; the view follows the cursor, the view keeps at least 256 frames and never wider than the sample; the wave, numbers and dimming draw from the view.

#### Scenario: Zoom in and out
- **WHEN** the owner presses zoom-in twice and then zoom-out
- **THEN** the wave is a quarter of its full width and then back at the half of it; the cursor stays at the same place of the picture

### Requirement: Chop region
The system SHALL let the owner set the start and the end of the playable region from the cursor with the region actions and clear it back to the whole sample; equal parts and playback cover only the region; the region is at least one frame long.

#### Scenario: Region from I to O
- **WHEN** the owner presses `,` at frame 30_000 and `.` at frame 60_000 and then runs equal parts
- **THEN** the parts cover the region from 30_000 to 60_000, and the outside is dimmed

#### Scenario: A region keeps pads linked to the slices inside it
- **WHEN** 16 slices of a break are on pads and the owner presses `,` in the middle of the break
- **THEN** the pads with slices inside the region keep following their slices; the pads whose cuts fell outside play the slice that now covers that place, as after a delete

#### Scenario: Play the region
- **WHEN** the region is set and the owner presses play
- **THEN** the sound stops at the region end, not at the end of the file

#### Scenario: Clear and undo
- **WHEN** the owner presses `Ctrl+,` to clear the region
- **THEN** the region is the whole sample again and undo restores the previous region

### Requirement: Cursor modes
`J` `L` (← →) SHALL move the cursor within the region.
In even steps, a tap SHALL move 1/128 of the visible window, at least one frame.
Holding SHALL accelerate over 6 repeats to 1/32 of the window per repeat.
Slice bounds SHALL use the same steps at every zoom.
Zooming in SHALL give finer steps because steps follow the view.

#### Scenario: Steps follow the view
- **WHEN** the view shows the whole file and the owner presses →
- **THEN** the cursor moves by 1/128 of the file; after zooming in the same press moves it by 1/128 of the smaller view

#### Scenario: Even steps on a break
- **WHEN** the chop screen opens on a break of decaying hits and the owner presses → twelve times
- **THEN** the cursor moves twelve even steps, a peak does not hold it back

### Requirement: Cursor hit jumps

In hits mode, the cursor SHALL jump to the next or previous strong hit's start when that hit is on screen.
Without a strong hit ahead on screen, it SHALL move in even steps.
A strong hit SHALL use the shared threshold in "One rule for the hits".

#### Scenario: Hit to hit
- **WHEN** hits mode is on and the owner presses → and then ←
- **THEN** the cursor lands on the start of the next hit and then back on the start of the previous one

#### Scenario: No hits on the screen
- **WHEN** hits mode is on and no hit lies ahead of the cursor on the screen
- **THEN** → moves the cursor in even steps until a hit comes into view, and then jumps to it

### Requirement: Cursor cut jumps

In cuts mode, the cursor SHALL jump right to the next cut, or the region end when none remains.
It SHALL jump left to the previous cut, or the region start.
The slice under the cursor SHALL be selected and play, and the page SHALL follow it.

#### Scenario: Cut to cut
- **WHEN** cuts mode is on, the cursor stands inside slice 2 of 4 and the owner presses `L` twice and then `J`
- **THEN** the cursor lands on the start of slice 3, then of slice 4, then back on the start of slice 3, and each slice plays

#### Scenario: To the ends
- **WHEN** cuts mode is on, the cursor stands in the last slice and the owner presses `L`
- **THEN** the cursor lands on the end of the region; in the first slice `J` lands on its start

### Requirement: Cursor snap selection

Chop SHALL open in even steps.
`N` SHALL cycle even steps, hit to hit and cut to cut, with the mode visible in the help line.
The cut command SHALL cut at the cursor also on pause.

#### Scenario: N walks the three modes
- **WHEN** the chop screen opens and the owner presses `N` three times
- **THEN** the help line says `snap: hits`, then `snap: cuts`, then `snap: off`

#### Scenario: Fine step
- **WHEN** the owner zooms in four times with `I` and presses `L`
- **THEN** the cursor moves by 1/128 of the smaller view, a sixteenth of the step on the
  whole file

#### Scenario: Cut on pause
- **WHEN** the playback is stopped and the owner presses the cut command
- **THEN** a cut appears at the cursor, as it does while playing

#### Scenario: Small rises are passed
- **WHEN** hits mode is on and between two hits of a break a sustained sound swells by a few dB
- **THEN** → goes from the first hit to the second, the swell does not stop the cursor

### Requirement: Held keys repeat on the chop screen
The system SHALL repeat the cursor arrows, the cut nudge and zoom while their key is held; pads and the other commands act once per press.

#### Scenario: Hold the arrow
- **WHEN** the owner holds → on the chop screen
- **THEN** the cursor keeps moving until the key is released

#### Scenario: A held pad does not roll
- **WHEN** the owner holds a pad key
- **THEN** the pad plays once

### Requirement: Open on your slice
The system SHALL open the chop screen from a pad that plays a part of the file with the
view on that part: the part in the middle, half of its length shown on each side, and the
slice where the part starts selected. This holds for a pad that follows the cuts of the
chop and for a pad that plays its own piece (its start or end moved by hand, or its
source set to the pad). From a pad that plays the whole file the view is left as it was. The zoom out
shows the whole file again.

#### Scenario: Open on your slice
- **WHEN** slice 5 of a break lies on pad 2-Q and the owner presses F3 on it
- **THEN** the view shows slice 5 in its middle with half of it on each side, slice 5 is selected, and zooming out reaches the whole file

#### Scenario: Open on your own piece
- **WHEN** pad 7 played slice 3 of a break, the owner moved its start 2 ms later on the
  SOURCE page, closed the edit and presses F3 on it
- **THEN** the view shows the piece of pad 7 in its middle with half of it on each side,
  slice 3 (where the piece starts) is selected, and the chop keeps its cuts

### Requirement: The source is marked
The system SHALL mark on the chop screen where the whole file lies: the bank row puts `◆` at every bank that holds a pad playing this file whole, and on the grid such a pad says `◆ all` and is not dimmed.

#### Scenario: The source is marked
- **WHEN** pad 1-1 plays the whole break, its slices lie on bank 2 and the owner opens the chop screen from pad 2-Q
- **THEN** the bank row shows `◆` at bank 1; after `Ctrl+1` pad 1-1 says `◆ all` and is not dimmed

### Requirement: The chop wave is colored by frequency
Chop SHALL colour each selected-slice wave column by the band most above its own file average: low red, mid green, high blue.
That band SHALL contribute most of the colour.
Other slices within the region SHALL be grey at the same brightness, and the wave outside SHALL be dim.
Picture mode SHALL use the same colours.
Other screens' strips and pad waves SHALL remain uncoloured.

#### Scenario: Kick and hat by color
- **WHEN** a break with a kick and a hat is open on the chop screen and the slice with them is selected
- **THEN** the columns of the kick are red and the columns of the hat are blue, also when the three bands of the break are nearly even on average

#### Scenario: The selection stays visible
- **WHEN** slice 3 is selected
- **THEN** the wave of slice 3 is in color and the other slices are grey, with no background plate

### Requirement: The source pad stands apart
The system SHALL draw, on the chop screen, the frame of a pad that plays the whole file of
the open chop dashed (`┄` `┆`), in the colour a frame of that pad has anyway; the pads of
slices and the other pads keep solid frames. The wave and the `◆ all` note stay.

#### Scenario: The whole file among its slices
- **WHEN** pad 1-1 plays the whole break, pads 1-2…1-8 play its slices and the chop screen shows bank 1
- **THEN** pad 1-1 has a dashed frame and pads 1-2…1-8 solid ones

#### Scenario: Another file
- **WHEN** a pad of the bank plays the whole of another file
- **THEN** its frame stays solid

### Requirement: Graphic waveform
The system SHALL draw the waveforms with Braille characters by default and SHALL draw them with kitty graphics when the owner switches the picture on in the settings (F12) and the terminal supports it: the chop screen waveform. The switch is kept in the config and takes effect at the next start.

#### Scenario: Braille by default
- **WHEN** the owner starts the program in a terminal with graphics and has not switched the picture on
- **THEN** the waveforms are drawn with Braille characters

#### Scenario: The picture switched on
- **WHEN** the owner sets "Wave" to "picture" on F12 and starts the program again in a terminal with graphics
- **THEN** the waveforms are drawn as pictures

#### Scenario: The picture does not blink
- **WHEN** the owner moves the cursor across slice bounds on a terminal with graphics
- **THEN** the wave stays on the screen the whole time: a changed picture appears over the old one before the old one is taken off

#### Scenario: Terminal without graphics
- **WHEN** the terminal does not support kitty graphics
- **THEN** the waveforms are drawn with Braille characters, all functions are available

> The X-Session chop profile moved to 21-edit-mode (the parameter standard); it lands
> after the key roles and the edit pages are in place.

### Requirement: Auto-chop by hits

H SHALL cut at hit starts using a threshold below the loudest hit.
Each press SHALL cycle -6, -12, -18, -24, -30 dB, then -6 again.
`chop.threshold` SHALL set a continuous threshold and recut immediately.
H and Shift+H SHALL replace cuts with equal-parts pad-link safeguards and make one undo step when cuts or threshold change.
The status SHALL name threshold and slice count.

#### Scenario: Break by hits
- **WHEN** a break is on Chop and the owner presses H
- **THEN** cuts land on the starts of the loud hits, and the status says how many

#### Scenario: More presses, more cuts
- **WHEN** the owner presses H again
- **THEN** quieter hits get cuts too; Ctrl+Z brings back the cuts of the press before

#### Scenario: The threshold on a knob
- **WHEN** the owner turns the knob bound to `chop.threshold`
- **THEN** cuts are added or removed at once as the threshold moves

### Requirement: Reverse hit threshold

Shift+H (`chop.hits_back`) SHALL select the next higher listed threshold, stopping at -6 dB without wrapping.
Before a threshold is set, it SHALL start at -6 dB.
For a continuous value below -6 dB, it SHALL select the next higher listed value.
A value at or above -6 dB SHALL stay unchanged, so reverse never adds quieter hits.
A clamped press SHALL change nothing and add no undo step.

#### Scenario: Reverse the hit threshold
- **WHEN** the hit threshold is -18 dB and the owner presses Shift+H
- **THEN** the threshold becomes -12 dB and quieter cuts disappear
- **AND** one Ctrl+Z restores the previous cuts and -18 dB threshold

#### Scenario: Reverse hit threshold stops at the first step
- **WHEN** the threshold is -6 dB and the owner presses Shift+H again
- **THEN** cuts and threshold stay, no undo step is added, and the flash names -6 dB

#### Scenario: Reverse a continuous hit threshold
- **WHEN** the threshold knob has set -15 dB and the owner presses Shift+H
- **THEN** the threshold becomes -12 dB and the cuts follow the same hit rule

#### Scenario: Reverse keeps a higher knob threshold
- **WHEN** the threshold knob has set -3 dB and the owner presses Shift+H
- **THEN** the threshold and cuts stay and no undo step is added

#### Scenario: First reverse hit command
- **WHEN** no hit threshold has been explicitly set and the owner presses Shift+H
- **THEN** the threshold becomes -6 dB, hit cuts are made and the status names the threshold

### Requirement: Snap to hits
The system SHALL, when snapping is on, move a marker set in time to the nearest hit within 90 ms.

#### Scenario: Cut slightly off the hit
- **WHEN** the sample plays, snapping is on, the owner presses "cut" 40 ms after a hit
- **THEN** the marker lands exactly on the hit, the status says "marker snapped to hit"

### Requirement: Where a slice lies
The system SHALL name on the slice line of the chop screen the pads that hold the selected slice, as bank and key joined by a dash (`2-4`, `2-Q`) like the labels at the slice numbers (27), also when the slice is too narrow for its label.

#### Scenario: Two pads hold the slice
- **WHEN** slice 3 lies on pad Q of the current bank and pad W of bank 2 and the owner selects slice 3
- **THEN** the slice line says "on 1-Q, 2-W"

### Requirement: Chop knobs
Learned Chop knobs SHALL use the configured response for absolute values:
- `chop.cursor_pos`: cursor within the region.
- `chop.zoom`: visible span from region to whole file, logarithmically around the cursor.
- `chop.region_start_pos` / `chop.region_end_pos`: region edges, at least 10 ms apart.
MIDI learn SHALL list these under CHOP.
The X-Session Pro profile SHALL bind `chop.zoom` to k12.

#### Scenario: Zoom knob
- **WHEN** the owner turns the zoom knob
- **THEN** the view spans from the whole file to close-ups smoothly; with the response
  `pickup` grabbing the knob from another position does not jump the view, with `jump`
  the view goes to the knob position at once

#### Scenario: Region knobs
- **WHEN** the owner turns the learned region knobs
- **THEN** the edges follow as the knob response says, and the region never gets shorter
  than 10 ms

#### Scenario: Cursor knob
- **WHEN** the owner turns the learned cursor knob
- **THEN** the cursor moves inside the region, the slice under it is selected

### Requirement: Learned cursor buttons

`chop.cursor_left` / `chop.cursor_right` SHALL step the cursor as `J` `L` do.
MIDI learn SHALL list both under CHOP.

#### Scenario: Cursor buttons
- **WHEN** the owner presses the learned "cursor right" button
- **THEN** the cursor moves one step right, as with `L`

### Requirement: Normalize

The system SHALL, on Ctrl+G on Chop, set the volume of the pad the screen was opened
from so the loudest peak of what that pad plays sits at -1 dB or just below (the volume
is whole dB, rounded down), within the pad volume range (up to +6 dB). The status SHALL
show the gain and say when the limit stopped it. The file on disk SHALL NOT change.
When Chop was opened without a pad, the status SHALL say there is no pad to normalize.

#### Scenario: Quiet break
- **WHEN** the owner opens a pad whose peaks are 6 dB below full scale and presses Ctrl+G
- **THEN** the pad volume is +5 dB, the status shows "+5 dB", and the pad plays without clipping

#### Scenario: Very quiet break
- **WHEN** the pad peaks are 20 dB below full scale and the owner presses Ctrl+G
- **THEN** the pad volume is +6 dB and the status says the volume limit stopped it

#### Scenario: Twice in a row
- **WHEN** the owner presses Ctrl+G a second time
- **THEN** gain is measured again from the sound, so the level does not climb twice

#### Scenario: Chop without a pad
- **WHEN** Chop was opened from Files and the owner presses Ctrl+G
- **THEN** the status says there is no pad to normalize

### Requirement: Slice edit mode
The system SHALL open the edit mode of the selected slice with `Enter` on the chop screen
in the listen mode and close it with `Enter` or `Esc`; in the assign mode `Enter` stays
commit. The page is BOUNDS (slice start, slice end). `I` `K` pick the bound and `J` `L`
move it, the same keys as on the pads screen; a pad hit still sounds and selects the slice
it plays.

#### Scenario: Enter opens on the selected slice
- **WHEN** a slice is selected in the listen mode and the owner presses `Enter`
- **THEN** the screen names the BOUNDS page and shows what the knobs turn now

#### Scenario: Commit is untouched
- **WHEN** the assign mode is on and the owner presses `Enter`
- **THEN** the plan applies to the pads, as before

### Requirement: Assign from the keyboard
The system SHALL start the assign from the keyboard with `P` on the chop screen; `Enter`
opens the slice edit in the listen mode and stays the commit in the assign mode, and the
`chop.commit` action keeps "no plan — start, plan — apply" for device buttons. With the
slice edit open neither `P` nor a device commit starts the assign: `Esc` comes first.

#### Scenario: P starts the assign
- **WHEN** the chop screen is open in the listen mode and the owner presses `P`
- **THEN** the assign mode starts by hand from slice 1, as it did from `Enter` before

#### Scenario: No assign from the edit
- **WHEN** the slice edit is open and the owner presses `P` or the device commit button
- **THEN** the assign does not start and the slice edit stays open

### Requirement: The assign mode ignores the rest
Assign SHALL ignore Chop actions except pads, `Tab` (by hand / in order), `Enter` (apply), `Esc` (back), slice selection and undo.
`J` `L` (← →) SHALL select and play the previous or next slice and move the cursor to its start without changing cuts.
Ignored keys SHALL show "assigning: Enter applies, Esc leaves" in the status line.

#### Scenario: No stray cut during the assign
- **WHEN** the assign mode is on and the owner presses `M`, `I` `K`, `Ctrl+J`,
  `Alt+L`, `Y` and `,`
- **THEN** nothing is cut, zoomed or moved: the cuts, the cursor, the view and the plan
  stay as they were, and the status line says how to apply or leave

#### Scenario: Pick the slice for a pad
- **WHEN** slice 5 is selected in the assign mode by hand, the owner presses `J` three times and hits pad C
- **THEN** slice 2 is planned on pad C; the cuts did not move

### Requirement: Slice bounds from the keyboard
The system SHALL move the bounds of the selected slice in the listen mode with a two-hand
chord: `Ctrl+J` `Ctrl+L` move its left bound (the start), `Alt+J` `Alt+L` its right bound
(the end), the arrows with the same modifiers too; a bound steps like the cursor (see
"Cursor modes"): a tap 1/128 of the visible wave, a held key speeds up to 1/32 per repeat;
the status line says which pads changed, as after a slice edit.

#### Scenario: Ctrl and Alt name the bound
- **WHEN** the chop screen is open in the listen mode and the owner presses `Ctrl+J`, then
  `Alt+L`
- **THEN** the start of the selected slice moves a step left, its end moves a step right,
  and the pads linked to the changed slices follow

### Requirement: The cursor and the zoom on the right hand
The system SHALL, on the chop screen outside the edit mode, move the cursor with `J` `L`
(and `←` `→`) and zoom the wave around the cursor with `I` (in) and `K` (out), also
`↑` `↓`. The cursor step follows the view, so zooming in is its fine step; holding a key
repeats it. The slice under the
cursor is the selected one, as before; the slices are not walked with up and down.

#### Scenario: Zoom on I and K
- **WHEN** the chop screen is open and the owner presses `I` twice and then `K`
- **THEN** the view zooms in twice and out once around the cursor; the cursor stays

#### Scenario: Minus and equal do not zoom
- **WHEN** the chop screen is open and the owner presses `−` or `=`
- **THEN** the view does not change

### Requirement: Cut and region at the cursor
The system SHALL put the actions at the cursor on the bottom row of the right hand: `M`
cuts at the cursor, `,` starts the region at the cursor, `.` ends it there, `Ctrl+,`
makes the region the whole sample again.

#### Scenario: Region on comma and dot
- **WHEN** the cursor stands at 0.5 s and the owner presses `,`, moves the cursor to 1.5 s
  and presses `.`
- **THEN** the region is 0.5 s – 1.5 s, and `Ctrl+,` brings back the whole sample

### Requirement: Equal parts on a letter
The system SHALL step the equal parts with `Y` on the chop screen, away from the keys the
right hand moves with; `Tab` switches in order / by hand in the assign mode.

#### Scenario: Y steps the parts
- **WHEN** the chop screen is open in the listen mode and the owner presses `Y`
- **THEN** the sample is cut into the next count of equal parts

### Requirement: Play modes
`O` SHALL cycle Chop play modes:
- "to the end": cursor to region end, or sample end without a region.
- "to the cut": cursor to its slice's end.
- "the slice": the whole slice under the cursor.
A green bar directly above the wave SHALL always show the next play span and follow cursor and cuts immediately.
Help SHALL name the mode.
Chop SHALL open in "to the end" without saving the mode.

#### Scenario: To the cut
- **WHEN** the cursor stands in the middle of slice 3, the mode is "to the cut" and the
  owner presses play
- **THEN** playback runs from the cursor to the end of slice 3 and stops there

#### Scenario: The slice
- **WHEN** the cursor stands in the middle of slice 3, the mode is "the slice" and the
  owner presses play
- **THEN** the whole slice 3 plays, from its start

#### Scenario: The bar shows what plays
- **WHEN** the owner presses `O` and moves the cursor into the next slice
- **THEN** the green bar above the wave changes at once to the span the next play covers,
  and the help line names the mode

### Requirement: The chop screen opens in silence
The system SHALL fade out every sounding pad voice without a click when the chop screen
opens, so the owner hears the slices alone. The slice playback of the chop screen is not a
pad voice and is not touched; pad hits there keep playing slices, as in the listen mode;
leaving the chop screen stops nothing.

#### Scenario: A long pad fades on entry
- **WHEN** a one-shot pad plays a long sample and the owner presses F3
- **THEN** the pad fades out without a click as the chop screen opens

#### Scenario: Slices still play after the hush
- **WHEN** the chop screen opened with the pads hushed and the owner hits a pad
- **THEN** the slice of that pad plays, as in the listen mode

### Requirement: The cuts belong to the file
The Chop header SHALL name the file ("cuts of break.wav").
Each slice SHALL show its pads, excluding `all` and `pad` sources.
Opening from a slice pad SHALL select its slice.
Opening from a whole-file pad SHALL start a fresh whole-file chop without cuts (42).
The older chop SHALL stay with its pads, which keep their slices and follow its cuts (45).
Undo SHALL restore that older chop on screen.

#### Scenario: Open from a slice pad
- **WHEN** slices 1–8 of a break lie on pads 5–12 and the owner presses F3 on pad 7
- **THEN** the header says "cuts of break.wav", slice 3 is selected, and every slice shows its pad

#### Scenario: Open from the whole-file pad
- **WHEN** slices of a break lie on pads 2-1…2-8, pad 1-1 plays the whole break and the owner presses F3 on pad 1-1
- **THEN** the break opens with no cuts, the pads 2-1…2-8 play what they played and still say `slice 1`…`slice 8`, and the status says that the older chop stays with its pads

#### Scenario: Undo the fresh chop
- **WHEN** the owner presses `Ctrl+Z` right after a fresh chop opened
- **THEN** the older chop is on the screen again with its cuts, and the pads 2-1…2-8 follow their slices as all along

### Requirement: The region keeps the pads outside it
The system SHALL leave a pad that plays a slice falling outside a new region playing what it played: it gets its own bounds (`pad`) instead of moving onto a slice left in the region; the status names the pads that changed and undo links them back.

#### Scenario: A region over a laid-out break
- **WHEN** 9 slices lie on pads 1–9 and the owner sets the region start at the start of slice 9
- **THEN** pads 1–8 play what they played and say `pad`; pad 9 still follows its slice (now slice 1 of the region); undo brings the 9 slices and the links back

### Requirement: Undo keeps the slice under the cursor
The system SHALL, after an undo on the chop screen, select the slice under the cursor, so the slice line, the edit and the bounds act on the slice the cursor stands in.

#### Scenario: Edit after an undo
- **WHEN** the cursor stands in slice 9, the owner undoes a region and opens the slice edit
- **THEN** the edit is on slice 9

### Requirement: Slices on pads are marked
The system SHALL mark in the row over the wave where every slice lies: a slice on a pad gets a plate in the color of the bank of its first pad across the slice's width, with the slice number and its pads on it (`5 2-Q`); a slice on no pad shows its number with a dim `○` and no plate. The bank row of the chop screen is shown in the listen mode too and marks the banks that hold slices of this file. While slices with `○` stand next to slices on pads, the hint says how to place them.

#### Scenario: A cut leaves a slice on no pad
- **WHEN** slices 1–8 lie on pads and the owner cuts inside slice 5
- **THEN** slice 5 wears a plate in the color of its bank saying `5 1-A`, the new slice 6 shows a dim `○` and no plate, and the hint says "○ slices lie on no pad: P, J L and a pad hit place them"

### Requirement: Banks on the chop screen
The system SHALL turn the banks on the chop screen with `PgUp` `PgDn` and `Ctrl+1`…`8`,
as on every screen with pads: in the listen mode, in the assign mode and in the slice
edit. Until a slice of this file lies on a pad, the same keys page the keyboard of slices.

#### Scenario: The next bank while chopping
- **WHEN** bank 1 is shown on the chop screen in the listen mode and the owner presses `PgDn`
- **THEN** bank 2 is shown and the cursor stays where it was

#### Scenario: The same keys while assigning
- **WHEN** the assign mode is open on bank 1 and the owner presses `PgDn`
- **THEN** bank 2 is shown, as in the listen mode

#### Scenario: Pages of the keyboard of slices
- **WHEN** a sample is cut into 20 slices, none lies on a pad, and the owner presses `PgDn`
- **THEN** the row over the grid says `slices 17–20` and the first pad key plays slice 17

### Requirement: Several chops of one file
The system SHALL keep each chop with a pad-linked slice.
Fresh chops SHALL leave older chops and pads unchanged.
F3 on a pad SHALL open its slice's chop, where cuts affect only that chop's pads.
Files SHALL open the newest chop.
An unlinked chop SHALL be dropped once a newer chop of that file exists.
The header SHALL identify multiple chops: `cuts of break.wav · chop 2 of 3`.
Chops SHALL be saved with the project.

#### Scenario: Chop the same sample again
- **WHEN** slices 1–8 of a break lie on pads 2-1…2-8, the owner opens the whole-file pad 1-1 with F3, cuts the break in 4 and lays them on pads 3-1…3-4
- **THEN** pads 2-1…2-8 still say `slice 1`…`slice 8` and play what they played, pads 3-1…3-4 say `slice 1`…`slice 4`

#### Scenario: Back to an older pad
- **WHEN** after that the owner presses F3 on pad 2-3
- **THEN** the chop screen shows the 8 slices of the first chop with slice 3 selected, and the header says `chop 1 of 2`

#### Scenario: A cut moves only its own pads
- **WHEN** the owner moves the start of slice 3 in that first chop
- **THEN** pad 2-3 starts later and pads 3-1…3-4 do not change

#### Scenario: Saved and opened
- **WHEN** the project with both chops is saved and opened again
- **THEN** F3 on pad 2-3 opens the first chop and F3 on pad 3-2 the second, each with its cuts

#### Scenario: A project saved before
- **WHEN** a project saved before this change has pads linked to a chop that is no longer in it, and the owner presses F3 on one of them
- **THEN** the chop opens with the cuts at the starts and ends of those pads, and the pads say `slice N` again

### Requirement: Loop the play
`Alt+O` SHALL toggle Chop looping, with its state next to the play mode in help (`loop: on`).
Chop SHALL open with looping off.
With looping on, `Space` SHALL repeat the play-mode span (to the end, to the cut, the slice) from its start until `Space` stops it.
With looping off, `Space` SHALL play once as before.

#### Scenario: The slice over and over
- **WHEN** the play mode is "slice", the loop is on, the cursor stands in slice 3 and the owner presses `Space` and waits three times the length of slice 3
- **THEN** slice 3 plays three times in a row and goes on

#### Scenario: Space stops the loop
- **WHEN** the loop plays and the owner presses `Space`
- **THEN** the sound stops

### Requirement: Loop edits and seams

A cut, slice bound or region edge moved during looping SHALL affect the next turn without stopping playback.
Each turn SHALL fade out and in at its seam without a click.

#### Scenario: Move a bound while it loops
- **WHEN** the loop plays slice 3 and the owner moves the end of slice 3 to the left
- **THEN** the next turn ends at the new end, and the sound does not stop in between

#### Scenario: No click at the seam
- **WHEN** a slice that ends loud loops
- **THEN** the render has no step at the seam larger than the slice's own, no NaN, no clipping

#### Scenario: Off plays once
- **WHEN** the loop is off and the owner presses `Space` in the slice mode
- **THEN** the slice plays once and stops

### Requirement: Play as the pad
The system SHALL switch "as pad" on and off with `Alt+P` on the chop screen; it starts
off. While it is on, what the chop screen plays (`Space`, a cursor move into a slice, a
slice pick) sounds with the volume, pitch, envelope and filter of the pad the slice under
the cursor lies on, the first such pad in pad order; a slice on no pad plays bare. A hit
on that pad does not cut the preview. The help line shows `as pad` next to the play mode
while the switch is on.

#### Scenario: Slice 3 as its pad
- **WHEN** slice 3 lies on pad W with pitch −12 and a low-pass at 500 Hz, "as pad" is on and the owner plays slice 3
- **THEN** the render is an octave lower and has less than a tenth of the energy above 2 kHz that slice 3 has bare

#### Scenario: Off plays bare
- **WHEN** "as pad" is off and the owner plays slice 3
- **THEN** the render is the bare slice, sample for sample

#### Scenario: A slice on no pad
- **WHEN** "as pad" is on and the slice under the cursor lies on no pad
- **THEN** it plays bare

#### Scenario: The switch in the help line
- **WHEN** the owner presses `Alt+P` on the chop screen
- **THEN** the help line says `as pad` next to the play mode; `Alt+P` again takes it away

### Requirement: One rule for the hits

Chop SHALL share one threshold for cursor hits, snapped cuts, hit marks and hit chopping.
Hits SHALL count within that threshold below the region's loudest hit.
The threshold SHALL stay -12 dB until H, Shift+H, a learned hit command or threshold knob sets it, then use that value.
In hits mode, `▾` SHALL mark each counted on-screen hit's start: cursor stops and hit cuts.
Owner `chop.hits` bindings, including former Ctrl+Y, SHALL use this rule.

#### Scenario: The marks are the stops
- **WHEN** hits mode is on and the screen shows four marks
- **THEN** Right from the start of the view stops at the four marks in turn

#### Scenario: Ctrl+Y cuts at the marks
- **WHEN** the owner explicitly maps Ctrl+Y to `chop.hits`, presses it and turns hits mode on
- **THEN** a mark stands at every new cut and nowhere else in the region

#### Scenario: No marks in even steps
- **WHEN** the cursor walks in even steps or cut to cut
- **THEN** no hit marks are drawn

#### Scenario: H cuts at the marks by default
- **WHEN** the default map is loaded, the owner presses H and turns hits mode on
- **THEN** a mark stands at every new cut and nowhere else in the region

### Requirement: Enter starts the order from the selected pad
The system SHALL, in the assign mode "all in order" with no start pad hit yet, take the
selected pad as the start on `Enter` and show the plan; the next `Enter` applies it as any
plan, with the overwrite question when pads are taken. A pad hit before `Enter` sets the
start as before.

#### Scenario: Enter with no pad hit
- **WHEN** pad Q of bank 2 is selected, the owner opens the assign mode, switches to "in
  order" with `Tab` and presses `Enter`
- **THEN** the plan lays slice 1 on pad Q of bank 2 and the rest after it, and the
  summary is shown; a second `Enter` applies it

### Requirement: By hits says what it replaced

The system SHALL, when H or Shift+H chops by hits over existing cuts, name the threshold,
new and previous slice counts, and Ctrl+Z in the status:
`by hits at -12 dB: 7 slices (were 16) · Ctrl+Z brings them back`. With no cuts before,
the line SHALL end after the slice count. A clamped reverse command SHALL report its
threshold without claiming it replaced cuts.

#### Scenario: By hits says what it replaced
- **WHEN** the owner cut a break into 16 slices by hand and presses H
- **THEN** the status says `by hits at -6 dB: N slices (were 16) · Ctrl+Z brings them back`
- **AND** Ctrl+Z brings the 16 slices back

### Requirement: The header shows play

In listen and slice edit, LISTEN SHALL occupy 10 terminal cells including outer spaces: ` LISTEN   ` at rest, ` LISTEN ▶ ` during matching playback.
At rest it SHALL be dim without a blue background.
During matching playback it SHALL show `▶` with the existing blue background.
Following header text SHALL NOT move between states.
The assign badge SHALL keep its text, size and colours.

#### Scenario: The header shows play
- **WHEN** the chop screen is open and the owner presses `Space`
- **THEN** the header says `LISTEN ▶` with the blue background; after the second
  `Space` and the preview stops, it says dim `LISTEN` if no matching pad still sounds

#### Scenario: Chop listen badge has fixed width
- **WHEN** Chop changes between silence and playback
- **THEN** the LISTEN badge occupies the same 10 terminal cells and the file name and
  remaining header text stay in the same columns

### Requirement: Chop listen badge follows matching playback

Chop preview or a linked pad voice of the open chop SHALL light LISTEN, including positions outside the waveform view.
Pads of another sample or chop, old-project voices and effect tails alone SHALL NOT light it.
This indicator SHALL NOT change sound, transport or edit commands.

#### Scenario: Chop listen badge lights only while playback runs
- **WHEN** a Chop preview or linked slice pad starts and then ends or is released
- **THEN** LISTEN lights blue with `▶` while it plays and returns to dim without `▶`
  when neither the preview nor a linked pad of this chop still sounds
- **AND** keyboard pads, raw notes and learned pad buttons give the same result

#### Scenario: Chop listen badge stays lit beyond the waveform view
- **WHEN** a linked pad still plays but its position is outside the zoomed waveform view
- **THEN** the green line is hidden outside the view but LISTEN stays blue with `▶`

### Requirement: Named assign order action

On Chop, the system SHALL bind Tab to `chop.assign_order`. It SHALL switch by-hand and
in-order assignment only while an assign plan is open, with the same plan and undo
behavior as the former `edit.param_next`. It SHALL do nothing in listen or slice edit
and SHALL NOT apply a plan. Existing owner keymaps and learned controls for
`edit.param_next` and `chop.auto_mode_next` SHALL keep switching the order while
assigning. Threshold commands SHALL stay ignored while assigning.

#### Scenario: Tab has the named assign order action
- **WHEN** a by-hand plan is open and the owner presses Tab
- **THEN** the plan switches to in-order with no target until one is selected
- **AND** the next Tab returns to an empty by-hand plan without applying either plan

#### Scenario: Old parameter binding still switches assign order
- **WHEN** a learned controller button or owner key sends `edit.param_next` while assigning
- **THEN** it switches by-hand and in-order as Tab does

#### Scenario: Old auto mode binding still switches assign order
- **WHEN** an owner binding sends `chop.auto_mode_next` while assigning
- **THEN** it keeps its former order-switching behavior and does not apply the plan

#### Scenario: Assign order does not edit a listening slice
- **WHEN** `chop.assign_order` is sent in listen or slice edit
- **THEN** cuts, selection, bounds and playback stay unchanged and no plan opens

#### Scenario: New hit keys stay deaf while assigning
- **WHEN** an assign plan is open and the owner presses H or Shift+H
- **THEN** cuts, threshold and plan stay unchanged and the status names apply and back

### Requirement: Pad playback line on Chop

In listen or slice edit, a pad playing a slice of the open chop SHALL show a thin green line at its actual sample position.
Keyboard pads, raw MIDI notes and learned pad buttons SHALL give the same display.
The line SHALL follow real playback, including pitch and retrigger, and disappear when the voice ends.
It SHALL use the current wave view in Braille and picture modes.

#### Scenario: Keyboard pad shows the Chop playback line
- **WHEN** a linked slice is open on Chop in listen or slice edit, the owner hits its
  keyboard pad and the real audio callback advances that pad voice
- **THEN** the green line follows that voice's frame position in the waveform
- **AND** the cursor keeps the existing hit selection but does not follow playback

#### Scenario: MIDI pad shows the same Chop playback line
- **WHEN** the same linked slice is hit from a raw MIDI note or explicitly learned pad
  button through the real dispatch and audio queues
- **THEN** the green line follows the same sounding position as the keyboard path
- **AND** no extra pad hit or preview is started to supply that position

#### Scenario: Chop pad line follows pitch and retrigger
- **WHEN** a linked slice pad plays with changed pitch or is hit again on Chop
- **THEN** the green line follows the voice's real sample position and returns to
  that slice's start on the new hit, not elapsed UI time or a bare preview's speed

#### Scenario: Chop pad line ends with its voice
- **WHEN** a displayed pad voice reaches its end, finishes its gate release or is stopped
- **THEN** that voice's green line disappears without starting or stopping another voice

### Requirement: Chop pad line is display only

The pad line SHALL change only playback position display.
It SHALL NOT add hits or previews, change sound, move the cursor during playback or change hit-driven slice selection.
Space, M, transport, editing, preview and assignment SHALL keep their existing behavior.
The displayed pad position SHALL NOT replace the preview position used by those commands.

#### Scenario: Chop pad line keeps transport and edits unchanged
- **WHEN** a linked pad sounds on Chop, its new green line is visible and the owner
  moves the cursor, presses M or uses Space
- **THEN** the cursor and commands keep their existing behavior, independent of the
  new pad position display
- **AND** Space preview, bare slice previews and assignment previews keep their existing
  playback line behavior

### Requirement: Chop pad line rejects unrelated input

The pad line SHALL NOT show another sample's position or retain an old project's position.
Silent, blocked or queue-rejected hits SHALL NOT create a new playback line.

#### Scenario: Chop pad line rejects unrelated and silent hits
- **WHEN** a pad plays another sample, the project changes, a question or learn blocks
  a new hit, or the audio queue rejects a new hit
- **THEN** no line is created from that unrelated, old-project, blocked or rejected hit
- **AND** an already sounding matching voice keeps its valid position until it ends
