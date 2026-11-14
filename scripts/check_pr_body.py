#!/usr/bin/env python3
"""Check required pull request body sections and code review evidence."""

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


COMMENT = re.compile(r"<!--.*?-->", re.DOTALL)
HEADING = re.compile(r"^(#{2,3})\s+(.+?)\s*$", re.MULTILINE)
LIST_MARK = re.compile(r"(?m)^\s*(?:[-*]|\d+\.)\s*(?:\[[ xX]\])?\s*")
RENDER_PATH = re.compile(r"\brenders/[A-Za-z0-9_./*?-]+")


def sections(markdown: str) -> list[tuple[str, str]]:
    clean = COMMENT.sub("", markdown)
    heads = list(HEADING.finditer(clean))
    return [
        (match[2].strip(), clean[match.end():heads[i + 1].start() if i + 1 < len(heads)
                                else len(clean)].strip())
        for i, match in enumerate(heads)
    ]


def plain_content(text: str) -> str:
    return LIST_MARK.sub("", text).strip()


def render_reason(text: str) -> bool:
    plain = plain_content(text).replace("\n", " ").strip()
    if RENDER_PATH.search(plain):
        return True
    words = re.findall(r"[A-Za-z]+", plain)
    return (
        len(words) >= 6
        and bool(re.search(r"\b(no|none|without)\b", plain, re.IGNORECASE))
        and plain.endswith((".", "!", "?"))
        and len(re.findall(r"[.!?](?:\s|$)", plain)) == 1
    )


def check(template: str, body: str, code_changed: bool) -> list[str]:
    expected = [name for name, _ in sections(template)]
    actual = sections(body)
    by_name: dict[str, list[str]] = {}
    for name, content in actual:
        by_name.setdefault(name.casefold(), []).append(content)
    errors = []
    if not expected:
        return ["PR template has no sections"]
    for name in expected:
        values = by_name.get(name.casefold(), [])
        if not values:
            errors.append(f"missing section: {name}")
        elif len(values) > 1:
            errors.append(f"section appears more than once: {name}")
        elif not re.search(r"[A-Za-z0-9]", plain_content(values[0])):
            errors.append(f"empty section: {name}")
    if code_changed:
        for name in ("How to check by hand", "Renders"):
            if name.casefold() not in by_name:
                errors.append(f"code PR needs section: {name}")
        renders = by_name.get("renders", [])
        if len(renders) == 1 and plain_content(renders[0]) and not render_reason(renders[0]):
            errors.append("Renders needs a render path or one sentence that explains no render")
    return errors


def code_changed(root: Path, base: str, head: str) -> bool:
    result = subprocess.run(
        ["git", "-C", str(root), "diff", "--name-only", "--diff-filter=ACMR",
         base, head, "--", "crates/"],
        capture_output=True, text=True, encoding="utf-8", check=False,
    )
    if result.returncode:
        raise ValueError(result.stderr.strip())
    return bool(result.stdout.strip())


def cli() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--body-file", type=Path, help="Read body from a file, not PR_BODY")
    parser.add_argument("--base", help="PR base commit for code detection")
    parser.add_argument("--head", help="PR head commit for code detection")
    args = parser.parse_args()
    root = args.root.resolve()
    if bool(args.base) != bool(args.head):
        parser.error("--base and --head must be used together")
    try:
        template = (root / ".github/PULL_REQUEST_TEMPLATE.md").read_text(encoding="utf-8")
        body = (args.body_file.read_text(encoding="utf-8") if args.body_file
                else os.environ.get("PR_BODY", ""))
        errors = check(template, body, code_changed(root, args.base, args.head)
                       if args.base else False)
    except (OSError, ValueError) as error:
        print(f"PR body: {error}", file=sys.stderr)
        return 1
    for error in errors:
        print(f"PR body: {error}", file=sys.stderr)
    print(f"PR body: {len(sections(template))} required sections; {len(errors)} errors")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(cli())
