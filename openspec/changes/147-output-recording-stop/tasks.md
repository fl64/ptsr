## 1. Output recording STOP

- [x] Route STOP during output recording on Files, Chop and Assign.
- [x] Keep a pending Files preview from starting after STOP.
- [x] Show the preview step in the recording hint.
- [x] Test the saved take through keyboard and recorded SMC-PAD input.

## Verification

- Files, Chop and Assign keep the paused position and the tail in the WAV.
- A late Files load cannot start the preview after STOP.
- Keyboard and raw SMC-PAD input pass the three STOP scenarios.
