# 54 — Tab and two controller buttons turn the edit page

## Why

The owner (2026-11-01): "now, when I open the pad edit, I have to press down to get to the
next block. Switch the modes by a key and by a MIDI button, then every mode uses the same
knobs". The knobs already are page slots (21, 48), but the page changes only as the pick
walks parameter by parameter: from TONE to FILTER takes five presses.

## What changes

- `Tab` turns the page of the edit forward, `Shift+Tab` back, in a ring. The pick lands on
  the first parameter of the new page. `I` `K` (`↑` `↓`) still walk one parameter.
- Two new actions, `edit.page_next` and `edit.page_prev`, for the keymap, the device
  profiles and the learn mode.
- A page knob, `edit.page`: its travel is split into equal parts, one per page; it turns
  the page at once, no pickup. Not bound by default, bound in F12.
- The X-Session Pro profile: b3 and b4 in the edit layer turn the page, not the parameter
  (the parameter turns on its own knob, a pick button gives nothing there).
- An object with one page (the slice edit, BOUNDS): the page keys do nothing.

## Not in this change

- Delay and reverb: a separate change (effects on two buses, a SEND page on the pad).
- Showing what is set on a pad without the edit: a separate change, the owner picks the look.
