# Tasks — 06c

- [x] 1. Bank fill glyphs in the bank row (two braille glyphs per bank, lit dot = pad with a sound, on every screen with the row, live updates); render check that row width stays inside 200 and 120 columns; commit.
- [x] 2. First-steps hint is the fallback content of the status line: it returns after fading messages; gone once the first pad holds a sample; logic test on recorded message sequences; commit.
- [x] 3. Audio-unavailable: one plain message, corner indicator carries the state, raw error to the log only; logic test for message wording and no status-line residency; commit.
- [x] 4. Files screen: title shows `~` for the home folder; preview strip three rows tall with the length label kept; logic test for the short path; commit.
- [x] 5. Files screen names both paths from a selected file (Enter to a pad, F3 to chop); commit.
- [x] 6. Same-sound replace asks nothing (whole file on the pad, same whole file again); logic test; commit.
- [x] 7. Chop screen texts: "1 slice" in plain English; slice line "start … · length …"; logic test; commit.
- [x] 8. F12 Audio: honest message when backend is jack and the server is down; commit.
- [x] 9. Help line: the screens group lists F1, F3, F6, F12 in ascending key order; commit.
