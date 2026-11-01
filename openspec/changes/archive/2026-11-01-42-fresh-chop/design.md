# Design 42

## Decisions (owner, 2026-10-31)

- 2 of: ask / always fresh / keep. "Always fresh": a whole-file pad means a new chop.
- The fresh chop is a new epoch with one undo step, the way equal parts and hits already
  replace the cuts: linked pads let go, undo links them back. No new mechanism.
- The source pad `◆ all` is a whole-file pad too: F3 on it starts fresh.
- As samplers do it (owner: "2, do it as the classic samplers do"): a chop is a job on the file, the
  laid-out pads are its result. After a fresh chop the old pads are on their own (`pad`);
  their pieces are tuned on the SOURCE page; only an undo right away brings the old cuts
  back. Several living chops per file (each pad leading to its own chop) were weighed
  and left for later: the epochs would carry them, the project format would change.
- The files screen (F3 on a file) continues the file's current chop: a chop with no pads
  must not be lost by opening the file again.

## Open questions

None.
