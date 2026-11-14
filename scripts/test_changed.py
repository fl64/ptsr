"""Run tests for paths changed against the base branch."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


APP_INPUT = {
    "audio": ("input_tests::",),
    "chop": ("input_tests::chop_",),
    "config": ("input_tests::saving_projects::", "input_tests::transport::"),
    "controller_map": ("input_tests::controller_map::",),
    "dispatch": ("input_tests::",),
    "help": ("input_tests::notices_help::",),
    "input_sources": ("input_tests::",),
    "learn": ("input_tests::registry::",),
    "loader": ("input_tests::badges::",),
    "midi": ("input_tests::midi_state::", "input_tests::registry::"),
    "notices": ("input_tests::notices_help::",),
    "pads": ("input_tests::feedback::", "input_tests::bank_scenarios::"),
    "pattern_record": ("input_tests::patterns::",),
    "pattern_view": ("input_tests::pattern_view::", "input_tests::pattern_step::"),
    "saving": ("input_tests::saving_projects::",),
    "session": ("input_tests::patterns::",),
    "transport": ("input_tests::transport::", "input_tests::patterns::"),
}
DRAWING = {"graphics", "meters", "screens", "view"}


def plan(paths: set[str]) -> list[list[str]]:
    """Return commands for changed paths. Unknown paths use the fast set."""
    packages: set[str] = set()
    app_filters: set[str] = set()
    integration: set[str] = set()
    app_full = False
    screens = False
    scenarios = False
    fallback = not paths
    for path in paths:
        if path.startswith("crates/engine/"):
            packages.add("engine")  # Includes the offline render checks.
        elif path.startswith("crates/tui/"):
            packages.add("tui")
            screens = True
        elif path.startswith("crates/input/"):
            packages.add("input")
            app_filters.add("input_tests::")  # Keyboard and raw MIDI routes.
        elif path.startswith("crates/app/src/input_tests/") and path.endswith(".rs"):
            app_filters.add(f"input_tests::{Path(path).stem}::")
        elif path == "crates/app/src/input_tests.rs":
            app_filters.add("input_tests::")
        elif path == "crates/app/src/main.rs":
            app_full = True
        elif path == "crates/app/src/session/pattern_tests.rs":
            app_filters.add("session::pattern_tests::")
        elif path.startswith("crates/app/src/") and path.endswith(".rs"):
            stem = Path(path).stem
            if stem in DRAWING:
                screens = True
            elif stem in {"saving_tests", "spec_sync_tests"}:
                app_filters.add(f"{stem}::")
            elif stem in APP_INPUT:
                app_filters.add(f"{stem}::tests::")
                app_filters.update(APP_INPUT[stem])
            else:
                fallback = True
        elif path.startswith("crates/app/tests/") and path.endswith(".rs"):
            integration.add(Path(path).stem)
        elif path.startswith("openspec/"):
            scenarios = True
        else:
            fallback = True

    if fallback:
        return [["task", "test:fast"]]
    commands = [
        ["cargo", "nextest", "run", "-p", package] for package in sorted(packages)
    ]
    if app_full:
        commands.append(["cargo", "nextest", "run", "-p", "ptsr"])
    elif app_filters:
        expression = " + ".join(f"test({name})" for name in sorted(app_filters))
        commands.append(["cargo", "nextest", "run", "-p", "ptsr", "-E", expression])
    for binary in sorted(integration):
        commands.append(["cargo", "nextest", "run", "-p", "ptsr", "-E", f"binary({binary})"])
    if scenarios:
        commands.append(["cargo", "nextest", "run", "-p", "ptsr", "-E", "binary(scenarios)"])
    if screens:
        commands.append(["task", "screens"])
    return commands


def changed_paths(base: str) -> set[str]:
    diff = subprocess.run(
        ["git", "diff", "--name-only", "--diff-filter=ACMR", base],
        check=True,
        capture_output=True,
        text=True,
    )
    untracked = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard"],
        check=True,
        capture_output=True,
        text=True,
    )
    return set(diff.stdout.splitlines() + untracked.stdout.splitlines())


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="origin/master")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    commands = plan(changed_paths(args.base))
    for command in commands:
        print(" ".join(command), flush=True)
        if not args.dry_run:
            subprocess.run(command, check=True)


if __name__ == "__main__":
    main()
