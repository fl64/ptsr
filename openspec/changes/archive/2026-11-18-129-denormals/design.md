# Design

## Evidence

Change 128 logged the callback load once a second on the owner's machine.
The callback count stayed at 95 calls per second. The callback time rose to
946 ms per second, then SIGXCPU and SIGKILL came. A probe used the owner's FX:
two reverbs, pads with a low-pass filter, drive and sends. Without flushing, the time
rose from 2 ms to 173 ms per second after the hits ended. With flushing, it stayed at 2 ms.

## Flush

At the start of `Realtime::process`, the thread sets FTZ and DAZ in MXCSR on x86_64
and FZ in FPCR on aarch64. Other targets keep their default. The setting is cheap,
and a repeat in each callback also covers an audio server that changes threads.

## Checks

- A unit test plays the owner-like FX set, then 60 s of silence. The output must reach
  exact zero, and a subnormal product in the callback thread must be zero.
- The benchmark adds a decaying-tail workload. Without flushing, its mean load was
  10.75 % and its maximum 105 %. With flushing, the mean was 0.31 %. The check fails
  over a 2 % mean or a 50 % p99.
- An offline render `renders/129-reverb-tails.wav` gives the tail for the owner.
