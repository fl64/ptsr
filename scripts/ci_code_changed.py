#!/usr/bin/env python3
"""Tell CI if a pull request needs the Rust checks.

The Rust checks run unless every changed path is text that no build or test reads.
Rust tests read openspec/, config/, tests/ and two documents, so those paths still need the checks.
"""

import argparse
import os
import subprocess
import sys
from pathlib import Path

TEST_INPUT_DOCS = {"docs/USER-STORIES.md", "docs/CONTROLLER-PATHS.md"}


def text_only(path: str) -> bool:
    if path in TEST_INPUT_DOCS:
        return False
    if path.startswith("docs/"):
        return True
    return "/" not in path and path.endswith(".md")


def needs_rust(paths: list[str]) -> bool:
    return not paths or not all(text_only(path) for path in paths)


def changed_paths(root: Path, base: str, head: str) -> list[str]:
    result = subprocess.run(
        ["git", "-C", str(root), "diff", "--name-only", "--no-renames", f"{base}...{head}"],
        capture_output=True, text=True, encoding="utf-8", check=False,
    )
    if result.returncode:
        raise ValueError(result.stderr.strip())
    return [line for line in result.stdout.splitlines() if line]


def cli() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    args = parser.parse_args()
    try:
        paths = changed_paths(args.root.resolve(), args.base, args.head)
    except ValueError as error:
        print(f"Rust scope: {error}", file=sys.stderr)
        return 1
    rust = needs_rust(paths)
    code = [path for path in paths if not text_only(path)]
    print(f"Rust scope: {len(paths)} changed paths, {len(code)} need Rust checks; run = {rust}")
    for path in code[:20]:
        print(f"  {path}")
    output = os.environ.get("GITHUB_OUTPUT")
    if output:
        with open(output, "a", encoding="utf-8") as file:
            file.write(f"rust={'true' if rust else 'false'}\n")
    return 0


if __name__ == "__main__":
    sys.exit(cli())
