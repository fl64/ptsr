"""Compare callback timings with the PR base on one CI runner."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import tempfile
from pathlib import Path


REPORTS = ("steady playback", "dense pattern playback", "active recording", "active erasure")


def wall_p99(text: str, name: str) -> float:
    match = re.search(
        rf"paired timing: {re.escape(name)}\n  wall: .*?p99 ([0-9.]+) %",
        text,
    )
    if match is None:
        raise ValueError(f"No wall p99 for {name}")
    return float(match.group(1))


def metrics(text: str) -> dict[str, float]:
    result = {name: wall_p99(text, name) for name in REPORTS}
    switch = re.search(r"project switch: .*?switch peak p99 ([0-9.]+) %", text)
    dropped = re.search(r"dense pattern: .*?dropped hits ([0-9]+)", text)
    if switch is None or dropped is None:
        raise ValueError("The callback report has no switch or pattern result")
    result["switch peak"] = float(switch.group(1))
    result["dropped hits"] = float(dropped.group(1))
    return result


def changed_audio(base: str, head: str) -> bool:
    diff = subprocess.run(
        ["git", "diff", "--name-only", f"{base}..{head}"],
        check=True,
        capture_output=True,
        text=True,
    )
    return any(
        path.startswith("crates/engine/") or path == "crates/app/src/audio.rs"
        for path in diff.stdout.splitlines()
    )


def run_bench(directory: Path, target: Path, report_only: bool) -> str:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(target)
    env.pop("PTSR_BENCH_QUICK", None)
    env.pop("PTSR_BENCH_REPORT_ONLY", None)
    if report_only:
        env["PTSR_BENCH_REPORT_ONLY"] = "1"
    result = subprocess.run(
        ["cargo", "bench", "-p", "engine"],
        cwd=directory,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    if result.returncode and report_only:
        raise RuntimeError(f"The head benchmark failed:\n{result.stdout[-3000:]}")
    return result.stdout


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--label", default="false")
    args = parser.parse_args()
    if args.label != "true" and not changed_audio(args.base, args.head):
        print("No audio callback change or bench label; benchmark not required.")
        return

    root = Path.cwd()
    target = root / "target" / "ci-bench"
    with tempfile.TemporaryDirectory(prefix="ptsr-bench-") as temporary:
        base_tree = Path(temporary) / "base"
        subprocess.run(["git", "worktree", "add", "--detach", str(base_tree), args.base], check=True)
        try:
            base = metrics(run_bench(base_tree, target, report_only=False))
        finally:
            subprocess.run(["git", "worktree", "remove", "--force", str(base_tree)], check=True)
        head = metrics(run_bench(root, target, report_only=True))

    lines = ["### Callback benchmark on this runner", "", "| Measure | Base | Head | Change |", "| --- | ---: | ---: | ---: |"]
    for name in REPORTS + ("switch peak",):
        delta = head[name] - base[name]
        lines.append(f"| {name} wall p99 | {base[name]:.2f}% | {head[name]:.2f}% | {delta:+.2f} points |")
    lines.append(f"| Dropped pattern hits | {base['dropped hits']:.0f} | {head['dropped hits']:.0f} | n/a |")
    report = "\n".join(lines) + "\n"
    print(report)
    if summary := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(summary, "a", encoding="utf-8") as output:
            output.write(report)
    if head["dropped hits"]:
        raise RuntimeError("The head benchmark dropped pattern hits")


if __name__ == "__main__":
    main()
