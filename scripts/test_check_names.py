import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import check_names

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/check_names.py"
TITLE = "fix(pads): keep original gate releases [85]"
SIGNOFF = "Signed-off-by: ptsr tests <ptsr-tests@example.invalid>"


class TitleTests(unittest.TestCase):
    def test_known_types_and_scopes_pass(self) -> None:
        for kind in check_names.TYPES:
            for scope in check_names.SCOPES:
                with self.subTest(kind=kind, scope=scope):
                    check_names._check_title(f"{kind}({scope}): keep the value [86a]")

    def test_numbers_ranges_and_repository_identifiers_pass(self) -> None:
        for identifier in ("09", "85", "86a", "83a-85", "85–86a", "86-86a",
                           "repo-format", "repo-v1-plan", "repo-license-policy"):
            with self.subTest(identifier=identifier):
                check_names._check_title(f"docs(build): state the rule [{identifier}]")

    def test_invalid_titles_fail(self) -> None:
        for title in (
            "", "85: pads — keep releases", "fix: keep releases [85]",
            "fix(pads): keep releases", "fix(pads):  [85]",
            "fix(pads): Keep releases [85]", "fix(pads): keep releases [v1]",
            "fix(pads): keep releases [license-policy]",
            "fix(pads): keep releases [repo-]", "fix(pads): keep releases [repo-A]",
            "fix(pads): keep releases [85-83a]", "fix(pads): keep releases [85-85]",
            "fix(pads): keep releases [86a-86]", "fix(pads): keep releases [85] ",
            " fix(pads): keep releases [85]", "fix(pads): keep releases [85]\n",
            "fix(pads): keep\treleases [85]", "fix(pads): keep\x00releases [85]",
            "fix(pads): keep\x1breleases [85]", "fix(pads): keep\x7freleases [85]",
            "fix(pads): keep\u2028releases [85]", "fix(pads): keep releases [٨٥]",
            "fix(pads): keep releases [85] (#8)", "fixup! " + TITLE,
            "squash! " + TITLE, "Merge branch 'master'", "fix(pads)!: break it [85]",
        ):
            with self.subTest(title=title), self.assertRaises(ValueError):
                check_names._check_title(title)

    def test_unknown_types_and_scope_aliases_fail(self) -> None:
        for kind, scope in (("change", "pads"), ("fix", "keyboard"), ("fix", "ui"),
                            ("test", "tests"), ("fix", "controls"), ("fix", "keymap")):
            with self.subTest(kind=kind, scope=scope), self.assertRaises(ValueError):
                check_names._check_title(f"{kind}({scope}): keep the value [85]")

    def test_only_final_commits_allow_the_github_suffix(self) -> None:
        for suffix in ("", " (#8)", " (#123)"):
            check_names._check_title(TITLE + suffix, final_commit=True)
        for suffix in (" (#0)", " (#01)", " (#x)", " (#8) extra", " (#8) (#9)"):
            with self.subTest(suffix=suffix), self.assertRaises(ValueError):
                check_names._check_title(TITLE + suffix, final_commit=True)

    def test_scope_table_matches_the_checker(self) -> None:
        agents = (ROOT / "AGENTS.md").read_text()
        table = agents.split("| Scope | Area |", 1)[1].split("\n## ", 1)[0]
        scopes = [line.split("`", 2)[1] for line in table.splitlines()
                  if line.startswith("| `")]
        self.assertEqual(list(check_names.SCOPES), scopes)


class MessageTests(unittest.TestCase):
    def test_signoff_trailers_pass(self) -> None:
        check_names._check_message(f"{TITLE}\n\n{SIGNOFF}\n")
        check_names._check_message(f"{TITLE}\n\nExplain the change.\n\n{SIGNOFF}\n")
        check_names._check_message(f"{TITLE} (#8)\n\n{SIGNOFF}\n", final_commit=True)

    def test_missing_or_fake_signoffs_fail(self) -> None:
        for body in ("", "\n\nSigned-off-by: missing-email", "\n\n" + SIGNOFF +
                     "\n\nThis is body text, not a trailer.",
                     "\n\nSigned-off-by: tests <invalid>"):
            with self.subTest(body=body), self.assertRaises(ValueError):
                check_names._check_message(TITLE + body)


class GitTests(unittest.TestCase):
    def setUp(self) -> None:
        target = ROOT / "target"
        target.mkdir(exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix="names-", dir=target)
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.env = dict(os.environ, GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL="/dev/null",
                        GIT_TERMINAL_PROMPT="0")
        for key in ("GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY",
                    "GIT_ALTERNATE_OBJECT_DIRECTORIES"):
            self.env.pop(key, None)
        self._git("init", "--quiet", "--initial-branch=master")
        self._git("config", "user.name", "ptsr tests")
        self._git("config", "user.email", "ptsr-tests@example.invalid")
        self._git("config", "commit.gpgsign", "false")
        self._git("commit", "--quiet", "--allow-empty", "-m", "historical title")
        self.base = self._git("rev-parse", "HEAD").stdout.strip()

    def _run(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(args, cwd=self.repo, env=self.env, text=True,
                              capture_output=True, check=False)

    def _git(self, *args: str) -> subprocess.CompletedProcess[str]:
        result = self._run("git", *args)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result

    def _check(self, *args: str) -> subprocess.CompletedProcess[str]:
        return self._run(sys.executable, "-B", str(SCRIPT), *args)

    def test_only_new_commits_are_checked(self) -> None:
        self._git("commit", "--quiet", "--allow-empty", "--signoff", "-m", TITLE)
        result = self._check("--range", f"{self.base}..HEAD")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Names: 1 commits checked.", result.stdout)

    def test_empty_range_passes(self) -> None:
        result = self._check("--range", "HEAD..HEAD")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Names: 0 commits checked.", result.stdout)

    def test_each_invalid_new_commit_is_reported(self) -> None:
        self._git("commit", "--quiet", "--allow-empty", "--signoff", "-m", "old style")
        self._git("commit", "--quiet", "--allow-empty", "-m", TITLE)
        result = self._check("--range", f"{self.base}..HEAD")
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr.count("Commit "), 2)
        self.assertIn("Use <type>(<scope>)", result.stderr)
        self.assertIn("Signed-off-by", result.stderr)

    def test_merge_commits_are_not_silently_skipped(self) -> None:
        self._git("switch", "--quiet", "-c", "other")
        self._git("commit", "--quiet", "--allow-empty", "--signoff", "-m",
                  "fix(pads): keep other gate releases [85]")
        self._git("switch", "--quiet", "master")
        self._git("commit", "--quiet", "--allow-empty", "--signoff", "-m", TITLE)
        self._git("merge", "--quiet", "--no-ff", "other", "-m", "Merge other")
        result = self._check("--range", f"{self.base}..HEAD")
        self.assertEqual(result.returncode, 1)
        self.assertIn("Use <type>(<scope>)", result.stderr)

    def test_invalid_or_missing_refs_fail(self) -> None:
        for value in ("HEAD", "HEAD...HEAD", "..HEAD", "HEAD..", "missing..HEAD",
                      "--all..HEAD", "HEAD..--all"):
            with self.subTest(value=value):
                result = self._check("--range", value)
                self.assertNotEqual(result.returncode, 0)

    def test_missing_or_invalid_utf8_message_files_fail(self) -> None:
        for name in ("missing-message", "invalid-message"):
            with self.subTest(name=name):
                if name == "invalid-message":
                    (self.repo / name).write_bytes(b"\xff")
                result = self._check("--message-file", name)
                self.assertEqual(result.returncode, 1)
                self.assertIn("Names: ERROR:", result.stderr)

    def test_title_and_final_commit_cli_modes(self) -> None:
        result = self._check("--title", TITLE)
        self.assertEqual(result.returncode, 0, result.stderr)
        message = self.repo / "final-message"
        message.write_text(f"{TITLE} (#8)\n\n{SIGNOFF}\n")
        result = self._check("--final-message-file", str(message))
        self.assertEqual(result.returncode, 0, result.stderr)
        result = self._check("--message-file", str(message))
        self.assertEqual(result.returncode, 1)

    def test_commit_hook_rejects_invalid_messages_and_accepts_signoff(self) -> None:
        for folder in ("scripts", ".githooks"):
            (self.repo / folder).mkdir()
        shutil.copyfile(SCRIPT, self.repo / "scripts/check_names.py")
        hook = self.repo / ".githooks/commit-msg"
        shutil.copyfile(ROOT / ".githooks/commit-msg", hook)
        hook.chmod(0o755)
        self._git("config", "core.hooksPath", ".githooks")
        for args in (("--signoff", "-m", "old style"), ("-m", TITLE)):
            with self.subTest(args=args):
                result = self._run("git", "commit", "--quiet", "--allow-empty", *args)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self._git("rev-parse", "HEAD").stdout.strip(), self.base)
        self._git("commit", "--quiet", "--allow-empty", "--signoff", "-m", TITLE)


if __name__ == "__main__":
    unittest.main()
