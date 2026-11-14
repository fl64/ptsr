import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path

import check_spec_sync


class SpecSyncTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "openspec/changes/archive").mkdir(parents=True)
        (self.root / "openspec/specs").mkdir(parents=True)
        self.now = datetime(2026, 11, 14, 12, tzinfo=timezone.utc)

    def write(self, relative: str, content: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def delta(self, folder: str, text: str, capability: str = "patterns") -> None:
        self.write(f"openspec/changes/archive/{folder}/specs/{capability}/spec.md", text)

    def main_spec(self, text: str, capability: str = "patterns") -> None:
        self.write(f"openspec/specs/{capability}/spec.md", text)

    def audit(self, merged: list[check_spec_sync.MergedCommit] | None = None):
        return check_spec_sync.audit(self.root, merged or [], self.now)

    def test_missing_archived_scenario_fails(self) -> None:
        self.delta("2026-11-01-01-play", """## ADDED Requirements
### Requirement: Play
#### Scenario: Missing hit
- **WHEN** a pad plays
- **THEN** it sounds
""")
        self.main_spec("# patterns Specification\n")
        errors, warnings, count = self.audit()
        self.assertEqual(count, 1)
        self.assertEqual(len(errors), 1)
        self.assertIn("Missing hit", errors[0])
        self.assertEqual(warnings, [])

    def test_later_removed_requirement_exempts_its_scenario(self) -> None:
        self.delta("2026-11-01-01-play", """## ADDED Requirements
### Requirement: Old playback
#### Scenario: Old hit
""")
        self.delta("2026-11-02-02-capture", """## REMOVED Requirements
### Requirement: Old playback
**Reason**: Capture replaces it.
""")
        self.main_spec("# patterns Specification\n")
        errors, _, count = self.audit()
        self.assertEqual((errors, count), ([], 0))

    def test_modified_requirement_cannot_silently_drop_old_scenario(self) -> None:
        self.delta("2026-11-01-01-play", """## ADDED Requirements
### Requirement: Play
#### Scenario: Keep this hit
""")
        self.delta("2026-11-02-02-edit", """## MODIFIED Requirements
### Requirement: Play
#### Scenario: New hit
""")
        self.main_spec("""### Requirement: Play
#### Scenario: New hit
""")
        errors, _, count = self.audit()
        self.assertEqual(count, 2)
        self.assertEqual(len(errors), 1)
        self.assertIn("Keep this hit", errors[0])

    def test_renamed_scenario_follows_its_new_name(self) -> None:
        self.delta("2026-11-01-01-play", """## ADDED Requirements
### Requirement: Play
#### Scenario: Old name
""")
        self.delta("2026-11-02-02-rename", """## RENAMED Scenarios
### Requirement: Play
- FROM: `#### Scenario: Old name`
- TO: `#### Scenario: New name`
""")
        self.main_spec("""### Requirement: Play
#### Scenario: New name
""")
        errors, _, count = self.audit()
        self.assertEqual((errors, count), ([], 1))

    def test_later_removed_scenario_exempts_only_that_scenario(self) -> None:
        self.delta("2026-11-01-01-play", """## ADDED Requirements
### Requirement: Play
#### Scenario: Old action
#### Scenario: Keep action
""")
        self.delta("2026-11-02-02-remove", """## REMOVED Scenarios
#### Scenario: Old action
**Reason**: A later action replaces it.
""")
        self.main_spec("""### Requirement: Play
#### Scenario: Keep action
""")
        errors, _, count = self.audit()
        self.assertEqual((errors, count), ([], 1))

    def test_renamed_scenario_requires_its_new_name_in_main_specs(self) -> None:
        self.delta("2026-11-01-01-play", """## ADDED Requirements
### Requirement: Play
#### Scenario: Old action
""")
        self.delta("2026-11-02-02-rename", """## RENAMED Scenarios
- FROM: `#### Scenario: Old action`
- TO: `#### Scenario: New action`
""")
        self.main_spec("# patterns Specification\n")
        errors, _, count = self.audit()
        self.assertEqual(count, 1)
        self.assertEqual(len(errors), 1)
        self.assertIn("New action", errors[0])

    def test_requirement_rename_keeps_scenario_until_removal(self) -> None:
        self.delta("2026-11-01-01-play", """## ADDED Requirements
### Requirement: Old play
#### Scenario: Play once
""")
        self.delta("2026-11-02-02-rename", """## RENAMED Requirements
- FROM: `### Requirement: Old play`
- TO: `### Requirement: New play`
""")
        self.delta("2026-11-03-03-remove", """## REMOVED Requirements
### Requirement: New play
**Reason**: The owner removed it.
""")
        self.main_spec("# patterns Specification\n")
        errors, _, count = self.audit()
        self.assertEqual((errors, count), ([], 0))

    def test_archive_without_delta_needs_fixed_sentence(self) -> None:
        self.write("openspec/changes/archive/2026-11-01-01-move/proposal.md", "A move.\n")
        errors, _, _ = self.audit()
        self.assertEqual(len(errors), 1)
        self.write("openspec/changes/archive/2026-11-01-01-move/proposal.md",
                   check_spec_sync.NO_BEHAVIOR_SENTENCE + "\n")
        errors, _, _ = self.audit()
        self.assertEqual(errors, [])

    def test_merged_active_change_warns_after_two_days(self) -> None:
        self.write("openspec/changes/122-words/proposal.md", "# Proposal\n")
        merged = [check_spec_sync.MergedCommit(
            "feat(tui): show words [122] (#7)", self.now - timedelta(days=3)
        )]
        errors, warnings, _ = self.audit(merged)
        self.assertEqual(errors, [])
        self.assertEqual(len(warnings), 1)
        self.assertIn("::warning file=openspec/changes/122-words/proposal.md", warnings[0])
        merged[0] = check_spec_sync.MergedCommit(
            merged[0].title, self.now - timedelta(days=2)
        )
        self.assertEqual(self.audit(merged)[1], [])

    def test_merged_range_includes_change_identifier(self) -> None:
        self.assertTrue(check_spec_sync.title_has_id("feat(engine): patterns [96-98k]", "98c"))
        self.assertFalse(check_spec_sync.title_has_id("feat(engine): patterns [96-98k]", "99"))
        self.assertFalse(check_spec_sync.title_has_id("feat(tui): words [122]", "12"))


if __name__ == "__main__":
    unittest.main()
