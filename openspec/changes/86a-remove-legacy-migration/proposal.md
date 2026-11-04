# Proposal

## Why

The owner no longer uses `arasaka-404` settings and requested removal of their automatic migration.
Startup still checks and copies these obsolete folders.

## What Changes

- **BREAKING:** Stop importing settings and sessions from `arasaka-404` folders.
- Remove the copy code, startup message and three migration tests.
- Test current `ptsr` files and startup without current files.
- Remove obsolete migration instructions from current documentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `projects`: Read current `ptsr` files without copying older program folders.

## Impact

The affected areas are startup, settings loading, tests and documentation.
Existing `ptsr` files and XDG paths stay unchanged. No dependency is added.
The owner approved this separate change in the existing PR on `86-screen-registry`.
The specification cleanup remains a separate commit.

## Out of scope

Do not delete folders on disk, change saved formats, remove other compatibility rules or change sound.
Keep historical OpenSpec archives unchanged. Do not activate change 87.

## How to verify

- Automated: Check current settings, keys and session loading with conflicting legacy files present.
- Automated: Check that legacy files alone cause no copy and that absent current files use existing defaults.
- Keyboard: Start normally. Check saved settings in F12 and the existing pad session.
- Controller: Check an existing learned control after startup. No new controller action is added.
- Listening: No new render is needed because sound processing is unchanged. Existing hardware and listening results remain unverified.
