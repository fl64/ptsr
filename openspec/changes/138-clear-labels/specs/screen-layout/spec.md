## ADDED Requirements

### Requirement: Chop knob labels name the trim and the hit level

On Chop, the knob strip SHALL name the trim knobs `trim<` and `trim>` and the threshold knob `hits`. The line of the last turned knob SHALL name them «trim start», «trim end» and «hit level».

#### Scenario: The trim knob names its edge
- **WHEN** the owner turns knob 5 on Chop
- **THEN** the knob strip says `trim<` and the line of the last knob says «trim start»
