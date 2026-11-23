## 1. Settings input

- [x] Use the same tab transition for Tab and SMC-PAD LEFT and RIGHT.
- [x] Limit the row and value controls to their tabs.
- [x] Check both input devices with the shipped bindings and a temporary config.
- [x] Show encoder roles for the current tab and folder picker.
- [x] Keep the folder list and controls visible when the panel is active.
- [x] Repeat the walk and checks for the folder picker.

## Verification

- Input row and value controls passed with keyboard and raw SMC-PAD events.
- Paths passed with both roots and both SMC-PAD tab directions.
- Folder picker navigation and cancel passed.
- The encoder roles passed with raw SMC-PAD input. The screen catalogue passed at both sizes.
- The picker keeps the selected folder and its controls visible in a long list.
- Keyboard and virtual MIDI can cancel or save at 80x26 and 120x40.
