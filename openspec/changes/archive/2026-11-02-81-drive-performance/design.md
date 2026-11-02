## Context

At fixed drive the engine repeats the same normalization calculation for each channel
and frame. Drive changes glide over each audio block. The audio callback cannot allocate,
wait, perform I/O or free sample memory.

## Goals / Non-Goals

Reduce CPU cost while preserving the existing curve and the current drive glide. Do not
change the character, control range, file format or scheduler settings.

## Decisions

Measure the original code with all 32 voices sustained. Compute normalization once per
frame and share it between the two channels; when the drive is constant, compute it once
per block. Keep the same per-frame formula while the drive changes. Compare to the old
formula over representative inputs and a drive ramp. Render both versions for the owner.
A lookup approximation is not needed unless measurement proves this insufficient.

## Risks / Trade-offs

A different operation order can change rounding. Bound the output error in the spec and
check finite samples, clipping and smooth transitions. Measure on the same core to make
comparisons meaningful; wall-clock peaks can also include scheduler delays.

## Open Questions

None.
