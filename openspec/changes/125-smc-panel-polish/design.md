# Design

## Review order

Change 124 supplies the fixed knob roles. The owner will try both changes in one
run. Base this branch on the review branch for change 124. Use the main checkout.

## Panel

Place Pads, Files, Chop and A1 on row 1. Place FX, Mixer, Setup and More on
row 2. Mixer stays dim until its screen exists. Put Help on More, cell 10, in
each view. Keep Back, Undo, Redo and Panic at the bottom of the command pages.
Keep Panic dim until the double press from 93a is available.
Use `knobs`, `record`, `open`, `grid -` and `grid +`. Each label fits six cells.

Start with Open panel for a supported USB controller after a valid status check.
Keep a saved Off choice. A press that the status check misses still permits
the first-hit reveal. The shortcut has no action on another controller.

## Target choice

The current route needs a return to the playing layer to select a target.
An Assign plan then needs another layer change to reach Commit. A hint can
explain those layer changes. Direct target cells remove them for pads 1–12.
The owner will test the full layout together. Use direct targets for the
review build. Record the prior and new software routes in the route table.
Record actual button counts on the device during the owner check.

A target view uses cells 1–12 for the matching pad positions of the current
bank. P13 cancels. P14 applies an Assign plan. A Copy, Swap or file target
uses the existing placement action on the target hit. Targets 13–16 still
use the playing layer. The first layer 1 hit closes the panel as before.
On return, the target view makes Apply available when Assign is still open.
P7 and P8 answer questions on the command layer.

## Device warnings

Warn once per Master connection on an unassigned, out-of-range pad press on
the performance channel. Never infer a new map from the shifted note.
A verified wrong USB preset reports its number and disables knob actions.
A failed status read retains the existing reveal fallback.
F1 and the setup guide explain the device Note Repeat clock.

## Edge cases

| Situation | Expected result | Status |
| --- | --- | --- |
| A question opens after a target hit. | The next P7 or P8 answers the question once. | Check with recorded MIDI. |
| The owner hits the playing layer during a visible target view. | The hit uses the current target route. The panel closes. | Preserve the existing route. |
| The owner releases a held Gate while choosing a target. | The release reaches the original voice. | Preserve the held-pad route. |
| A second Master sends a target cell. | It cannot choose a target on the first Master's panel. | Check panel ownership. |
| The status reply arrives after a later playing pad hit. | The reply cannot reopen the panel. | Preserve the existing order check. |
| A saved setup has Off. | The application keeps Off after restart. | Check config load and save. |
| The unit reports a wrong preset during polling. | Disable knob actions and keep the slot number in the notice. | Check a recorded USB response. |
| A shifted note still overlaps the shipped ranges. | The program cannot infer a shift from that note. | Document this limit. |
