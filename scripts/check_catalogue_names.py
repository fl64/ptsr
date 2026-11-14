#!/usr/bin/env python3
"""Check proposal catalogue names against fixture names defined in Rust code."""

import argparse
import fnmatch
import re
import sys
from pathlib import Path


MODE_NAME = re.compile(r'Self::(\w+)\s*=>\s*"([a-z0-9-]+)"')
SCREEN_INFO = re.compile(
    r'Self::(\w+)\s*=>\s*ScreenInfo\s*\{.*?key:\s*"([a-z0-9-]+)"'
    r'.*?screen_learn:\s*(true|false)', re.DOTALL,
)
APPLIES = re.compile(
    r'((?:Self::\w+\s*(?:\|\s*)?)+)\s*=>\s*'
    r'(screen == Screen::\w+|screen\.rules\(\)\.screen_learn|'
    r'matches!\(screen,[^)]+\))', re.DOTALL,
)
QUOTED = re.compile(r'`([^`]+)`')
SCREEN_PATH = re.compile(r'renders/screens/[a-z0-9*_.-]+')


def between(text: str, start: str, end: str) -> str:
    try:
        return text.split(start, 1)[1].split(end, 1)[0]
    except IndexError as error:
        raise ValueError(f"Catalogue source does not contain {start!r} and {end!r}") from error


def fixture_names(root: Path) -> tuple[set[str], set[str], set[str]]:
    catalogue = (root / "crates/app/src/screens.rs").read_text(encoding="utf-8")
    screens = (root / "crates/input/src/lib.rs").read_text(encoding="utf-8")
    mode_all = set(re.findall(
        r'Self::(\w+)', between(catalogue, "const ALL: &'static [Self]", "fn name(self)"),
    ))
    names = dict(MODE_NAME.findall(between(catalogue, "fn name(self)", "fn applies(")))
    if not names or set(names) != mode_all or len(set(names.values())) != len(names):
        raise ValueError("Catalogue mode names differ from Mode::ALL")

    screen_all = set(re.findall(
        r'Self::(\w+)', between(screens, "pub const ALL: [Self;", "fn info(self)"),
    ))
    info = SCREEN_INFO.findall(between(screens, "fn info(self)", "/// The screen code"))
    screen_keys = {variant: key for variant, key, _ in info}
    learn = {variant for variant, _, flag in info if flag == "true"}
    if not screen_keys or set(screen_keys) != screen_all:
        raise ValueError("Screen keys differ from Screen::ALL")

    applies = between(catalogue, "fn applies(self, screen: Screen)", "#[derive(Clone, Copy, Debug)]")
    if "_ => true" not in applies:
        raise ValueError("Catalogue mode default applicability changed")
    restricted: dict[str, set[str]] = {}
    for match in APPLIES.finditer(applies):
        modes = re.findall(r'Self::(\w+)', match[1])
        expression = match[2]
        if expression.startswith("screen == Screen::"):
            allowed = {expression.split("::", 1)[1]}
        elif expression == "screen.rules().screen_learn":
            allowed = learn
        else:
            allowed = set(re.findall(r'Screen::(\w+)', expression))
        for mode in modes:
            if mode in restricted or mode not in names:
                raise ValueError(f"Unknown or repeated catalogue mode rule: {mode}")
            restricted[mode] = allowed
    if not restricted or not all(allowed <= screen_all for allowed in restricted.values()):
        raise ValueError("Catalogue applicability could not be read")

    size_match = re.search(r"const SIZES:.*?=\s*\[(.*?)\];", catalogue, re.DOTALL)
    if size_match is None:
        raise ValueError("Catalogue sizes could not be read")
    size_source = size_match[1]
    sizes = [(int(w), int(h)) for w, h in re.findall(r'\((\d+),\s*(\d+)\)', size_source)]
    if len(sizes) != 2 or len(set(sizes)) != 2:
        raise ValueError("Catalogue must define two distinct sizes")
    stems = {
        f"{key}-{name}"
        for variant, key in screen_keys.items()
        for mode, name in names.items()
        if variant in restricted.get(mode, screen_all)
    }
    full = {f"{stem}-{width}x{height}.ansi" for stem in stems for width, height in sizes}
    return stems, full, set(screen_keys.values())


def named_entries(proposal: str, screen_keys: set[str]) -> set[str]:
    tokens = QUOTED.findall(proposal) + SCREEN_PATH.findall(proposal)
    entries = set()
    for token in tokens:
        name = token.removeprefix("renders/screens/").strip()
        if any(name.startswith(f"{screen}-") for screen in screen_keys):
            entries.add(name)
    return entries


def check(root: Path) -> tuple[list[str], int]:
    stems, full, screen_keys = fixture_names(root)
    errors = []
    count = 0
    changes = root / "openspec/changes"
    for folder in sorted(path for path in changes.iterdir() if path.is_dir()
                         and path.name != "archive"):
        proposal = folder / "proposal.md"
        if not proposal.is_file():
            continue
        for name in sorted(named_entries(proposal.read_text(encoding="utf-8"), screen_keys)):
            count += 1
            exists = (
                bool(fnmatch.filter(full, name)) if "*" in name else
                name in full if name.endswith(".ansi") else name in stems
            )
            if not exists:
                errors.append(f"{proposal.relative_to(root)}: no catalogue fixture for {name}")
    return errors, count


def cli() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        errors, count = check(args.root.resolve())
    except (OSError, ValueError) as error:
        print(f"Catalogue names: {error}", file=sys.stderr)
        return 1
    for error in errors:
        print(f"Catalogue names: {error}", file=sys.stderr)
    print(f"Catalogue names: checked {count} proposal entries; {len(errors)} errors")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(cli())
