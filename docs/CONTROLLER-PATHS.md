# SMC-PAD paths for current stories

This table covers the headings in [User stories](USER-STORIES.md) for change 93. It describes the shipped SMC-PAD profile and panel map. Raw MIDI tests check software routes. The owner has not yet checked these routes on the physical device.

## Count a command

Each pair is **visible / hidden**: button presses from the visible main panel, then from a hidden panel. A main cell is `1/3`: the hidden route adds PAD BANK and one reveal hit. A context cell is `2/4`: More and the action cell, with the same two entry presses when hidden. E6 turns select a context page without a button press. A first-bank knob is `0/0`. PLAY, STOP and RECORD are `1/1`. A modal P7 or P8 answer is `1/2`: it needs PAD BANK from the first bank, but no reveal hit. The count excludes knob turns, target pad hits and later modal answers. A first-bank pad hit is a separate sound or target action. An owner binding can replace a shipped cell.

| User story | SMC-PAD route and command counts | Exception or limit |
| --- | --- | --- |
| Terms | Read the screen and pad labels (`0/0`). | Observation. |
| Two hands | First-bank pads play. E8 selects banks (`0/0`). Help is main P7 (`1/3`). | Pad hits are separate. |
| SMC-PAD command panel | PAD BANK, reveal, main cell (`1/3`). More and context cell (`2/4`). E6 selects later pages (`0/0`). | Local PAD BANK has no host MIDI message. |
| Chop the sound of a pad | First-bank pad selects. Main Chop (`1/3`). E1 cursor (`0/0`). Main Cut, Parts, Hits or Assign (`1/3`). Context Commit (`2/4`). PLAY preview (`1/1`). | Target pads and a replacement answer are separate. |
| Edit a pad | First-bank pad selects. Main Edit (`1/3`). Pad-edit main page and parameter cells (`1/3`). Context value cells (`2/4`). Back (`1/3`). | First-bank hits remain playable. |
| Read modes and operation results | Main Copy (`1/3`). Fixed Back (`1/3`). Main Help (`1/3`). Help P7/P8 tabs and P9/P10 move (`1/1` while open). | Result levels, status, and badges are observations (`0/0`). |
| Read pad playback | First-bank hit starts a voice. Watch PLAY (`0/0`). | Playback and release use the pad, not a command cell. |
| Your own keys | Edit `~/.config/ptsr/keymap.toml` with typed text. | Keyboard-only document edit. |
| The splash screen | Open a screen from a main cell (`1/3`). Play a first-bank pad. | Splash text is an observation (`0/0`). Silent input does not end it. |
| Play a beat | First-bank pads play. Transport toggle is context page 2 P2 (`2/4`). | A Gate release belongs to its original pad. |
| Name a pad | Open and edit its name with `Ctrl+N` and typed text. | Keyboard-only text entry. |
| Copy a pad without a growing name | Main Copy (`1/3`). First-bank target pad. Context Clear (`2/4`). Fixed Undo (`1/3`). Context Save (`2/4`). | The generated name is an observation. |
| A project for a track | Context Project (`2/4`). E1 selects (`0/0`). P8 opens (`1/2`). Files main Favorite (`1/3`). | A new project name needs keyboard text. |
| Load a sound onto a pad | Main Files (`1/3`). E1 or LEFT/RIGHT browses (`0/0` or `1/1`). Main Take or Load (`1/3`). Context preview toggle (`2/4`). | File filtering needs keyboard text. Target pad is separate. |
| Set up a pad | Main Edit (`1/3`). Page and parameter main cells (`1/3`). Context value cells (`2/4`). Context Mode, Mark bank, Clear or Gate (`2/4`). E8 selects bank (`0/0`). | A question takes P7/P8 separately. |
| Mute groups: the closed hat stops the open hat | Main Edit (`1/3`). Page and parameter cells (`1/3`). Context value cells (`2/4`). Fixed Undo (`1/3`). | Pad hits and audible result are separate. |
| The pad panel in a small window | Page and parameter main cells (`1/3`). | Folding, labels and arrows are observations (`0/0`). |
| Effects: drive, reverb, delay | Main FX (`1/3`). FX page and parameter main cells (`1/3`). Context value cells (`2/4`). Fixed Undo (`1/3`). | Listen to existing renders. No new sound route. |
| Undo a change to the pads | Fixed Undo (`1/3`). | A no-op Undo shows its existing message. |
| What is set on a pad | Main Pads (`1/3`). First-bank pad selects. Context Mode (`2/4`). | Mode line, mark and changed-value brightness are observations (`0/0`). |
| Waves on the pads | Main Pads (`1/3`). Page and value cells as in Set up a pad. | Wave drawing is an observation (`0/0`). |
| One look for the screens | Main Pads, Files or Chop (`1/3` each). | Layout is an observation (`0/0`). |
| Hide the pads | Pads context Grid (`2/4`). Other views use page 2 Grid (`2/4`). | Hidden grid state remains visible in the hint. |
| Set up a pad with the X-Session Pro | Use the X-Session profile and its own physical controls. | Other-device story. The SMC-PAD does not claim its hardware route. |
| Chop a break | Main Chop (`1/3`). E1 cursor, E2 zoom, E3 threshold, E4/E5 bounds (`0/0`). Main Cut, Parts, Hits, Assign (`1/3`). Page 4 Hits back, Delete cut, Delete end, Order (`2/4`). | PLAY is `1/1`. Target pads and answers are separate. |
| Keyboard and controller pads on Chop | First-bank pad selects or assigns. E8 selects bank (`0/0`). | A pad hit and its release are separate from command counts. |
| Select what plays | Chop context Play mode, Loop or As pad (`2/4` each). PLAY and STOP preview (`1/1`). | The playback bar is an observation. |
| Move the bounds of a slice | E4 moves the start and E5 moves the end (`0/0`). | A knob step follows the current wave view. |
| Bounds in edit mode | Chop context Edit (`2/4`). Edit main parameter cells (`1/3`). Context value cells (`2/4`). Back (`1/3`). | The amber bound is an observation. |
| Assign the slices to the pads | Chop main Assign (`1/3`). First-bank target pads. Page 4 Order (`2/4`). Chop context Commit (`2/4`). Back (`1/3`). | P7/P8 answer replacement separately. |
| Assign mode ignores the rest | Use the Assign cell (`1/3`). Try another cell (`1/3`). | The refusal and unchanged plan are observations (`0/0`). |
| Swap pads | Pads main Take (`1/3`) or page 2 Take on any view (`2/4`). First-bank target pad. Back (`1/3`). | The target hit is separate. |
| Select file, copy and swap targets without a hit sound | Files main Take (`1/3`) or Pads main Copy/Take (`1/3`). First-bank target pad. P7/P8 answer (`1/2`). | Target hits are separate and silent in these modes. |
| Controller commands and the next pad hit | Use Take (`1/3`), bank E8 (`0/0`), then a first-bank target pad. | Watch the input order and held release (`0/0`). |
| MIDI knobs | Main Settings (`1/3`). Main Tab and Learn (`1/3`). E1 selects a row and E2 changes a value (`0/0`). P8 accepts a pending choice (`1/2`). | A device binding can replace the shipped map. |
| Bind the controller directly on the screen | Use its action cell. Page 2 Learn (`2/4`). Touch a control. P8 saves or P7 cancels a pending choice (`1/2`). | Back leaves learning. Typed action names are not needed. |
| Record sound | Hold RECORD for microphone (`1/1`). Page 2 Outrec toggles output capture (`2/4`). | Recorded file placement uses a first-bank target pad. |
| Session | Context Save (`2/4`). Page 2 Exit (`2/4`), then P8 to leave or P7/Back to stay (`1/2`). | Automatic save is an observation (`0/0`). |

## Open owner checks

The software tests do not establish physical E6, E7 or E8 movement, physical PAD BANK order, terminal composition or listening results. Use [Live checks](LIVE-CHECKS.md) for those passes. This change has no new render. `renders/92-command-panel.wav` is the existing transport reference.
