# Design

## Context

See `proposal.md` for the owner decision.
Startup calls `settings_moved` before reading current files. That helper calls `config::move_old_folders` and queues a copy notice.
The recursive copy helper has no other callers.

## Goals / Non-Goals

**Goals:** Remove this startup path without changing current file loading.

**Non-Goals:** Do not change session-version conversion, FX bindings, XDG resolution or sound processing.

## Decisions

- Remove the copy helpers, startup call and notice entry. Do not keep a disabled compatibility path because the owner no longer needs it.
- Replace the two migration-era requirements with current-file requirements. Keep the program name, client names, folder paths and XDG behavior unchanged.
- Replace three migration tests with tests that use the production settings, keymap and session loaders in temporary folders.
- Keep this change in the approved branch and PR, with commits separate from specification formatting.

## Risks / Trade-offs

- Legacy-only installations start with current defaults rather than copied settings. The owner explicitly approved this behavior.
- Automated file-loading checks do not prove hardware or listening results. These checks need no live sampler run.

## Migration Plan

The program performs no file migration or folder removal. Existing `ptsr` files remain in place.
Reverting the implementation commit can restore the old startup behavior if needed.

## Open Questions

None. The owner explicitly requested full removal, including code, tests and current documentation.
