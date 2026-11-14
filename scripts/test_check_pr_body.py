import subprocess
import tempfile
import unittest
from pathlib import Path

import check_pr_body


TEMPLATE = """## Description

<!-- Explain the change. -->

## Validation

## How to check by hand

## Renders

## Checklist

## Changelog entries
"""

GOOD_BODY = """## Description

The screen shows the selected pad.

## Validation

- `task check`: passed.

## How to check by hand

Open Pads and select a sound.

## Renders

No new render is needed because sound stays unchanged.

## Checklist

- [x] Specs and tests are current.

## Changelog entries

- Show the selected pad.
"""


class PrBodyTests(unittest.TestCase):
    def test_complete_code_body_passes_with_reason_for_no_render(self) -> None:
        self.assertEqual(check_pr_body.check(TEMPLATE, GOOD_BODY, True), [])

    def test_missing_and_empty_sections_fail(self) -> None:
        body = GOOD_BODY.replace("Open Pads and select a sound.", "<!-- no text -->")
        body = body.replace("- Show the selected pad.\n", "")
        errors = check_pr_body.check(TEMPLATE, body, True)
        self.assertIn("empty section: How to check by hand", errors)
        self.assertIn("empty section: Changelog entries", errors)
        errors = check_pr_body.check(TEMPLATE, body.replace("## Renders", "## Sounds"), True)
        self.assertIn("missing section: Renders", errors)

    def test_code_body_needs_render_or_one_sentence_reason(self) -> None:
        body = GOOD_BODY.replace(
            "No new render is needed because sound stays unchanged.", "No."
        )
        self.assertIn(
            "Renders needs a render path or one sentence that explains no render",
            check_pr_body.check(TEMPLATE, body, True),
        )
        body = GOOD_BODY.replace(
            "No new render is needed because sound stays unchanged.",
            "Listen to `renders/124-pattern.wav`.",
        )
        self.assertEqual(check_pr_body.check(TEMPLATE, body, True), [])

    def test_text_only_body_does_not_need_render_path(self) -> None:
        body = GOOD_BODY.replace(
            "No new render is needed because sound stays unchanged.",
            "The change updates only documentation.",
        )
        self.assertEqual(check_pr_body.check(TEMPLATE, body, False), [])

    def test_code_detection_uses_changed_crate_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def git(*args: str) -> str:
                result = subprocess.run(
                    ["git", *args], cwd=root, capture_output=True, text=True, check=True,
                )
                return result.stdout.strip()

            git("init", "-q")
            git("config", "user.name", "Test")
            git("config", "user.email", "test@example.com")
            (root / "docs").mkdir()
            (root / "docs/plan.md").write_text("first\n")
            (root / "crates/app/src").mkdir(parents=True)
            (root / "crates/app/src/lib.rs").write_text("pub fn sound() {}\n")
            git("add", ".")
            git("commit", "-qm", "first")
            base = git("rev-parse", "HEAD")
            (root / "docs/plan.md").write_text("second\n")
            git("commit", "-qam", "text")
            text_head = git("rev-parse", "HEAD")
            self.assertFalse(check_pr_body.code_changed(root, base, text_head))
            git("checkout", "-qb", "new-base", base)
            (root / "crates/app/src/lib.rs").write_text("pub fn sound() { println!(\"x\"); }\n")
            git("commit", "-qam", "base code")
            new_base = git("rev-parse", "HEAD")
            git("checkout", "-q", text_head)
            self.assertFalse(check_pr_body.code_changed(root, new_base, text_head))
            (root / "crates/app/src/lib.rs").write_text("pub fn sound() { println!(\"y\"); }\n")
            git("add", ".")
            git("commit", "-qm", "code")
            self.assertTrue(check_pr_body.code_changed(root, base, git("rev-parse", "HEAD")))


if __name__ == "__main__":
    unittest.main()
