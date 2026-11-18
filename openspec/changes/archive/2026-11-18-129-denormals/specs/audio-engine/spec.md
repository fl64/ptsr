## ADDED Requirements

### Requirement: The audio callback flushes subnormal numbers

At the start of each callback, the audio thread SHALL treat subnormal floating-point numbers as zero.
Decaying effect and filter tails SHALL reach exact silence.
The callback benchmark SHALL measure decaying reverb tails after pad hits.
It SHALL fail when their mean load is over 2 % or their p99 is over 50 % of the budget.

#### Scenario: Decaying reverb tails reach exact silence
- **WHEN** pads with a filter, drive and sends play into two reverbs and then stop for 60 seconds
- **THEN** the output reaches exact zero
- **AND** a subnormal product in the callback thread gives zero
