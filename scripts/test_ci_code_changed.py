import subprocess
import tempfile
import unittest
from pathlib import Path

import ci_code_changed
from ci_code_changed import needs_rust


class NeedsRustTest(unittest.TestCase):
    def test_text_only_paths_skip_rust(self):
        self.assertFalse(needs_rust(["docs/plans/PATTERNS.md", "README.md", "AGENTS.md"]))

    def test_code_and_test_inputs_need_rust(self):
        for path in [
            "crates/app/src/main.rs",
            "config/keymap.toml",
            "tests/midi/smc-profile.txt",
            "openspec/specs/patterns/spec.md",
            "openspec/untested.md",
            "docs/USER-STORIES.md",
            "docs/CONTROLLER-PATHS.md",
            "Cargo.lock",
            "Taskfile.yml",
            "deny.toml",
            "scripts/test_changed.py",
            ".github/workflows/rust.yml",
            "crates/app/README.md",
        ]:
            with self.subTest(path=path):
                self.assertTrue(needs_rust(["docs/VISION.md", path]))

    def test_empty_diff_needs_rust(self):
        self.assertTrue(needs_rust([]))

    def test_deleted_code_needs_rust(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)

            def git(*args: str) -> str:
                return subprocess.run(
                    ["git", *args], cwd=root, capture_output=True, text=True, check=True,
                ).stdout.strip()

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
            self.assertFalse(needs_rust(ci_code_changed.changed_paths(root, base, text_head)))
            git("rm", "-q", "crates/app/src/lib.rs")
            git("commit", "-qm", "delete code")
            paths = ci_code_changed.changed_paths(root, base, git("rev-parse", "HEAD"))
            self.assertTrue(needs_rust(paths))


if __name__ == "__main__":
    unittest.main()
