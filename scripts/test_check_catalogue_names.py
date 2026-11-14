import tempfile
import unittest
from pathlib import Path

import check_catalogue_names


class CatalogueNamesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.write("crates/app/src/screens.rs", """const SIZES: [(u16, u16); 2] = [(120, 40), (80, 26)];
impl Mode {
    const ALL: &'static [Self] = &[Self::Normal, Self::Tone];
    fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Tone => "edit-tone",
        }
    }
    fn applies(self, screen: Screen) -> bool {
        match self {
            Self::Tone => screen == Screen::Pads,
            _ => true,
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Case;
""")
        self.write("crates/input/src/lib.rs", """impl Screen {
    pub const ALL: [Self; 2] = [Self::Pads, Self::Files];
    fn info(self) -> ScreenInfo {
        match self {
            Self::Pads => ScreenInfo { key: "pads", screen_learn: true },
            Self::Files => ScreenInfo { key: "files", screen_learn: true },
        }
    }
    /// The screen code
}
""")
        (self.root / "openspec/changes").mkdir(parents=True)

    def write(self, relative: str, content: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def test_existing_names_and_both_sizes_pass_without_generated_files(self) -> None:
        self.write("openspec/changes/124-view/proposal.md",
                   "Inspect `pads-normal`, `pads-edit-tone-80x26.ansi`, and "
                   "`renders/screens/files-normal-120x40.ansi`.\n")
        errors, count = check_catalogue_names.check(self.root)
        self.assertEqual((errors, count), ([], 3))

    def test_missing_catalogue_name_fails_even_with_generated_file(self) -> None:
        self.write("openspec/changes/124-view/proposal.md", "Inspect `pads-missing`.\n")
        self.write("renders/screens/pads-missing-120x40.ansi", "not a fixture\n")
        errors, count = check_catalogue_names.check(self.root)
        self.assertEqual(count, 1)
        self.assertIn("pads-missing", errors[0])

    def test_mode_must_apply_to_the_named_screen(self) -> None:
        self.write("openspec/changes/124-view/proposal.md", "Inspect `files-edit-tone`.\n")
        errors, count = check_catalogue_names.check(self.root)
        self.assertEqual(count, 1)
        self.assertIn("files-edit-tone", errors[0])

    def test_unknown_wildcard_has_no_fixture(self) -> None:
        self.write("openspec/changes/124-view/proposal.md",
                   "Inspect `renders/screens/pads-absent-*.ansi`.\n")
        errors, count = check_catalogue_names.check(self.root)
        self.assertEqual(count, 1)
        self.assertIn("pads-absent-*.ansi", errors[0])


if __name__ == "__main__":
    unittest.main()
