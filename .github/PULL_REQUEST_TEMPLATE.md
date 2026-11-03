<!--
Use <type>(<scope>): <summary> [<change>] for the title.
Use the scopes from AGENTS.md. Keep the description short.
Cover the full diff to the target branch, not only its last change.
Complete each section. If a section does not apply, state why.
Use the asd-ste100 skill before publication and after each edit.
Run its scripts/ste-lint.py on the title and body. Review each finding.
Use STE-flavored mode. Do not claim full dictionary compliance from a clean lint result.
Keep future plans and unverified results explicit.
-->

## Description

<!-- Name the OpenSpec change. State the user-facing changes.
For several areas, use an Area | Change table. State owner approval for combined changes.
Keep completed work, plans and spec archives separate. -->

## Why do we need it

<!-- State the problem and its effect on the user in one or two sentences. -->

## What is the expected result

<!-- State the observable result. Name the important behavior that stays unchanged.
Include technical detail only if the reader needs it to understand the result. -->

## Validation

<!-- Replace "not run" with results, counts and the commit.
State failures, skips and limits. A successful benchmark exit does not prove its maximum budget target.
Separate local checks from remote CI. State the skill mode and the STE lint result. -->

- `task check`: not run.
- `openspec validate --all --strict --no-interactive`: not run.
- `task build`: not run.
- ASD-STE100 review with the `asd-ste100` skill: not run.

### Listening artifacts

<!-- List the renders/*.wav paths and what to hear.
Link the exact manual steps in docs/LIVE-CHECKS.md.
State whether you attached the WAV files or kept them as local generated outputs.
For a non-sound change, state why the owner does not need to listen.
Keep agent screen checks, owner listening, actual key releases and hardware results separate. -->

## Checklist

- [ ] I described the full diff and used the repository rules for the title.
- [ ] I used the asd-ste100 skill and reviewed its lint results for the title and body.
- [ ] I added OpenSpec requirements and named tests for behavior changes, or explained why this does not apply.
- [ ] I reported the required checks and their results.
- [ ] I added `Signed-off-by` to every outgoing commit.
- [ ] I updated the user documents and manual steps, or explained why they need no change.
- [ ] I stated the owner and hardware results. I kept unverified checks explicit.
- [ ] I received explicit owner approval for the spec archive, or kept the change active.

## Changelog entries

<!-- Use a few short user-facing bullets. Include all completed areas.
Do not repeat implementation details. For rules or tools, state that sampler behavior stays unchanged.
Describe those changes instead. -->
