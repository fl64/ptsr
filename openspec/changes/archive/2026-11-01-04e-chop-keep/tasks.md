# Tasks — 04e

- [x] 1. Cuts are saved without pads: any change of the cuts marks the session dirty; test "Chopped, nothing laid out"; commit.
- [x] 2. Enter opens the assign mode with the selected slice; test "Assign the slice just heard"; commit.
- [x] 3. A cut inside a slice on pads unlinks these pads, the status names them, undo links them again; unlinked pads are the `pad` source of 27; tests "Cut inside a slice on a pad", "Undo the cut"; render `renders/04e-cut-inside.wav` (the pad sounds the same before and after the cut); commit.
- [x] 4. The info line names the pads of the selected slice; test "Where a slice lies"; commit.
