# Design 44

## Decisions (owner, 2026-10-31)

- The filter: 1 of 3 ("Q1.1"), per pad, a FILTER page with three parameters in knob
  order: type (off / low-pass / high-pass / band), cutoff, resonance.

```
  EDIT · FILTER
1 filter   low-pass
2 cutoff   1.2 kHz
3 reso     20 %
```
- A turn on a sounding pad is heard at once and smoothly, no click ("Q2.1"), the same as
  the volume, the pitch and the envelope already do (scenarios "Glide while it sounds",
  "The music plays on").

- The picture: 1 of 3 ("Q3.1"), on the FILTER page the strip shows the filter curve in
  place of the wave: the envelope lives in time and lies on the wave, the filter acts on
  the frequencies and does not. The filtered wave (Q3.2) was left: cutoff and resonance
  cannot be read from it, and a low-pass barely changes a break's peaks.

```
 low-pass · 1.2 kHz · reso 40 %
 ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔╱╲
                      ╲
                       ╲▁▁▁▁▁▁▁▁▁
 20 Hz      200 Hz   1.2k   5k   20k
```

- The steps (default, not asked): the cutoff a semitone a tap, an octave a held key, a
  knob along the octaves 20 Hz–20 kHz; the resonance 1 % a tap, 10 % held. The project
  file gets three pad fields; absent reads "off", so older projects open unchanged (no
  version change, as the pitch and the envelope were added).

- The glide of a filter change is 10 ms (measured while coding): at 5 ms a jump of the
  cutoff from 200 Hz to 8 kHz overshot the tone's own step by 9 %, at 10 ms it does not.
  Volume and pitch glide 20 ms.

- The curve fits the strip (owner, 2026-10-31, a screenshot of the low-pass at 20 kHz
  with 73 % resonance: "fit it to the screen width, it flies past the edge"): half an
  octave of room at both ends of the frequency axis, the top of the level axis at least
  +12 dB and above the hump.

## Archive order

"Parameter pages" stands on 21, "The screen shows the layer" on 43: 44 is archived after
21 and 43, and its code comes after 43's (the pad panel of 43).

## Open questions

- How strong the band and the resonance sound at 100 %. Default: the resonance stops
  short of self-oscillation, so no pad whistles on its own; the owner judges by ear in
  `renders/`.
