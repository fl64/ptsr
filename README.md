# ptsr (PT's SampleR)

```
          
  _ _/_ __
 /_// _\/ 
/         
```

ptsr is a sampler for the Linux terminal. You play pads from the computer keyboard and from MIDI controllers. You chop a break into slices, put the slices on pads and shape each pad: pitch, envelope, filter, mute groups and effects. Projects and the session stay on the disk.

Version 0.1.0. The specifications in `openspec/specs/` define the behavior. Each change starts as a specification. Tests and code follow.

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
- `F6` opens the files. Select a file. Press `Enter` to take it in hand. Hit a pad to load the file.
- `F3` opens the chop screen for the selected pad. `Y` cuts equal parts. `P` starts the assign of the slices to the pads.
- `Enter` on the pads screen opens pad edit. `Tab` changes the page. `I` and `K` select a parameter. `J` and `L` change its value.
- `F4` opens the shared FX screen. `Tab` selects FX 1 or FX 2. The same page knobs control the selected block.
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

The `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_STATE_HOME` variables move these folders. ptsr reads settings and sessions from the current `ptsr` folders only.

## Controllers

- ptsr has a profile for the M-Audio X-Session Pro: `config/devices/xsession-pro.toml`.
- `config/devices/smc-pad.toml` is a template for the M-VAVE SMC-PAD. It is not complete, and ptsr does not read it yet. The [SMC-PAD plan](docs/plans/SMC-PAD-INTEGRATION.md) describes the next integration, not current controls.
- A device without a profile plays the pads from notes 36–51.
- More than one controller can work at the same time.
- To bind a control to an action, press `F10` on a screen (learn mode), or use the MIDI tab in `F12`.

## Road to v1.0.0

One complete loop, from the keyboard and the SMC-PAD, with no clicks and no crashes:

**sample → chop → pads → pattern → resample → effects → export**

- **0.2 — Input:** status feedback, confirmation, button holds, relative encoders and complete SMC-PAD controls.
- **0.3 — Sound:** callback budget, click-free changes, pitch quality, master safety and recording latency.
- **0.4 — Patterns:** transport, recording, quantize, step editing, slots, note repeat and mute.
- **0.5 — Complete the loop:** sample tempo, resampling in tempo, core effects, quick sample operations and offline WAV export.
- **0.6 — Release:** project recovery, format compatibility, a four-hour soak, one release binary and a short manual.

Release needs three finished owner tracks and completed keyboard, controller and listening checks.
The callback timing issue and physical-controller checks remain open.
Everything outside this minimum is **1.x**. These are plans, not current capabilities.

See [the V1 plan and release criteria](docs/plans/V1.md), [the backlog](docs/BACKLOG.md)
and [owner checks](docs/LIVE-CHECKS.md).

## Development

- `AGENTS.md` gives the rules for the code: real time, tests, keys, the process.
- The process is OpenSpec (`npm install -g @fission-ai/openspec`). First a change in `openspec/changes/`, then the code. `openspec list` shows the open changes.
- `task check` runs name, format, dependency license, Clippy, test and audio benchmark checks. Commit only after they pass.
  The test `crates/app/tests/scenarios.rs` fails if a spec scenario has no test.
- The tests write WAV files to `renders/`. Listen to them.
- `docs/LIVE-CHECKS.md` lists the checks by hand and by ear.
- Each change goes in its own branch and a pull request to `master`.

### Commit and pull request names

Use one format for new commits and PR titles:

```text
fix(pads): keep original gate releases [85]
ci(build): check change names [repo-format]
```

The format is `<type>(<scope>): <summary> [<change>]`. See `AGENTS.md` for types and scopes.
Use an OpenSpec number or an approved range, such as `83a-85`.
Repository-only work uses one stable `repo-<short-name>` identifier, not a release label.
Write the summary in English. Start it with a lowercase ASCII letter or digit.
Existing `master` history stays unchanged.

Development checks need Python 3.10 or newer and Git. No Python packages are necessary.
Enable the commit hook once in each clone:

```sh
task hooks:install
```

Fetch the base branch. Check the new branch commits. Then make a commit with a signoff:

```sh
git fetch origin master
task check
git commit --signoff -m 'ci(build): check change names [repo-format]'
```

`task names` runs only the checker tests and new-commit checks.
The hook checks each proposed commit message. CI checks the PR title and all new commits.
CI also runs after a title edit. Unknown scopes, old-format titles and missing signoffs fail.
The protected `master` branch requires a PR and the `Change names` check.

Merge by squash with the unchanged PR title and an exact-head guard.
Keep the final commit's signoff. Do not override the title with `--subject` or API `commit_title`.
GitHub adds the PR number to the final commit. Check its title after merge.

The hook applies only to clones that enable it. CI cannot judge the quality of English.
GitHub without Enterprise cannot enforce a manually edited final squash title.
Administrators can change protection. The merge procedure remains necessary.

### Dependency license checks

Install the checked tool version:

```sh
cargo install cargo-deny --version 0.20.2 --locked
```

Run `cargo deny check licenses` for licenses only, or `task check` for all checks.
The policy in `deny.toml` covers workspace, normal, build and dev dependencies.
A new allowed license needs the owner's approval.

### Offline callback benchmark

Run `cargo bench -p engine --bench callback`. The benchmark needs no sound device or controller.
The benchmark does not start the sampler or change system scheduling settings.

It keeps the same 32 voices, 128 samples at 48 kHz, 20,000 playback blocks and 200 project
switches. The full block budget is 2.667 ms. The 50% target is 1.333 ms. Existing wall-time
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
only steady wall p99 and per-switch wall peak p99. A value strictly above 50% fails. Equality passes. A successful command does not prove that every block met the target. CPU values
and maxima do not replace those checks.

Before the workload, 2,000 empty calls show timer overhead. The separate
`measurement-envelope mean` includes sampling and storage. Raw empty-call means are in
microseconds. The benchmark does not subtract overhead from workload times.
A CPU clock error stops the command with an error, not invented zero CPU time.
Earlier slow-core CPU spikes remain open. This report does not optimize the engine.

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
OpenSpec-generated instructions. Review `git diff` before you commit them. They do
not archive changes or change the sampler keys.

OpenSpec 1.9.0 checks the version of the first installed skill. If only another skill
has an old version, `update` can miss it. Run `openspec:init` again with the same
`OPENSPEC_TOOLS` to regenerate the selected agents' instructions. The update task
does not use `--force`, because that option can also remove legacy OpenSpec files.

## License

- ptsr is free software under **GPL-3.0-or-later**. See [LICENSE](LICENSE) for the full terms.
- Music, samples and projects made with ptsr belong to their authors.
  The program's license does not cover these works.
- Documentation in `docs/` is under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
- Audio in `renders/` is generated by the project and released under [CC0](https://creativecommons.org/publicdomain/zero/1.0/).
  Do not add third-party samples to the repository.
