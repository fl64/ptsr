"""Tests for the changed-path test selection."""

import unittest

from test_changed import plan


class ChangedTests(unittest.TestCase):
    def test_engine_runs_render_tests(self):
        self.assertEqual(plan({"crates/engine/src/rt.rs"}), [["cargo", "nextest", "run", "-p", "engine"]])

    def test_tui_checks_catalogue(self):
        self.assertEqual(plan({"crates/tui/src/lib.rs"})[-1], ["task", "screens"])

    def test_input_checks_keyboard_and_raw_midi(self):
        self.assertIn("test(input_tests::)", plan({"crates/input/src/lib.rs"})[-1][-1])

    def test_app_module_uses_matching_input_tests(self):
        self.assertIn("test(input_tests::patterns::)", plan({"crates/app/src/pattern_record.rs"})[-1][-1])

    def test_specs_use_only_scenarios(self):
        self.assertEqual(
            plan({"openspec/specs/pattern/spec.md"}),
            [["cargo", "nextest", "run", "-p", "ptsr", "-E", "binary(scenarios)"]],
        )

    def test_main_uses_all_app_tests(self):
        self.assertEqual(plan({"crates/app/src/main.rs"}), [["cargo", "nextest", "run", "-p", "ptsr"]])

    def test_integration_file_uses_its_binary(self):
        self.assertEqual(
            plan({"crates/app/tests/scenarios.rs"}),
            [["cargo", "nextest", "run", "-p", "ptsr", "-E", "binary(scenarios)"]],
        )

    def test_unknown_path_uses_fast_set(self):
        self.assertEqual(plan({"Cargo.toml"}), [["task", "test:fast"]])


if __name__ == "__main__":
    unittest.main()
