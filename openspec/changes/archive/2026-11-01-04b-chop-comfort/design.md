# Design 04b

Screen sketches: `docs/ui/chop.md` (cutting), `docs/ui/pads.md` (envelope and the big view). Builds on the slice model of 04a, the linked chops of 04c and the pad parameters of 19; done after 04d (owner decision, 2026-10-28).

## Pad envelope (decided with the owner, 2026-10-30)

- Sound settings live on the pad, not on the slice: 19 put volume, pitch and the scope there; 04b adds attack, decay and release on the same model. The same slice on two pads can fade differently, and what a pad shows is what it plays.
- Attack: fade-in over the first `attack` ms of the piece. Decay: the last `decay` ms of the piece fade to silence, the piece still ends at the same place. Release: in gate mode the voice fades over `release` ms after the release; a one-shot pad plays to the end. Attack or decay 0 means off; release 0 means the current short click-free cut.
- On the pads screen `Tab` steps through volume → pitch → attack → decay → release; `−` `=` change the picked parameter for the change scope (the selected pad or the marked pads, a bank, all pads); the pop-up shows the value, the limit pop-up counts the pads that clamped. The envelope follows the scope like volume and pitch.
- A full ADSR only on request. Reverse is out of 04b; a separate small change can add it.
- The big view of the selected pad's piece lives on the pads screen, the envelope drawn over the piece shape.

## Engine

The envelope lives in the voice next to the pitch glide: attack and decay follow the read position of the piece, release runs after the gate off. All smoothing is per voice, no allocation in the audio callback; one engine command per pad carries all three times, so a scope change stays one burst of commands like in 19.

## Session

The three envelope times extend the pad entries of the session like `pitch` did; a missing field means the default (no attack, no decay, the engine's short release), so no new session version and old sessions open unchanged.

## Kitty graphics (task 4, defaults set 2026-10-30, the owner looks live)

- Detection at start, before the event loop: the graphics query
  `ESC _G i=31,s=1,v=1,a=q,t=d,f=24;AAAA ESC \` followed by the device attributes query
  `ESC [c`; a graphics reply before the attributes reply means support. Timeout 200 ms.
  Inside tmux (`$TMUX` set) and when the terminal does not report its size in pixels
  (`crossterm::terminal::window_size`), graphics stay off. The log says which.
- What is drawn as a picture: only the wave itself, in the area where the Braille wave is
  drawn today (the chop screen wave, the big view of the pad). Cut lines, the cursor, the
  playhead, the envelope line and every text stay terminal characters drawn over it: the
  picture sits under the text (`z=-1`), the wave cells are left blank in graphics mode.
- Colours of the picture: the same as the Braille wave of that area.
- Transmission: RGBA (`f=32`), direct (`t=d`), base64 in chunks of at most 4096 bytes, a
  fixed image id per area (chop wave 1, big view 2), `q=2` so the terminal sends no
  replies, `C=1` so the cursor does not move. A picture is sent again only when what it
  shows changes (the peaks, the size, the colours); it is deleted (`a=d,d=i`) when its
  area is not on the screen.
- No new dependency: base64 is a few lines.
- Double buffer (owner, 2026-10-30: "the whole picture blinks, it is annoying"): every
  area has two image ids; a changed picture is transmitted under the spare id and placed,
  then the old id is deleted. Deleting first left the screen empty while about a megabyte
  of base64 went out.
- Tests: graphics off draws exactly the Braille of today; the escape encoder (chunks,
  keys, ids) and the rasteriser (peaks to pixel columns) have unit tests. How it looks is
  the owner's check.

## Superseded

- The big view of the selected pad under the panel (task 3) is taken out by 29
  (owner, 2026-10-30: the same piece was drawn twice, the narrow tall area made it a
  spindle); every pad draws its own wave instead. Its data stays: the strip above the
  grid draws the pad's envelope in the edit (21).

## Open questions

- Curve of attack and decay: linear first; a choice of curve is left out until asked. Default: no curve choice.
- Steps and limits: `−` `=` step 5 ms, limits 0..5000 ms. Default; the owner may ask for finer steps later.
- Kitty graphics: see the section below. Braille is the default; the picture is a switch
  on F12, `[ui] graphics = true` in the config, from the next start (owner, 2026-10-30,
  after a live try: "the whole picture blinks", "it started to lag" — "make Braille the
  default, I switch the picture on in the settings if I want"). A full-resolution picture
  is ~1.7 MB of base64 per send; holding a key sends it on every step of the view.
