## Why

Aggregate callback times cannot identify the CPU, resource events or sample time of a particular slow block.
The owner requested automatic records for future overruns, without changes to sound processing.

## What Changes

- Record each measured block whose wall or thread CPU time is strictly above 50% of the budget.
- Include its phase, index, time offset, paired durations, boundary CPU IDs and thread resource-counter changes.
- Observe available CPU frequency after an overrun. Record its source and read interval, not an assumed frequency inside the callback.
- Save local CSV files with unique names. Keep records when a p99 exit gate fails.
- Report observation overhead. Keep the original workload, raw times and exit gates.
- Keep raw CSV files, detailed machine logs and identifying local paths outside Git. Permit anonymized verification summaries.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `audio-engine`: per-block offline benchmark records and their limitations.

## Impact

Only benchmark support, its call sites, tests and generic documentation change.
No new dependencies, sound-device access, system settings, live logging or saved formats.
This report does not fix performance or close the callback maximum-budget gate.
The owner approved one PR for 84b and the completed save-safety changes 86b–86d. Change 87 stays inactive.

## How to verify

1. Run `task bench` without an audio device or controller.
2. Read the printed CSV path and the observation-overhead report.
3. Verify that the CSV contains its header even when no block exceeds the target.
4. Use deterministic tests for rare overruns, missing counters, output failures and a failing exit gate.
5. Compare traced callback output with direct callback output in an automated test.

Keyboard and controller tests do not apply to this command-only change.
Sound processing stays unchanged. This change needs no new listening render.
Do not publish raw CSV files, detailed machine logs or identifying local paths.
The owner permits anonymized verification summaries in documents and commit history.
