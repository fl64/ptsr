# Proposal

## Why

The owner recorded hits with quantize 1/4, then set quantize 1/16 to place a new hit between beats. All earlier hits moved too, also the hits that were already in the correct place. Today quantize is a playback setting of the whole A1, and A1 keeps the played time of each hit.

The owner chose the behavior of a hardware sampler such as an MPC or SP-404: quantize applies when the sampler records a hit.

## What Changes

- A captured hit gets the quantize and strength of the moment of its capture. A1 saves the quantized tick.
- A change of Quantize or Strength applies only to the next hits. Saved hits stay in place.
- Playback plays the saved ticks. It does not quantize again.
- One pad in one quantized cell keeps one hit. The last captured hit replaces the earlier one, in one pass and across passes. A flam needs a finer grid or quantize Off.
- The program does not convert old projects: a saved tick plays as it is. The owner's projects in `~/ptsr/projects` get a one-time conversion by a script, with a backup, so they sound as before.
- Pattern controls and F1 say that Quantize and Strength set the next hits.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `patterns`: quantize on capture, playback of saved ticks and the replacement rule.
- `screen-help`: the meaning of Quantize and Strength in pattern controls and F1.

## Impact

App capture and pattern compilation, pattern controls text, help.
The saved project format stays the same. An old A1 from another place plays its played time.
No dependency change. The audio callback work does not grow: it plays fewer quantize steps.

## Out of scope

- A command that quantizes all of A1 again. A later change can add it with a question.
- The step editor (draft [98a](../../../docs/plans/98a-step-edit.md)).

## How to check

1. Set Quantize 1/4. Record a kick on beats 1 and 3. End the pass.
2. Set Quantize 1/16. Check that the kicks stay on beats 1 and 3.
3. Record a snare between beats. Check that it lands on the nearest 1/16 cell.
4. Hit the same pad twice in one 1/16 cell in one pass. Check that one hit stays.
5. Open one of your old projects with A1. Check that it sounds as before and that `project.toml.before-timing-*` is next to `project.toml`.
6. Listen to `renders/98c-quantize-on-input.wav`.
