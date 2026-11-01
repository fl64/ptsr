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
- `config/devices/smc-pad.toml` is a template for the M-VAVE SMC-PAD. It is not complete, and ptsr does not read it yet.
- A device without a profile plays the pads from notes 36–51.
- More than one controller can work at the same time.
- To bind a control to an action, press `F10` on a screen (learn mode), or use the MIDI tab in `F12`.

## Development

- `AGENTS.md` gives the rules for the code: real time, tests, keys, the process.
- The process is OpenSpec (`npm install -g @fission-ai/openspec`). First a change in `openspec/changes/`, then the code. `openspec list` shows the open changes.
- `task check` runs the format check, clippy, the tests and the audio benchmark. A commit is only made when it is green. The test `crates/app/tests/scenarios.rs` fails if a spec scenario has no test.
- The tests write WAV files to `renders/`. Listen to them.
- `docs/LIVE-CHECKS.md` lists the checks by hand and by ear.
- Each change goes in its own branch and a pull request to `master`.
