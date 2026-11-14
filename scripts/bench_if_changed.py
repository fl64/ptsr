"""Run the full callback benchmark when audio callback code changed."""

from __future__ import annotations

import argparse
import subprocess


def needs_bench(paths: set[str]) -> bool:
    return any(
        path.startswith("crates/engine/") or path == "crates/app/src/audio.rs"
        for path in paths
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="origin/master")
    args = parser.parse_args()
    result = subprocess.run(
        ["git", "diff", "--name-only", "--diff-filter=ACMR", args.base],
        check=True,
        capture_output=True,
        text=True,
    )
    paths = set(result.stdout.splitlines())
    if needs_bench(paths):
        subprocess.run(["task", "bench"], check=True)
    else:
        print("No audio callback changes; benchmark not required.")


if __name__ == "__main__":
    main()
