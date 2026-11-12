"""Reject growth in the two large source files unless the PR has an owner exception."""

import argparse
import subprocess
from pathlib import Path


FILES = ("crates/app/src/main.rs", "crates/tui/src/lib.rs")


def lines(data: bytes) -> int:
    return len(data.splitlines())


def revision_file(revision: str, path: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{revision}:{path}"])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", default="WORKTREE")
    parser.add_argument("--allow-growth", action="store_true")
    args = parser.parse_args()

    failed = False
    for path in FILES:
        before = lines(revision_file(args.base, path))
        after_data = (
            Path(path).read_bytes()
            if args.head == "WORKTREE"
            else revision_file(args.head, path)
        )
        after = lines(after_data)
        change = after - before
        print(f"{path}: {before} -> {after} ({change:+d})")
        if change > 0 and not args.allow_growth:
            failed = True
    if failed:
        print("The two large files must not grow without an owner-approved PR exception.")
    return int(failed)


if __name__ == "__main__":
    raise SystemExit(main())
