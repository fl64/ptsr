<!--
Use <type>(<scope>): <summary> [<change>] for commits and this title. Use types and scopes from AGENTS.md.
Use an OpenSpec number or an approved range. Repository-only work uses a stable repo-<short-name> identifier.
Use this PR title unchanged for the final squash commit. Do not supply a different merge title.
Cover the full diff to the target branch. Aim for at most 250 words, excluding the checklist.
Complete each section. For an inapplicable section, give one short reason.
This is a review summary, not an agent activity log.
Do not include intermediate commit hashes, raw logs, detailed benchmark runs or acceptance history.
Apply the asd-ste100 skill before publication and after each edit. Run its lint on the title and body.
Report style checks to the owner, not in this body. Never hide a failure or material limit to shorten the text.
-->

## Description

<!-- Name the change and its user-facing effects in a few bullets.
State owner approval for combined changes. Keep future work separate. -->

## Why do we need it

<!-- State the problem and its effect in one or two sentences. -->

## What is the expected result

<!-- State the observable result and important behavior that stays unchanged. -->

## Validation

<!-- Use 3–5 short bullets: command and result, plus relevant failures or limits.
Do not repeat sub-checks covered by task check. Link detailed evidence.
A baseline failure remains a failure. A successful p99 check does not prove the maximum target.
Do not include style-tool reports, harmless harness skips or routine git checks. -->

- `task check`: not run.
- `task build`: not run.
- `openspec validate --all --strict --no-interactive`: not run.

## How to check by hand

<!-- Give exact steps for the owner. For a text-only change, state why no program check applies. -->

## Renders

<!-- For sound changes, list new or changed WAV files and what to hear. State attached or local.
Otherwise, give one sentence that explains why this change needs no new render.
Do not repeat old render lists or acceptance history. State relevant unverified manual checks in one sentence. -->

## Checklist

- [ ] The scope and changelog cover the full diff.
- [ ] Required check results and unresolved limits are explicit.
- [ ] Applicable specifications, tests and user documents are current.
- [ ] All commits have signoffs. Any spec archive has owner approval.
- [ ] The current title and head commit pass `Change names`. The squash title will use the PR title.

## Changelog entries

<!-- Give a few short user-facing bullets. Do not repeat implementation details.
For rules or tools, describe the process change. Sampler behavior stays unchanged. -->
