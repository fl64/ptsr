# Pad sound decisions

Preserve playback in older projects. Missing pan and fine tune fields mean
center and zero cents. An older project starts with linear velocity and the
old linear envelope. This also keeps patterns that play empty pads the same.
Save the project defaults and the pad overrides. When the owner loads the
first sound into a previously empty pad, use the new pad defaults for fields
with no explicit pad setting. Keep explicit settings and preserve them when
an assigned pad gets a different sample.

For mono, multiply the sample by left and right gains on a constant power arc.
Normalize both gains to one at center. At one side, that side has a gain of the
square root of two and the other side is silent. On the unscaled arc each gain
is minus 3 dB at center. Apply the square root of two as a common scale to keep
the legacy center level. For stereo, attenuate only the side opposite the
balance direction by one minus the absolute balance fraction. Send the result to the shared effects
after pan. Check the master ceiling with a full level pan change.

Store fine tune as signed cents from -100 to +100. Add it to the existing pitch
with a ratio of 2 raised to cents divided by 1200. Check the saved value, the
shown value, and both limits at 48 kHz.

Keep attack linear. Linear with zero hold keeps the tail fade at the source end,
as in an older project. With a hold or the exponential curve, measure attack,
hold and decay in output time. A zero decay leaves the level at full after
attack and hold. A positive decay starts at the end of hold. The source end
can still stop the voice first.

Exponential decay and normal gate release reach minus 60 dB at the set duration.
A final fade of up to 2 ms then brings the level to zero. Release time zero
keeps the short cut. A short cut that interrupts an exponential release starts
from its current gain. Steals, pad groups, project switch and Panic keep their
existing deadlines.

Check a saved linear tail against the old render. Check a short source with a long
hold and a pitched pad with the same hold time. Check a group cut and Panic both
before and during a normal release. These cases protect the timing at the
boundaries between the new envelope and the existing voice controls.

For a new pad, use 30 dB of velocity range. Pad velocity remains a raw value in
the pattern. For positive velocities, map step 1 to minus the range and step 127
to unity. Zero range gives the same gain at each positive velocity. Use the old
linear formula in legacy mode. The Velocity row has a Linear value before
0 dB, so the owner can restore the old response. Show Linear for a legacy pad.
Check a manual hit, a replayed hit, and a copied pad.

## Pad sound controls

Add a SOUND page to the pad edit. It shows Pan, Fine tune, Velocity and Curve.
The ENV page adds Hold after the existing Attack, Decay and Release rows.
The TONE page keeps the current pitch control and shows semitones and cents
in its pitch value.

Append the SOUND page to the page cycle. The pad edit panel gives it a
visible action in the unused P14 cell. Use the existing row encoders E7,
E5, E3 and E1 in that order. Keep the strip and FX settings in place.

Each normal Fine tune step changes five cents. Shift with a key or the
verified PAD BANK hold with E5 changes one cent. Use the same one-cent step
for a mapped knob while its device holds the learned fine control. If PAD BANK
hold is off or unavailable, E5 changes one cent in SOUND without a hold.
The fine state changes only the Fine tune row on SOUND. Keep parameter learn
available for Pan, Fine tune, Velocity, Curve and Hold.

Check the new rows at 80 by 26 and 120 by 40. Show the shape on the page
after the owner releases PAD BANK. A controller can reach the page, change
the values and return to the pad grid.

## Open questions

- Should the coarse Fine tune step be five cents? Start with five cents.
- Should Velocity range stop at 60 dB? Start with 60 dB.
- Which control gives one-cent steps on SMC-PAD? Use PAD BANK with E5.
  If the device cannot report the hold, use one-cent steps on E5.
