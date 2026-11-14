"""Tests for the CI benchmark report parser."""

import unittest

from ci_bench_compare import metrics


class BenchmarkReport(unittest.TestCase):
    def test_reads_all_measures(self):
        report = "".join(
            f"paired timing: {name}\n  wall: mean 1.00 %, p99 2.50 %, max 3.00 %\n"
            for name in ("steady playback", "dense pattern playback", "active recording", "active erasure")
        )
        report += "project switch: 200 switches, switch peak p99 4.20 %\n"
        report += "dense pattern: 4096 hits, dropped hits 0\n"
        result = metrics(report)
        self.assertEqual(result["steady playback"], 2.5)
        self.assertEqual(result["switch peak"], 4.2)
        self.assertEqual(result["dropped hits"], 0)


if __name__ == "__main__":
    unittest.main()
