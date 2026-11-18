# Design

## Owner decision

On 2026-11-18 the owner asked for an optional diagnostic mode, not a one-time patch.

## Start

`--diag` on the command line or `PTSR_DIAG=1` turns the mode on for this run.
The mode is not saved. The config format stays the same.

## Callback watch

`Stats` keeps the start time of the running callback in microseconds and a stage
number. The callback stores them with relaxed atomics and stores zero at the end.
A diagnostic thread reads them every 20 ms. When one callback runs longer than
100 ms, the thread writes one warning with the duration and the stage. It writes
one warning for each stalled callback. The stages are: commands, project, render,
publish.

## Kernel limit

The audio server makes the callback thread real-time. The process has a 200 ms
real-time CPU limit. Over that limit the kernel sends SIGKILL without a message.
In diagnostic mode, the program sets the soft limit to half the hard limit after
the audio start. A SIGXCPU handler writes a fixed line to the log file with
`write`, which is safe in a signal handler. The process continues to the hard limit.

## Load each second

The first owner test logged SIGXCPU but no stalled callback. The real-time thread
used more than 100 ms of CPU without a sleep, but no single callback was long.
Each second, the diagnostic thread logs the callback count, the summed callback
time and the CPU time of each real-time thread of the process. It reads the thread
policy and CPU time from `/proc/self/task/<tid>/stat`. The last line before a kill
shows if the audio server calls the callback too often or uses the CPU itself.

## Inputs

Each keyboard action and each MIDI input gives one info line in diagnostic mode.
The UI and MIDI threads write the lines. The audio thread never writes the log.

## Open questions

- Is 100 ms the right warning threshold? The default is 100 ms, half the kernel limit.
