# 44 — A filter on every pad

## Why

The owner (2026-10-31): "let us add a filter on a pad". Picked "Q1.1": as an MPC does it,
a filter of its own on every pad (assumption: from memory, the MPC has a per-pad filter
with type, cutoff and resonance; the SP-404 has a filter as an effect on a bus).

## What changes

- The pad edit gets a FILTER page: filter type (off / low-pass / high-pass / band),
  cutoff and resonance, on knobs 1–3 and the edit keys, as the other pages.
- The filter is saved with the pad in the project.
- On the FILTER page the strip above the pads shows the filter curve in place of the wave.

## Not in this change

A filter on the whole output or a bank (as the SP-404 does it).

## Dependencies

None: the filter is a few lines of a state variable filter in the engine.
