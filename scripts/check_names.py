#!/usr/bin/env python3
import argparse
import re
import subprocess
import sys
from pathlib import Path

TYPES = (
    "feat", "fix", "docs", "refactor", "perf", "test", "build", "ci", "chore", "revert",
)
SCOPES = (
    "pads", "chop", "files", "settings", "midi", "input", "tui", "engine",
    "specs", "docs", "build",
)
NUMBER = r"[0-9]+[a-z]?"
CHANGE = rf"(?:{NUMBER}(?:[-–]{NUMBER})?|repo-[a-z0-9]+(?:-[a-z0-9]+)*)"
HEADER = (
    r"(?P<type>[a-z]+)\((?P<scope>[a-z][a-z0-9-]*)\): "
    rf"(?P<summary>[a-z0-9](?:.*\S)?) \[(?P<change>{CHANGE})\]"
)
SIGNOFF = re.compile(r"Signed-off-by: [^<>\n]+ <[^<>\s@]+@[^<>\s@]+>")


def _git(*args: str, input_text: str | None = None) -> str:
    result = subprocess.run(
        ["git", *args], input=input_text, text=True, encoding="utf-8",
        capture_output=True, check=False,
    )
    if result.returncode:
        raise ValueError(f"Git failed: {result.stderr.strip()}")
    return result.stdout


def _number_key(value: str) -> tuple[int, str]:
    match = re.fullmatch(r"([0-9]+)([a-z]?)", value)
    assert match is not None
    return int(match[1]), match[2]


def _check_title(title: str, *, final_commit: bool = False) -> None:
    if any(ord(char) < 32 or ord(char) == 127 or (char.isspace() and char != " ")
           for char in title):
        raise ValueError("Keep the title on one line. Do not use control characters.")
    suffix = r"(?: \(#[1-9][0-9]*\))?" if final_commit else ""
    match = re.fullmatch(HEADER + suffix, title)
    if match is None:
        raise ValueError("Use <type>(<scope>): <summary> [<change>]. "
                         "Start the summary with a lowercase ASCII letter or digit.")
    if match["type"] not in TYPES:
        raise ValueError(f"Use one of these types: {', '.join(TYPES)}.")
    if match["scope"] not in SCOPES:
        raise ValueError(f"Use one of these scopes: {', '.join(SCOPES)}.")
    change = match["change"]
    if not change.startswith("repo-"):
        ends = re.split("[-–]", change)
        if len(ends) == 2 and _number_key(ends[0]) >= _number_key(ends[1]):
            raise ValueError("Use an ascending change range.")


def _check_message(message: str, *, final_commit: bool = False) -> None:
    _check_title(message.partition("\n")[0], final_commit=final_commit)
    trailers = _git("interpret-trailers", "--parse", input_text=message)
    if not any(SIGNOFF.fullmatch(line) for line in trailers.splitlines()):
        raise ValueError("Add a Signed-off-by trailer with git commit --signoff.")


def _check_range(revision_range: str) -> int:
    parts = revision_range.split("..")
    if len(parts) != 2 or not all(parts):
        raise ValueError("Use a two-dot range, such as origin/master..HEAD.")
    base, head = (
        _git("rev-parse", "--verify", "--end-of-options", f"{ref}^{{commit}}").strip()
        for ref in parts
    )
    commits = _git("rev-list", "--reverse", f"{base}..{head}").splitlines()
    errors = []
    for commit in commits:
        try:
            _check_message(_git("show", "--no-patch", "--format=%B", commit))
        except ValueError as error:
            errors.append(f"Commit {commit[:12]}: {error}")
    if errors:
        raise ValueError("\n".join(errors))
    return len(commits)


def _main() -> int:
    parser = argparse.ArgumentParser(
        description="Check ptsr commit messages and pull request titles.",
    )
    modes = parser.add_mutually_exclusive_group(required=True)
    modes.add_argument("--title", help="Check a pull request title.")
    modes.add_argument("--message-file", type=Path, help="Check a local commit message.")
    modes.add_argument("--range", dest="revision_range", help="Check new branch commits.")
    modes.add_argument("--final-message-file", type=Path,
                       help="Check a final GitHub commit, with its optional PR suffix.")
    args = parser.parse_args()
    try:
        if args.title is not None:
            _check_title(args.title)
            print("Names: title passed.")
        elif args.revision_range is not None:
            count = _check_range(args.revision_range)
            print(f"Names: {count} commits checked.")
        else:
            path = args.message_file or args.final_message_file
            _check_message(path.read_text(encoding="utf-8"),
                           final_commit=args.final_message_file is not None)
            print("Names: commit message passed.")
    except (OSError, UnicodeError, ValueError) as error:
        print(f"Names: ERROR: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(_main())
