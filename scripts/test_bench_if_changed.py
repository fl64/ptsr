"""Tests for the local benchmark condition."""

import unittest

from bench_if_changed import needs_bench


class BenchmarkSelection(unittest.TestCase):
    def test_engine_and_audio_need_benchmark(self):
        self.assertTrue(needs_bench({"crates/engine/src/rt.rs"}))
        self.assertTrue(needs_bench({"crates/app/src/audio.rs"}))

    def test_other_changes_do_not_need_benchmark(self):
        self.assertFalse(needs_bench({"openspec/specs/foo/spec.md", "crates/app/src/browser.rs"}))


if __name__ == "__main__":
    unittest.main()
