# Tasks — 44

- [x] 1. The engine: a filter per voice (state variable, no allocation in the callback),
  type, cutoff and resonance per pad with a glide; offline renders `renders/44-*.wav`;
  tests "Behind a wall", "The rumble goes", "Off is untouched", "Turn while it sounds";
  the callback benchmark at 32 voices with the filter on stays under 50 % of the buffer;
  commit.
- [x] 2. The pads: the FILTER page, the keys and knob steps, the change scope; the
  session fields (absent reads off); tests "The filter page", "Marked pads together",
  "Saved and opened", "An older project", "The frame tells the mode", "The title shows
  where Tab leads"; commit.
- [x] 3. The filter curve on the strip; tests "The cutoff moves on the curve", "The
  resonance is a hump", "Off is flat", "Back to the wave"; commit.
- [x] 4. `docs/USER-STORIES.md`: the filter; commit.
- [x] 5. Live check with the owner (2026-10-31: "all seems ok"): the filter by ear on a hat, a kick, a break; the curve by eye.
