## ADDED Requirements

### Requirement: Chop knob labels name the region and the hit level

On Chop, the knob strip SHALL name the region knobs `r.from` and `r.to` and the threshold knob `hits`. The line of the last turned knob SHALL name them «region start», «region end» and «hit level».

#### Scenario: The region knob names its edge
- **WHEN** the owner turns knob 5 on Chop
- **THEN** the knob strip says `r.from` and the line of the last knob says «region start»
