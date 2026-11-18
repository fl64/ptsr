## ADDED Requirements

### Requirement: Diagnostic mode records stalls and inputs

`--diag` or `PTSR_DIAG=1` SHALL turn on diagnostic mode for one run.
In this mode, the program SHALL log each keyboard action and MIDI input.
It SHALL log an audio callback that runs longer than 100 ms, with its duration and stage.
It SHALL set the soft real-time CPU limit below the hard limit and log SIGXCPU.
The audio callback SHALL write only atomics for this record.
Without diagnostic mode, the program SHALL write no input or stall lines.

#### Scenario: Diagnostic mode reports a stalled callback
- **WHEN** one audio callback runs longer than 100 ms in diagnostic mode
- **THEN** the log gets one warning with the duration and the callback stage
- **AND** a callback that ends in time gives no warning

#### Scenario: Diagnostic mode logs inputs
- **WHEN** the owner presses a key or sends MIDI in diagnostic mode
- **THEN** the log gets one line for each action or MIDI input

#### Scenario: Normal start keeps the log quiet
- **WHEN** the program starts without `--diag` and without `PTSR_DIAG=1`
- **THEN** diagnostic mode is off and the log gets no input or stall lines

### Requirement: Diagnostic mode reports the real-time load

In diagnostic mode, the program SHALL log one line each second.
The line SHALL give the callback count, the callback time and the CPU time of each real-time thread.

#### Scenario: Diagnostic mode reports the real-time load each second
- **WHEN** one second passes in diagnostic mode
- **THEN** the log gets one line with the callback count and the callback time in that second
- **AND** the line gives the CPU time of each real-time thread of the program
