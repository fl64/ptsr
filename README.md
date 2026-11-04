# ptsr (PT's SampleR)

```
          
  _ _/_ __
 /_// _\/ 
/         
```

ptsr is a sampler for the Linux terminal. You play pads from the computer keyboard and from MIDI controllers. You chop a break into slices, put the slices on pads and shape each pad: pitch, envelope, filter, mute groups and effects. Projects and the session stay on the disk.

Version 0.1.0. The behavior is specified in `openspec/specs/`. Each change starts as a spec. Then a test and the code follow.

## What you need

- Linux with PipeWire and its JACK library (`pipewire-jack`), or a JACK2 server. If JACK is not available, ptsr uses ALSA.
- A terminal with the kitty keyboard protocol: Ghostty, kitty, WezTerm or foot. ptsr needs it for key releases. In other terminals, gate pads play as one-shot, and ptsr shows a warning.
- Optional: a terminal with the kitty graphics protocol shows the waves as pictures. Turn it on in F12 (`Wave`). Without it, the waves are Braille characters.
- Rust stable 1.85 or newer (the code uses edition 2024).
- Optional: [Task](https://taskfile.dev/installation/) for the short commands below.

## Install the build dependencies

Debian or Ubuntu:

```sh
sudo apt install build-essential pkg-config libasound2-dev pipewire-jack git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Fedora (the package names are not tested):

```sh
sudo dnf install gcc pkgconf-pkg-config alsa-lib-devel pipewire-jack-audio-connection-kit git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Arch:

```sh
sudo pacman -S base-devel pkgconf alsa-lib pipewire-jack git rustup
rustup default stable
```

The JACK library is not necessary for the build. ptsr loads it when it starts.

## Build and start

```sh
git clone git@github.com:fl64/ptsr.git
cd ptsr
task build    # or: cargo build --release -p ptsr
task run      # or: ./target/release/ptsr
```

Start ptsr from the repository folder. ptsr reads the controller profiles from `config/devices/` and the audio defaults from `config/audio.toml` in the current folder. If you start it from a different folder, ptsr does not know your controller.

## First steps

- The left half of the keyboard is the pads: `1 2 3 4`, `Q W E R`, `A S D F`, `Z X C V`, in any keyboard layout. `Shift` on a pad gives a full-strength hit.
- The right half, the arrows and the F keys are the commands. The bottom line shows the keys of the current screen. `F1` shows the full help.
- `F6` opens the files. Select a file and press `Enter` to take it in hand. Hit a pad to put the file on it.
- `F3` opens the chop screen for the selected pad. `Y` cuts equal parts. `P` starts the assign of the slices to the pads.
- `Enter` on the pads screen opens the pad edit. `Tab` turns the page, `I` `K` select a parameter, `J` `L` change it.
- `F4` opens the shared FX screen. `Tab` selects FX 1 or FX 2; the same page knobs control the selected block.
- `F12` opens the settings: the sound device, the buffer, MIDI and the knob response.

`docs/USER-STORIES.md` gives each task step by step.

## Files

| Path | What it holds |
| --- | --- |
| `~/.config/ptsr/config.toml` | your settings and the learned MIDI controls |
| `~/.config/ptsr/keymap.toml` | your keys over the built-in keymap (`config/keymap.toml`) |
| `~/.local/share/ptsr/session.toml` | the session: pads, chops, the open bank |
| `~/.local/state/ptsr/ptsr.log` | the log |
| `~/ptsr/projects/`, `~/ptsr/samples/` | the projects and the samples (change the roots in F12) |

The `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` variables move these folders. At the first start, ptsr copies the settings and the session from the old `arasaka-404` folders, if they exist.

## Controllers

- ptsr has a profile for the M-Audio X-Session Pro: `config/devices/xsession-pro.toml`.
- `config/devices/smc-pad.toml` is a template for the M-VAVE SMC-PAD. It is not complete, and ptsr does not read it yet. The [SMC-PAD plan](docs/plans/SMC-PAD-INTEGRATION.md) describes the next integration, not current controls.
- A device without a profile plays the pads from notes 36–51.
- More than one controller can work at the same time.
- To bind a control to an action, press `F10` on a screen (learn mode), or use the MIDI tab in `F12`.

## Next work

The [development plan](docs/plans/README.md) sets the order: keymap cleanup (83a), input
parity and refactors (84–90), SMC-PAD and its command panel (91–94), then transport and
patterns (95–101).
The goal is every action without typed text from the SMC-PAD, with the same logical
behavior as the keyboard. These features are planned, not shipped. Start with the
[keymap plan](docs/plans/KEYMAP-CLEANUP.md). Its cleanup 83a is implemented: Alt+R
records the microphone, Ctrl+G normalizes, H / Shift+H chop by hits, U/O turn banks on
Pads while J/L change values, Tab switches the assign order and Ctrl+C copies a pad;
the owner checks are in `docs/LIVE-CHECKS.md`. The owner accepted 83a without completing
those checks; it is archived. The owner checked and accepted input parity 84; it is
archived in `openspec/changes/archive/2026-11-03-84-input-parity/` on the same branch
by owner request. Physical-controller checks remain unverified, and callback benchmark
peaks above the 50% budget target remain unresolved. The owner accepted the benchmark
report below (84a) on 2026-11-03. It is archived in
`openspec/changes/archive/2026-11-03-84a-benchmark-report/` on the same approved branch.
The owner accepted input modes 85 on 2026-11-03. It is archived in
`openspec/changes/archive/2026-11-03-85-input-modes/`.
Closure of 85 and work on 86 share `86-screen-registry` by owner request.
Keyboard and MIDI share pad handling. Controller commands precede following target hits,
without delaying normal direct MIDI play. Mixed Chop holds and late direct notifications have separate repairs.
Keys, profiles, saved data and the audio callback stay unchanged. See section 85 of
`docs/LIVE-CHECKS.md` for separate keyboard/controller checks and the three 85 WAV files.
This is overall acceptance. Separate WAV listening remains unreported, and physical-controller checks remain unverified.
Recorded MIDI tests do not replace physical checks. The callback timing issue stays open.
The owner accepted the screen registry (86) on 2026-11-04.
Its archive is `openspec/changes/archive/2026-11-04-86-screen-registry/`.
Implementation is complete at `a186fbb`. Offline verification is recorded in `e90543c`.
Existing keys, help, learned assignments and FX controls stay unchanged for valid input.
Invalid shared MIDI state rejects new actions without losing original held-pad releases.
Section 86 of `docs/LIVE-CHECKS.md` records the evidence and separate keyboard, controller and listening checks.
This is overall acceptance, not individual keyboard, physical-controller or WAV listening results.
The callback timing issue remains open. Change 87 is next, but it is not active.

[Vision version 3](docs/VISION.md) also keeps longer-term ideas: modulation, parameter
locks, slice motion and the launcher. They are not requirements for the current step.

## Development

- `AGENTS.md` gives the rules for the code: real time, tests, keys, the process.
- The process is OpenSpec (`npm install -g @fission-ai/openspec`). First a change in `openspec/changes/`, then the code. `openspec list` shows the open changes.
- `task check` runs the format check, clippy, the tests and the audio benchmark. A commit is only made when it is green. The test `crates/app/tests/scenarios.rs` fails if a spec scenario has no test.
- The tests write WAV files to `renders/`. Listen to them.
- `docs/LIVE-CHECKS.md` lists the checks by hand and by ear.
- Each change goes in its own branch and a pull request to `master`.

### Offline callback benchmark

Run `cargo bench -p engine --bench callback`. No sound device or controller is needed.
The benchmark does not start the sampler or change system scheduling settings.

It keeps the same 32 voices, 128 samples at 48 kHz, 20,000 playback blocks and 200 project
switches. The full block budget is 2.667 ms; the 50% target is 1.333 ms. Existing wall-time
summary lines remain. New groups cover playback, all switch callbacks, and each switch
phase: first fade block, second fade block, clearing and restore.

Each group shows mean, p99, maximum, sample count and `>50 %` count for three times:

- `wall`: elapsed callback time, including time when the thread cannot run.
- `thread CPU`: time accounted to this thread. Its measurement also includes the two
  wall-clock reads and small timer overhead. It is not an exact profile of sound code.
- `estimated gap`: wall minus CPU for each block, with negative results set to zero.
  This is an estimate, not proof of a particular cause or program that delayed it.

`maximum target` says `EXCEEDED` when an observed wall or CPU maximum is above 50%.
Read that line and the counts even when the command succeeds. Exit status still checks
only steady wall p99 and per-switch wall peak p99: strictly above 50% fails; equality
passes. A successful command does not prove that every block met the target. CPU values
and maxima do not replace those checks.

Before the workload, 2,000 empty calls show timer overhead. The separate
`measurement-envelope mean` includes sampling and storage. Raw empty-call means are in
microseconds. No overhead is subtracted from workload times. A CPU clock error stops the
command with an error, not invented zero CPU time. Earlier slow-core CPU spikes remain
open; this report does not optimize the engine.

### OpenSpec agent setup

Install the OpenSpec CLI once with `npm install -g @fission-ai/openspec`. Then use:

```sh
task openspec:init                              # set up Pi
task openspec:init OPENSPEC_TOOLS=pi,codex       # or set up both agents
task openspec:update                            # update configured agent instructions
task openspec:check                             # validate specs and active changes
```

For Pi, OpenSpec uses `.pi/skills/` for skills and `.pi/prompts/` for commands.
The OpenSpec profile and delivery settings decide which instructions it generates.
Run `/reload` in Pi after setup or update.

These tasks do not install or upgrade the CLI. Setup and update can overwrite
OpenSpec-generated instructions; review `git diff` before you commit them. They do
not archive changes or change the sampler keys.

OpenSpec 1.9.0 checks the version of the first installed skill. If only another skill
is outdated, `update` can miss it. Run `openspec:init` again with the same
`OPENSPEC_TOOLS` to regenerate the selected agents' instructions. The update task
does not use `--force`, because that option can also remove legacy OpenSpec files.
