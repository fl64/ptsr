#!/usr/bin/env python3
"""Check archived scenario history and warn about merged active changes."""

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path


NO_BEHAVIOR_SENTENCE = "This change includes no behavior change."
SECTION = re.compile(r"^## (ADDED|MODIFIED|REMOVED|RENAMED) (Requirements|Scenarios)$")
REQUIREMENT = re.compile(r"^### Requirement: (.+)$")
SCENARIO = re.compile(r"^#### Scenario: (.+)$", re.MULTILINE)
RENAME = re.compile(r"^- (FROM|TO): `(?:### Requirement: |#### Scenario: )?(.+?)`$")
CHANGE_ID = re.compile(r"^([0-9]+[a-z]?)(?:-|$)")
TITLE_ID = re.compile(r"\[([0-9]+[a-z]?(?:[-–][0-9]+[a-z]?)?)\]")


@dataclass(frozen=True)
class Event:
    kind: str
    requirement: str = ""
    scenario: str = ""
    target: str = ""


@dataclass(frozen=True)
class MergedCommit:
    title: str
    at: datetime


def parse_delta(path: Path) -> list[Event]:
    events: list[Event] = []
    section = ""
    requirement = ""
    rename_from = ""
    for line in path.read_text(encoding="utf-8").splitlines():
        if match := SECTION.match(line):
            section = f"{match[1]} {match[2]}"
            requirement = ""
            rename_from = ""
        elif match := REQUIREMENT.match(line):
            requirement = match[1]
            if section == "REMOVED Requirements":
                events.append(Event("remove_requirement", requirement))
        elif match := SCENARIO.match(line):
            if section in ("ADDED Requirements", "MODIFIED Requirements"):
                events.append(Event("add_scenario", requirement, match[1]))
            elif section == "REMOVED Scenarios":
                events.append(Event("remove_scenario", requirement, match[1]))
        elif match := RENAME.match(line):
            if match[1] == "FROM":
                rename_from = match[2]
            elif rename_from and section == "RENAMED Requirements":
                events.append(Event("rename_requirement", rename_from, target=match[2]))
                rename_from = ""
            elif rename_from and section == "RENAMED Scenarios":
                events.append(Event("rename_scenario", requirement, rename_from, match[2]))
                rename_from = ""
    return events


def main_scenarios(root: Path) -> dict[str, set[str]]:
    specs = root / "openspec/specs"
    result: dict[str, set[str]] = {}
    for path in sorted(specs.rglob("spec.md")):
        capability = path.parent.relative_to(specs).as_posix()
        result[capability] = set(SCENARIO.findall(path.read_text(encoding="utf-8")))
    return result


def _id_key(value: str) -> tuple[int, str]:
    match = re.fullmatch(r"([0-9]+)([a-z]?)", value)
    assert match is not None
    return int(match[1]), match[2]


def title_has_id(title: str, identifier: str) -> bool:
    for value in TITLE_ID.findall(title):
        ends = re.split("[-–]", value)
        if len(ends) == 1 and ends[0] == identifier:
            return True
        if len(ends) == 2 and _id_key(ends[0]) <= _id_key(identifier) <= _id_key(ends[1]):
            return True
    return False


def check_archived(root: Path) -> tuple[list[str], int]:
    errors: list[str] = []
    # The key keeps requirement identity for later REMOVED and RENAMED entries.
    active: dict[tuple[str, str, str], str] = {}
    archive = root / "openspec/changes/archive"
    folders = sorted(path for path in archive.iterdir() if path.is_dir())
    for folder in folders:
        specs = folder / "specs"
        if not specs.is_dir():
            proposal = folder / "proposal.md"
            text = proposal.read_text(encoding="utf-8") if proposal.exists() else ""
            if NO_BEHAVIOR_SENTENCE not in text:
                errors.append(f"{folder}: no specs/ and no fixed no-behavior sentence")
            continue
        for path in sorted(specs.rglob("spec.md")):
            capability = path.parent.relative_to(specs).as_posix()
            for event in parse_delta(path):
                if event.kind == "add_scenario":
                    active[(capability, event.requirement, event.scenario)] = folder.name
                elif event.kind == "remove_requirement":
                    active = {key: origin for key, origin in active.items()
                              if key[:2] != (capability, event.requirement)}
                elif event.kind == "remove_scenario":
                    active = {key: origin for key, origin in active.items()
                              if not (key[0] == capability and key[2] == event.scenario
                                      and (not event.requirement or key[1] == event.requirement))}
                elif event.kind == "rename_requirement":
                    for key in list(active):
                        if key[:2] == (capability, event.requirement):
                            active[(capability, event.target, key[2])] = active.pop(key)
                elif event.kind == "rename_scenario":
                    for key in list(active):
                        if key[0] == capability and key[2] == event.scenario and (
                            not event.requirement or key[1] == event.requirement
                        ):
                            active[(capability, key[1], event.target)] = active.pop(key)

    current = main_scenarios(root)
    for (capability, requirement, scenario), origin in sorted(active.items()):
        if scenario not in current.get(capability, set()):
            errors.append(
                f"{origin}: {capability}/{requirement}: scenario '{scenario}' is absent "
                "from main specs"
            )

    return errors, len(active)


def check_unarchived(
    root: Path, merged: list[MergedCommit], now: datetime
) -> list[str]:
    warnings: list[str] = []
    changes = root / "openspec/changes"
    for folder in sorted(path for path in changes.iterdir() if path.is_dir()
                         and path.name != "archive"):
        match = CHANGE_ID.match(folder.name)
        if match is None:
            continue
        identifier = match[1]
        dates = [commit.at for commit in merged if title_has_id(commit.title, identifier)]
        if dates and now - min(dates) > timedelta(days=2):
            warnings.append(
                f"::warning file={folder.relative_to(root)}/proposal.md,"
                f"title=Unarchived merged change::Change {identifier} was merged "
                f"more than two days ago. Archive it after owner acceptance."
            )
    return warnings


def audit(
    root: Path, merged: list[MergedCommit], now: datetime
) -> tuple[list[str], list[str], int]:
    errors, count = check_archived(root)
    return errors, check_unarchived(root, merged, now), count


def git_history(root: Path, base: str) -> list[MergedCommit]:
    result = subprocess.run(
        ["git", "-C", str(root), "log", base, "--format=%cI%x00%s"],
        capture_output=True, text=True, encoding="utf-8", check=False,
    )
    if result.returncode:
        raise ValueError(result.stderr.strip())
    commits = []
    for line in result.stdout.splitlines():
        at, title = line.split("\0", 1)
        commits.append(MergedCommit(title, datetime.fromisoformat(at)))
    return commits


def default_base(root: Path) -> str:
    for ref in ("origin/master", "master"):
        result = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--verify", "--quiet", ref],
            capture_output=True, check=False,
        )
        if result.returncode == 0:
            return ref
    raise ValueError("No master ref is available. Fetch origin/master first.")


def cli() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--base", help="Merged master revision; default: origin/master")
    parser.add_argument("--only", choices=("sync", "unarchived"),
                        help="Run one check for a separate CI step")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        errors, count = check_archived(root) if args.only != "unarchived" else ([], 0)
        warnings = (
            check_unarchived(
                root, git_history(root, args.base or default_base(root)),
                datetime.now(timezone.utc),
            )
            if args.only != "sync" else []
        )
    except (OSError, ValueError) as error:
        print(f"Spec sync: {error}", file=sys.stderr)
        return 1
    for warning in warnings:
        print(warning)
    for error in errors:
        print(f"Spec sync: {error}", file=sys.stderr)
    if args.only != "unarchived":
        print(f"Spec sync: checked {count} current archived scenarios; {len(errors)} errors")
    else:
        print(f"Spec sync: checked merged changes; {len(warnings)} warnings")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(cli())
