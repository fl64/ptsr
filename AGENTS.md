# Agent rules — ptsr (PT's SampleR)

You write a terminal sampler in Rust for Linux. The project owner **does not read code**. The owner reads the specifications, listens to the renders and tries the program by hand. Talk to the owner in Russian. Be short, and do not use jargon where you can do without it.

## Process (OpenSpec)

- We work through OpenSpec. For any change of behavior, first make a change (proposal, spec deltas, tasks), then write code.
- **Work from the specs, not from the memory of the conversation.** The code implements what the specs of the active change and `openspec/specs/` say, and only that. What is not in a spec is not done. If a spec misses something, first edit the spec, then write the code.
- The requirements are a draft. If, during implementation, you see that a requirement is inconvenient or contradicts another requirement, **stop and propose a specification edit**. Do not silently work around the specification.
- One change at a time. Small changes are better than large ones.
- New behavior work uses one OpenSpec number, one branch and one PR per change. Do not combine new changes or add a letter suffix to a new number unless the owner requests that exception in the PR. Historical identifiers stay unchanged.
- Each change goes in its own branch and a pull request to `master`. Do not commit to `master` directly.
- After each completed task, make a commit with a clear message.
- **Everything you agree on with the owner goes into the spec at once.** A decision from the conversation (behavior, UX, keys, knobs, texts) goes into the proposal/design/spec/tasks of the active change before the code. Write a question without an answer in design.md, section "Open questions", together with the option you took by default.
- After an owner decision changes behavior, update `docs/plans/` and `docs/VISION.md` in the same change when that decision affects them.
- After owner acceptance, archive the change in the same PR or in an immediate repository closure PR. Do not start another behavior change while a merged change remains unarchived, unless the owner allows it.
- Run `task specs:sync` when you archive a change. The check compares archived scenarios with the main specs and warns about merged changes that still need an archive.
- **Commit after each step, and keep commits atomic**: one commit = one logical edit. Do not mix these in one commit: different steps, specs and code of other tasks, an AGENTS.md edit and code. An artifact edit without code is a separate commit. Code with the spec of its own step is one commit. Complete a step only after its checks pass and you commit it.
- Before a code commit, run `cargo fmt --all --check`, Clippy with `--all-targets -- -D warnings` for the changed crates, and tests for the changed behavior and its callers. Run the recorded keyboard and raw MIDI tests for input changes. For a TUI drawing change, run its TUI tests and compare the ANSI screen catalogue with the base. For a sound or audio-thread change, run the required offline render checks and the callback benchmark before the commit.
- Before a text-only commit, run the checks for that text: `task openspec:check` for OpenSpec files and STE lint for English text. Do not run Rust tests only because text changed. Run `cargo deny check licenses` when dependencies or license rules change. Fix a failed required check before the related commit.
- Before the first push of a code change or a code PR, run the full `task check` on the final code and test state. If code or tests change after it passes, run it again. For a text-only PR under `openspec/` or `docs/`, run `task specs:check` and `task names` before push, without Rust checks. For other text-only PRs, run their text checks and `task names`. Do not rerun `task check` only because the PR title or body changed. Report any required check failure, including a failure that passed on a later run.
- **Use one title format for commits and pull requests:** `<type>(<scope>): <summary> [<change>]`. Example: `fix(chop): keep the cursor at the cut [09]`. This rule includes the final squash commit. Use the types, identifiers and scopes below. Keep each commit atomic.
- At the end of a change, write to the owner: what is done, **what to check by hand and by ear** (exact steps), which files in `renders/` to listen to.
- All repository texts are in English and follow ASD-STE100 Simplified Technical English: `README.md`, `AGENTS.md`, `docs/`, the `openspec/` artifacts and commit messages. Key names of the Russian layout (`х`, `Й`) are technical names and stay. Talk to the owner in Russian.
- Prefer the `asd-ste100` skill for new or revised English documentation. Use STE-flavored mode for descriptions and Strict mode for procedures. Run its `scripts/ste-lint.py` on the changed text. Review the results without a full dictionary compliance claim. Keep the original meaning and technical names. Do not rewrite unrelated documents only for style.
- `docs/VISION.md` and `docs/plans/` are guides, not requirements. Requirements are only in `openspec/specs/` and in active changes. A deferred draft does not authorize code.
- Do not increase the line count of `crates/app/src/main.rs` or `crates/tui/src/lib.rs`. Put new code in modules. The required CI check compares each file with the PR base. The owner can approve an explicit exception by applying the `allow-large-file-growth` label to that PR.

## Manual spec review

- CI cannot decide if `docs/plans/` or `docs/VISION.md` contradict the specs after an owner decision. Update the affected guides in the same change as that decision.
- CI cannot prove that every item in a proposal's "What Changes" list is true on the screen or in the sound. Before the owner check, walk that list and the named screen catalogue entries. Report every item that is not done.

## Branch and pull request names

- For OpenSpec work, use the change folder name for the branch: `<change>-<short-name>`, for example `85-input-modes`.
- For repository rules, tools or documentation without an OpenSpec change, use one stable `repo-<short-name>` identifier and branch. Example: `repo-format`. Do not use release labels such as `v1` as change identifiers. Do not add a scope to branch names.
- Use `<type>(<scope>): <summary> [<change>]` for every new commit and PR title. Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
- Write a nonempty English summary. Start it with a lowercase ASCII letter or digit. Keep the title on one line. Use the scopes in the table below.
- Use the OpenSpec number, with its optional lowercase letter, as the change identifier. Examples: `85`, `86a`. Repository-only work uses its stable `repo-<short-name>` identifier.
- Combine changes in one branch or PR only with the owner's explicit approval. Use an ascending numeric range with an ASCII hyphen or an en dash. Example: `fix(input): keep controller commands before pad hits [83a-85]`.
- Historical titles stay unchanged. This rule applies to new commits, not to the existing history of `master`.
- Enable the local commit hook with `task hooks:install`. The hook checks the title and a `Signed-off-by` trailer. Use `git commit --signoff`.
- Run `task names` before publication. It tests the checker and checks all branch commits against `origin/master`. Fetch `origin/master` first. `task check` includes this task.
- The `Change names` CI check must pass before merge. It checks the PR title and every new commit, including signoffs. A title edit starts a new check.
- The `Specs` CI check must pass before merge. It checks OpenSpec format, archived scenarios, proposal catalogue names and PR body sections. A body edit starts a new check. Until the Rust job of `repo-fast-checks` exists, `Specs` also runs the scenario coverage test.
- Merge only by squash. Let GitHub use the PR title for the final commit. Do not supply a different `--subject` or API `commit_title`. Only GitHub's final commit may add ` (#<PR number>)`.
- Before merge, check the current title, head commit and successful CI result. Use an exact-head merge guard. Keep the final commit's signoff. After merge, check its title against the PR title.
- Keep `master` protected: require a PR, `Change names` and `Specs`, with no bypass actors. Require the Rust job when `repo-fast-checks` adds it. Do not push directly or use an admin merge bypass.
- CI checks the format, not the quality of English. Local hooks are optional for other clones. GitHub without Enterprise cannot enforce the manually edited final squash title. Follow the merge procedure above. Administrators can change repository protection.
- The title gives the main purpose. The description must cover the full diff to the target branch, not only the last change. Keep completed work, future plans and unverified checks separate.
- Use [the PR template](.github/PULL_REQUEST_TEMPLATE.md). Complete each section. If a section does not apply, state why in one sentence.
- For code under `crates/`, give exact steps in "How to check by hand". Name the WAV files in "Renders", or give one sentence that explains why the change needs no new render.
- Write for the reviewer, not as an agent activity log. Aim for at most 250 words, excluding the checklist. State the user-facing effect first.
- Keep `Validation` to 3–5 short bullets. Give commands and results, plus failures or limits that affect readiness. Link supporting evidence instead of copying it.
- Keep required check failures visible, even when the unchanged baseline also fails. Do not hide a material limit to meet the length target.
- Do not repeat sub-checks already covered by `task check`. Explain a skip only if it leaves a required check incomplete. State remote CI results only when they affect readiness.
- Keep intermediate commit hashes, agent history, raw logs, detailed benchmark runs and style-tool reports out of the PR body. Link detailed records instead. For a performance change, include only the measurements needed to judge its effect.
- For a sound change, list new or changed renders, what to hear and whether the files are attached or local. Otherwise, state that sound stays unchanged and link the manual checks. Do not repeat old render lists or acceptance history. State relevant unverified hardware or listening checks in one sentence.
- Use the [asd-ste100 skill](https://github.com/danyuchn/asd-ste100-skill) for every PR title, description and changelog. Apply it before publication and after each edit. Use its STE-flavored mode for PR text. Apply its structural rules and review its lexical advice. Keep technical names and the original meaning.
- Run the skill's `scripts/ste-lint.py` on the PR title and body. Review each finding. Revise the text or explain retained wording to the owner. Report the skill mode and lint result to the owner, not in the PR body. A clean lint result does not prove full ASD dictionary compliance. If the skill is unavailable, tell the owner before publication. Do not claim a review or check that you did not do.
- Use the same scopes for commits and PRs. Choose the main affected area, not a Rust crate name. Tests use the scope of the behavior they test, not `tests`. For a PR with several areas, choose the main scope and describe the other areas. Keep the atomic commit rule above.
- Do not add scope aliases. Use `input`, not `keyboard`, `keymap` or `controls`. Use `tui`, not `ui`.

| Scope | Area |
| --- | --- |
| `pads` | Pad play, copy, swap and pad parameters |
| `chop` | Chopping, cursor, slices and slice assignment |
| `files` | File browser, filter and sample loading |
| `settings` | Settings and their persistence |
| `midi` | MIDI devices, profiles and control learning |
| `input` | Keys, event routing and shared keyboard/MIDI rules |
| `tui` | Drawing, indicators, hints and layout |
| `engine` | Sound, voices, effects and audio-thread work |
| `specs` | OpenSpec requirements and artifacts only |
| `docs` | Documentation and plans |
| `build` | Build, dependencies, Taskfile, CI and development tools |

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
- Automatic checks of renders: no NaN or infinities, no clipping, required silence, level, timing ±1 sample and clicks.
- Render every new effect to WAV. Check level, timing and clicks automatically.
- For fx-core amount controls, check RMS at 0 %, 50 % and 100 %, within ±1.5 dB of the reference.
  Define reference sounds and measurement windows in the design.
- Render every roll-matrix cell at 172 BPM. Check press between steps, release between steps, and press/release within one step.
  Check quantized starts, the running pattern and return without unintended gaps or doubled hits.
- For each sound feature: a WAV in `renders/` for the owner.
- Do not add third-party samples, breaks or kits to the repository.
  Tests and repository renders use generated sounds or the owner's recordings released under CC0.
- If an optional demo needs a well-known break, the owner downloads it and keeps it outside the repository.
  Only that optional demo test skips when the external file is missing.
  Required tests use the permitted audio sources and do not skip because a demo file is absent.
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

- The project is under GPL-3.0-or-later. For each new dependency, check that its license is compatible.
- `cargo deny check licenses` checks every dependency in `task check`, including build and dev dependencies.
- A new license in `deny.toml` needs the owner's approval. Do not add an exception to bypass this rule.
- GPL-2.0-only is not compatible with the project's license. Do not add a dependency under GPL-2.0-only.
- Do not add heavy dependencies without a reason. Name the reason in the proposal.

## What not to do

- Do not implement what is not in the active change, even if VISION.md has it.
- Do not change the format of project files and configs without a migration and without a change.
- Do not say "done" if you did not run the tests and did not put the renders in place.
