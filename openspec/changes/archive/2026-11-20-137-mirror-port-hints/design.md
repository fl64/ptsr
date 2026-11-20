# Design

## Owner decisions, 2026-11-20

- Fix the blinking hints before change 135.

## Decisions

### D1. The mirror port is not an input device

The SMC-PAD Private port repeats the notes and controls of the Master port. Its profile is
empty, so it runs no action. A message from a port whose name contains `SMC-PAD-Private` does
not change the last input device. The connection of that port stays tracked, as now.

## Corner cases

| # | Case | Expected result | Status |
| --- | --- | --- | --- |
| E1 | Master hit first, Private copy second | SMC-PAD words stay | test |
| E2 | Private copy first, Master hit second | SMC-PAD words stay | test |
| E3 | A key, then only a Private message | Keyboard words stay | test |
| E4 | Only the Private port has a connection | Keyboard words stay, because Private runs no action | keep |

## Open questions

None.
