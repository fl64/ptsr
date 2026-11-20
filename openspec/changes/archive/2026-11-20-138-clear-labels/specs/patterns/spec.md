## ADDED Requirements

### Requirement: Erasure shows its state

While erasure is on, the mode line SHALL say «ERASE A1». It SHALL also say what a pad hit erases and which control turns erasure off. This applies in a pass and outside a pass. The off control SHALL be the erase cell or PAD BANK with device pad 1 after SMC-PAD input, and Esc after keyboard input. The line SHALL go when erasure ends.

#### Scenario: Erasure in a pass shows its state
- **WHEN** an A1 pass records and the owner turns on erasure with the SMC-PAD panel
- **THEN** the mode line says «ERASE A1» and names the off control
- **AND** a second press of the erase cell turns erasure off and the line goes

## MODIFIED Requirements

### Requirement: Erasure modes do not change into each other

Erasure outside a pass SHALL end when a recording pass starts or the project changes. It SHALL also end when the screen changes from Pads, and on Esc, on PAD BANK with device pad 1 or on a second toggle. PAD BANK with device pad 1 SHALL end erasure first and SHALL keep the screen; the next press SHALL go back as usual. Erasure inside a pass SHALL end with the pass. A pass SHALL start with ordinary recording. Erasure SHALL NOT start during CountIn or Finishing. Ordinary holds from before erasure SHALL keep their original releases.

#### Scenario: Starting a pass leaves whole-pad erasure
- **WHEN** whole-pad erasure is on and the owner starts recording
- **THEN** erasure turns off before CountIn, and pad presses in the pass record hits
- **AND** the start erases no row

#### Scenario: A pass end does not leave whole-pad erasure on
- **WHEN** held erasure is on in a pass and the pass ends
- **THEN** erasure is off, and the next pad press sounds and erases no row

#### Scenario: Leaving Pads leaves whole-pad erasure
- **WHEN** whole-pad erasure is on, and the owner opens Chop and then returns to Pads
- **THEN** erasure is off, and pad presses on Chop and Pads sound as usual

#### Scenario: One P1 ends erasure and keeps the screen
- **WHEN** whole-pad erasure is on and the owner presses PAD BANK with device pad 1 once
- **THEN** erasure is off, the screen stays and a notice says «erase off»
- **AND** the next pad press sounds and erases no row
