# Proposal

## Why

The owner no longer uses `arasaka-404` settings and requested removal of their automatic migration.
Startup still copies these obsolete folders.

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
Existing `ptsr` files and XDG paths stay unchanged. This change adds no dependency.
The owner approved this separate change in the existing PR on `86-screen-registry`.
The specification cleanup remains a separate commit.

## Out of scope

Do not remove folders on disk, change saved formats, remove other compatibility rules or change sound.
Keep historical OpenSpec archives unchanged. Do not activate change 87.

## How to verify

- Automated: Verify current settings, keys and session loading with conflicting legacy files present.
- Automated: Verify that legacy files alone cause no copy and that absent current files use existing defaults.
- Keyboard: Start normally. Verify saved settings in F12 and the existing pad session.
- Controller: Verify an existing learned control after startup. This change adds no controller action.
- Listening: No new render: this change keeps sound processing unchanged. Existing hardware and listening results remain unverified.
