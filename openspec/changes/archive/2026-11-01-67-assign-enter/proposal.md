# 67 — Enter lays the slices in order from the selected pad, with no pad hit

## Why

A UX check of the owner's agent (2026-11-01): in the assign mode "in order", `Enter` does
nothing until a pad is hit — the hit sets the start. `P`, `Tab`, `Enter` and nothing
happens: a dead end where it should be smoothest. The controller already does better: its
commit with no plan lays the slices in order from the selected pad (21). The owner: "yes".

## What changes

- In the assign mode "in order" with no start pad yet, `Enter` takes the selected pad as
  the start and shows the plan; the next `Enter` applies it, as now.
- A pad hit still sets the start, as now.
