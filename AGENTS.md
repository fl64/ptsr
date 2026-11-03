# Agent rules — ptsr (PT's SampleR)

You write a terminal sampler in Rust for Linux. The project owner **does not read code**. The owner reads the specifications, listens to the renders and tries the program by hand. Talk to the owner in Russian. Be short, and do not use jargon where you can do without it.

## Process (OpenSpec)

- We work through OpenSpec. For any change of behavior, first make a change (proposal, spec deltas, tasks), then write code.
- **Work from the specs, not from the memory of the conversation.** The code implements what the specs of the active change and `openspec/specs/` say, and only that. What is not in a spec is not done. If a spec misses something, first edit the spec, then write the code.
- The requirements are a draft. If, during implementation, you see that a requirement is inconvenient or contradicts another requirement, **stop and propose a specification edit**. Do not silently work around the specification.
- One change at a time. Small changes are better than large ones.
- Each change goes in its own branch and a pull request to `master`. Do not commit to `master` directly.
- After each completed task, make a commit with a clear message.
- **Everything you agree on with the owner goes into the spec at once.** A decision from the conversation (behavior, UX, keys, knobs, texts) goes into the proposal/design/spec/tasks of the active change before the code. Write a question without an answer in design.md, section "Open questions", together with the option you took by default.
- **Commit after each step, and keep commits atomic**: one commit = one logical edit. Do not mix these in one commit: different steps, specs and code of other tasks, an AGENTS.md edit and code. An artifact edit without code is a separate commit. Code with the spec of its own step is one commit. Commit only when `task check` is green. A step is not done until it is committed.
- **A commit message carries the scope of what changes.** Format: `<change>: <scope> — what changed` (for example, `09: chop — cursor: arrows move it, snaps to peaks, M cuts on pause`). The scope is the area the owner sees, not a crate name for its own sake: `chop`, `pads`, `files`, `settings`, `midi`, `tui` (drawing), `engine` (sound), `specs` (artifacts only), `docs`, `build`. The main scope goes first; a step that touches two areas is two commits (or takes the scope of the area the step was made for).
- At the end of a change, write to the owner: what is done, **what to check by hand and by ear** (exact steps), which files in `renders/` to listen to.
- All repository texts are in English and follow ASD-STE100 Simplified Technical English: `README.md`, `AGENTS.md`, `docs/`, the `openspec/` artifacts and commit messages. Key names of the Russian layout (`х`, `Й`) are technical names and stay. Talk to the owner in Russian.
- `docs/VISION.md` and `docs/plans/` are guides, not requirements. Requirements are only in `openspec/specs/` and in active changes. A deferred draft does not authorize code.

## Controller-first development

- Read `docs/PROJECT.md` and `docs/plans/README.md` before preparing the next change. The order is keymap cleanup (83a), input and visual control (84–90), SMC-PAD (91–94), then transport and patterns (95–101). Keep the owner decisions in each change's design. Ask about J/L on Pads before 83a code; proposed option A is not acceptance.
- The goal is every action without typed text from the SMC-PAD, without losing keyboard controls. The plans do not prove that goal is met. Measure the Linux device in 91 before accepting its profile.
- From 84, test each shared input scenario with keyboard events and raw MIDI through the real profile and dispatch. Controller-only holds, layers and velocity need MIDI tests, not a fake keyboard equivalent. Do not replace production routing with a test-only path.
- Refactors keep the agreed behavior and keymap. List each behavior fix in a delta with a test; do not hide a race fix inside a code move. New scope or a contradiction means stop and propose a spec edit.
- From 87, each new mode needs its visible badge, pad state and hints. From 92, each new action states its keyboard and controller paths, or its text-only exception. Change 93 checks reach from screens and modes; 93a adds comfort before transport. Pad lights (94) are optional.
- Each proposal gives separate owner checks for keyboard and controller, plus the renders to hear. Keep current capabilities, planned work and unverified hardware facts separate in documents and reports.

## Real time — you must not break it

- In the audio callback: no memory allocation, no mutexes, no I/O, no logging, no panics.
- Communication between threads: only lock-free queues (`rtrb`) and atomics.
- A background thread decodes the samples and loads them into memory. Then the background thread sends the ready samples to the audio thread.
- Smooth every change of volume and parameters. No clicks.
- The UI and MIDI run in their own threads. A UI freeze does not stop the sound.

## Tests — the agent does not hear sound

- The engine can make an **offline render**: a scenario (pad hits, time, parameters) → WAV without a sound card.
- Automatic checks of renders: no NaN and no infinities, no clipping, silence where it must be, hit timing ±1 sample.
- For each sound feature: a WAV in `renders/` for the owner.
- Audio callback benchmark: at 32 voices (polyphony from 02), not more than 50 % of the buffer budget (128 samples / 48 kHz).
- Test the input logic (keyboard, MIDI, keyboard layouts) without hardware, on recorded event sequences.
- **Every user scenario in the specs is covered by a test.** A requirement or scenario goes into a spec — the same change adds a test that walks it: input logic on recorded events, state in a unit test, sound in an offline render with the automated checks. Name the test after the spec scenario, so the name shows which scenario is covered. What cannot be automated (how the sound feels) the owner checks in `renders/`, but even there an automated check for clipping, silence and timing is required. A step is not done until its scenarios are covered. The test `crates/app/tests/scenarios.rs` checks this in `task check`: a scenario of `openspec/specs/` with no test that names it fails the check. A scenario that you cannot test without hardware goes into `openspec/untested.md` with its reason, and only there. Archive a change only when this test passes with its scenarios in the main specs.
- **The agent runs the program live only when it is cut off from the owner's sound.** At that time the owner listens to their own sound (Bluetooth headphones). A second JACK client with a small buffer can break that sound. Config in `target/ux/cfg/ptsr/config.toml`: `[audio] backend = "jack"`, `buffer_size = 1024`. Variables `PIPEWIRE_REMOTE=ptsr-none JACK_DEFAULT_SERVER=ptsr-none` and `XDG_CONFIG_HOME`/`XDG_STATE_HOME`/`XDG_DATA_HOME` inside `target/ux`. Run it in a separate server `tmux -L ptsr-ux`. Capture the screen with `capture-pane -p`. After the first start, check: the log has `JACK unavailable` and has no `JACK: active`, and `pw-cli ls Node | grep ptsr` is empty. This does not cut off MIDI: the run reads the owner's controller. This does not let you check sound (Space, cursor, the cut by ear). The owner checks it.

## Keyboard and controllers

- The left half of the keyboard is **only pads**: physical keys `1 2 3 4 / Q W E R / A S D F / Z X C V`, in any layout (including Russian), with a key release event. Shift on pads = full hit velocity.
- The only exception is an open text input line (filter, name). While it is open, all keys type text and the pads are silent. The line opens explicitly (`/`) and closes with Enter or Esc.
- Commands are only on the right half, arrows, F keys, Space, Enter, Esc, Tab, Backspace and Ctrl/Alt+letter. **Never** assign commands to the keys of the pad block. This rule also applies to these keys with Shift. Shift on pads is only hit velocity. With arrows and other command keys, Ctrl, Alt and Shift are modifiers (Shift is a fine step).
- The program thinks in **logical actions** (`chop.cut`, `slice.fade`, `pad.trigger`), not in the buttons of a specific device. Profiles in `config/devices/*.toml` and `config/keymap.toml` decide what each button does.
- Several MIDI devices work at the same time. One setting in F12 (MIDI tab) decides how absolute knobs and faders catch a value: `jump` (default, the parameter goes to the knob position at once), `pickup` (waits until the knob reaches the value) or `scale` (catches up toward the end stop). Takeover is counted separately for each device and knob. A value jump is smoothed without a click.
- At the bottom of each screen: help by groups, with the current states of the toggles. F1 shows the full help.

## Stack

Rust stable. Sound: `jack` through PipeWire-JACK (fallback: `cpal`). MIDI: `midir`. UI: `ratatui` + `crossterm` with the kitty keyboard protocol. Decoding: `symphonia`. Transients: `aubio`. Configs: `serde` + TOML. Queues: `rtrb`.

## License and dependencies

- The project is under GPL-3.0. For each new dependency, check that its license is compatible.
- Do not add heavy dependencies without a reason. Name the reason in the proposal.

## What not to do

- Do not implement what is not in the active change, even if VISION.md has it.
- Do not change the format of project files and configs without a migration and without a change.
- Do not say "done" if you did not run the tests and did not put the renders in place.
