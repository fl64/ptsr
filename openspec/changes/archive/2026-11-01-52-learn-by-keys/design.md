# Design 52

## Decisions (owner, 2026-10-31)

- The owner's flow: a key picks, the element lights, a control is moved, a question,
  saved only after `Enter`. The other element a taken control drives lights as well.
- Every pad parameter its own learnable knob ("Q2.1", the default taken while the owner
  answered with the flow): `pad.attack`, `pad.decay`, `pad.release`, `pad.filter`,
  `pad.cutoff`, `pad.reso`, `pad.start`, `pad.end` next to `pad.volume` and `pad.pitch`.
  The page knobs of the edit (`edit.param_1..3`) stay for the profiles.
- The walk by zones (47) goes: the keys pick; only the parameters, which have no key of
  their own, are walked with `I` `K`.

```
 LEARN · picked: pad Q · move a control on the controller
         a pad or a command key picks another · I K the parameters · F4 done
 ...
 LEARN · k5 → cutoff · k5 drives volume now, it lights · Enter save · Esc no
```

## Archive order

52 replaces requirements of 47: it is archived after 47.

## Open questions

- `Backspace` on a picked element with a control. Default: asks the same way ("clear k5
  from cutoff? Enter yes · Esc no").
