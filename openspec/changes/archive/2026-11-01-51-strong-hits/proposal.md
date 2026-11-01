# 51 — Hits mode stops at strong hits, and the hits are marked

## Why

The owner (2026-10-31), a screenshot of the chop screen in hits mode: "the cursor works
very oddly in mode N (snap to peaks); visually these are not peaks". Measured on the
owner's recordings: 4.7–7.6 hits a second, up to 11 of 37 more than 12 dB below the
loudest; any rise by half within 10 ms counted, with no floor, and nothing showed where
the cursor would stop. Picked "Q1.1"; then: "the auto-chop too, agree?".

## What changes

- One hit threshold for the cursor's hits mode, the snap of a cut, the marks and `Ctrl+Y`:
  −12 dB below the loudest hit of the region until `Ctrl+Y` or the knob sets it.
- In hits mode the counted hits on the screen carry a `▾` over the wave.
- The cursor still lands on the start of the attack, where a cut keeps the attack whole.
