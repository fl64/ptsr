## Context

The UI and MIDI send commands to the audio thread through separate bounded queues. A
session restore sends more commands than the UI queue can hold. A pending load can also
finish after the project changes. The audio callback must not wait or allocate.

## Goals / Non-Goals

Keep a new hit on the open project's state. Fade out voices from the previous project
when switching, without clicks. Do not change project files.

## Decisions

The existing bounded-queue test reproduces the fault: a full UI queue drops the
commands that clear the old pads. The MIDI queue is independent, so a UI-queue marker
alone cannot order MIDI hits against a switch. Increasing either queue is not a fix.

Use a project generation shared by the UI, MIDI and audio threads. A switch publishes a
new generation before it sends any new project state. Every queued command carries the
generation observed by its producer. The audio thread checks the published generation at
block boundaries, rejects commands from other generations and starts a short fade of all
old voices and the old effects output. It retires sample references through the existing
garbage queue, including references in rejected commands. It never frees them in the
callback. The callback handles at most a fixed number of queue entries and effect-buffer
clearing operations per block; a busy producer cannot keep it in a drain loop forever.
If the garbage queue is full, the callback keeps ownership of every sample reference and
pauses work that needs more retirement space. It resumes when the consumer makes space.
It does not leak a reference or release its last owner in the callback. A stalled garbage
consumer can delay project readiness, but it must not delay the fade to silence.

The UI keeps the new project's restore commands in a non-RT pending queue and retries them
in order as the bounded UI queue has room. A later switch cancels that pending work and
starts a new generation. The audio thread clears old pad sounds and settings before it
accepts the new restore commands. It fades the old effects output to zero, then clears the
delay and reverb storage in bounded increments before it accepts new hits. This includes
an effect whose kind did not change. No old effect tail can return later. Neither the
fade nor the clearing allocates or waits in the callback.

A final restore marker follows all new settings. The audio thread publishes readiness for
that same generation only after it has processed the marker, finished the fade and cleared
the old effects. Keyboard and MIDI producers check readiness when a hit is received and
drop the hit if it is not ready. The audio thread also checks the hit's generation and
readiness when it consumes the hit. Thus a hit racing a switch cannot sound in the new
project or wait until it becomes ready. A second switch invalidates both an earlier marker
and any earlier readiness acknowledgement. Ordinary sample loads still use per-pad load
generations: invalidate all of them at each switch and do not let a stale leader consume
followers of a newer load. Loaded sounds and their regions must be delivered in order;
queue pressure must not leave a new sound with old settings or an old region.

Test a full UI queue, queued and concurrent MIDI hits, two switches before acknowledgement,
a late load and a stale leader, effects with an unchanged kind, fade continuity, and return
to saved pad settings. The tests use the actual bounded queues and offline audio. Count both
allocations and deallocations in the callback during a switch; measure the callback budget
with 32 voices, effects, and transition work.

## Risks / Trade-offs

A switch can overlap MIDI hits and a late load. Test both paths without hardware and
keep the audio callback lock-free and allocation-free.

## Open Questions

None.
