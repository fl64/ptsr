# Design

## User contract

F5 opens a full-height A1 table. Time runs across columns. Eight bank sections form one vertical list. Each shown bank has 16 pad rows, even when some pads have no hits. The owner can compare rhythms across banks by moving through the list. The owner can view the table but cannot edit its hits. Existing pad and A1 commands keep their roles outside the table cells. Pad hits keep their ordinary sound and selection behavior.

This change comes before targeted removal. The deferred 98a draft remains a guide for a later editing change. On 2026-11-12, the owner accepted the 80x26 preview with 16 pad rows in the current bank and a read-only first step.

## Screen and input

The overview is a separate screen with the `PATTERN · A1` identity. Other screens name the F5 destination `pattern`. F5 opens the overview from an ordinary screen. F2 and Esc return to Pads. Other screen keys retain their existing routes. F1 opens full help and returns to the overview. A text line, picker, question or learning mode retains its input priority.

The existing U/O and PageUp/PageDown bank controls select one of eight banks and move the viewport to its section. Bank changes never move or restart A1. The fixed bank index names all eight banks and counts pads with visible A1 hits. Alt+H switches empty bank sections. The current bank stays visible when it has no visible hits. A bank with only hidden raw hits counts as empty. Pad keys play and select pads as on Pads. Shift gives full velocity. Releases keep the original sound owner across bank and screen changes. Learned `screen.pattern` and `pattern.overview_empty_toggle` buttons use the same actions as F5 and Alt+H. The existing learned view buttons use the same view state on Pads and the overview.

## Projection and pages

The App derives visible rows from the current project's checked raw pattern and the existing compiler descriptors. It filters hidden raw hits before quantize and wraps visible seam hits only after that filter. The projection keeps separate source events in one cell. It calculates the visible time page and bank sections, not a dense table for all 128 pads and the full loop. It uses the same symbols as the Pads strip: `·` empty, `■` on grid, `~` off grid, `2` to `9` or `+` for multiple events.

The table shares the runtime view division with the Pads strip. It supports 1/4, 1/8, 1/8T, 1/16, 1/16T, 1/32, 1/32T and 1/64. Changing the division does not edit the pattern, quantize, history or dirty state. The table chooses the largest set of complete bars that fits with a gap between adjacent step numbers. If a bar does not fit, it shows complete half bars or beats. It never splits a beat. A running A1 selects the page that contains the playhead. A stopped A1 shows the first page.

The playhead plate comes from one coherent current-project audio snapshot. It spans one time column across the shown bank sections and differs from event symbols by its background and position. Missing or old-project snapshots show no moving plate. The table shows `No A1` and the effective record command when the pattern is absent. An existing empty A1 shows the current bank with 16 empty rows.

## Layout and feedback

At 80x26, the table keeps the title, fixed bank index, 16 rows of the selected bank, message, status and three-row help visible. Sections continue below the viewport. Each section uses the same soft bank color as the pad screen for its outline. Its bank number and active-pad count remain legible without color. The current section has a distinct outline. Row labels show the pad key and use the available width for the sample name. At 120x40, each name has at least 40 terminal cells. When the selected name still clips, the title shows it in full if the title fits. The grid cells stretch to use the remaining width. The view header shows division, bar span, quantize and playback position. Beat shading and bar lines match the Pads strip. Color is not the only way to identify a hit or the playhead.

The overview shows all eight bank sections by default. Alt+H hides or shows empty sections without editing A1. The fixed index labels its numbers as pads with A1 hits. It always keeps all bank numbers. It keeps the shared status, safety notices and F1 help. It has no pad-mode badge because pad keys still sound. The offline catalogue includes absent, empty, straight, triplet, off-grid, multiple-event, hidden-hit and safety-warning examples at both supported sizes.

## Verification boundary

Named tests cover every scenario. Keyboard and raw MIDI through the real profile cover screen entry, bank changes, view commands and pad ownership. Projection tests cover event cells, pages, empty rows and stale snapshots. Screen catalogue tests cover both sizes. Sound does not change, so this change uses the existing pattern render and adds no WAV.

## Open questions

- Should an empty pad keep its row? Default: yes, so every pad has a stable position.
- Should F5 open a separate screen or replace the Pads strip? Default: a separate screen. Sixteen rows do not fit the three-row strip at 80x26.
- Should the overview show all eight banks in one vertical list? Default: yes. Keep a fixed bank index and scroll the list.
- Should the overview hide banks with no visible A1 hits at entry? Default: no. Show all bank sections at entry and add a show/hide control.

## Owner review on 2026-11-12

The owner tried the wide terminal view. The grid used a narrow part of the screen. Pad names lost text while the right side stayed empty. The bank marks and the `F5 A1` help label did not explain the view. The owner proposed a scrollable list of banks, a switch for empty banks, and a colored outline for each bank. The next layout must use the available width for names and time cells. It must name the F5 destination `pattern` in short help. The scroll and empty-bank defaults remain open above.

## References

- [Current 98a editor draft](../../../docs/plans/98a-step-edit.md)
- [Archived 98 design](../archive/2026-11-12-98-pattern-screen/design.md)
