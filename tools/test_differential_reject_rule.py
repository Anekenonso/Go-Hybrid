"""CLI regression tests for rejection-reason agreement (fake checker only)."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


HARNESS = Path(__file__).with_name("differential_loop.py")


class RejectRuleTests(unittest.TestCase):
    def run_case(self, diagnostics):
        return self.run_payload({"summary": {"verdict": "reject"},
                                 "diagnostics": diagnostics})

    def run_payload(self, envelope):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            corpus = root / "corpus"
            corpus.mkdir()
            (corpus / "01-accept.hy").write_text(
                "// EXPECTED-VERDICT: ACCEPT\nfn main() {}\n", encoding="utf-8")
            (corpus / "02-reject.hy").write_text(
                "// EXPECTED-VERDICT: REJECT A1\nfn main() {}\n", encoding="utf-8")
            payload = json.dumps(envelope)
            program = f"#!{sys.executable}\n"
            checker = root / "hyb"
            checker.write_text(
                f"#!{sys.executable}\nimport pathlib, sys\n"
                "if sys.argv[1] == 'check':\n"
                f"    print({payload!r})\n    sys.exit(1)\n"
                "p = pathlib.Path('a.out')\n"
                f"p.write_text({program!r}, encoding='utf-8')\np.chmod(0o755)\n",
                encoding="utf-8")
            checker.chmod(0o755)
            return subprocess.run(
                [sys.executable, str(HARNESS), "--bin", str(checker),
                 "--corpus", str(corpus)], capture_output=True, text=True,
                timeout=10)

    def assert_result(self, diagnostics, expected_code):
        result = self.run_case(diagnostics)
        self.assertEqual(result.returncode, expected_code, result.stdout + result.stderr)
        self.assertEqual("FULL AGREEMENT" in result.stdout, expected_code == 0)
        return result

    def test_matching_first_error_passes(self):
        self.assert_result([{"severity": "error", "tier0_rule": "A1"}], 0)

    def test_wrong_rule_fails(self):
        result = self.assert_result([{"severity": "error", "tier0_rule": "FC"}], 1)
        self.assertIn("first_rule=FC", result.stdout)

    def test_unclassified_error_fails(self):
        self.assert_result([{"severity": "error", "code": "HYB-E0002"}], 1)

    def test_later_matching_error_cannot_hide_wrong_first_error(self):
        self.assert_result([{"severity": "error", "tier0_rule": "FC"},
                            {"severity": "error", "tier0_rule": "A1"}], 1)

    def test_warning_does_not_override_matching_first_error(self):
        self.assert_result([{"severity": "warning", "tier0_rule": "FC"},
                            {"severity": "error", "tier0_rule": "A1"}], 0)

    def test_matching_warning_without_errors_fails(self):
        self.assert_result([{"severity": "warning", "tier0_rule": "A1"}], 1)

    def assert_protocol_failure(self, envelope):
        result = self.run_payload(envelope)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("FAIL  02-reject.hy", result.stdout)
        self.assertIn("differential reject-side (checker):   0/1", result.stdout)
        self.assertNotIn("FULL AGREEMENT", result.stdout)
        self.assertNotIn("Traceback", result.stderr)
        return result

    def test_malformed_envelopes_are_case_failures_without_tracebacks(self):
        for envelope in (
            None, [], 7,
            {"summary": None, "diagnostics": []},
            {"summary": [], "diagnostics": []},
            {"summary": {"verdict": "reject"}, "diagnostics": None},
            {"summary": {"verdict": "reject"}, "diagnostics": {}},
            {"summary": {"verdict": "reject"}, "diagnostics": [None]},
            {"summary": {"verdict": "reject"}, "diagnostics": ["error"]},
        ):
            with self.subTest(envelope=envelope):
                self.assert_protocol_failure(envelope)

    def test_inconsistent_error_counts_cannot_report_agreement(self):
        for count in (0, 2, -1, True, "1", None):
            with self.subTest(count=count):
                self.assert_protocol_failure({
                    "summary": {"verdict": "reject", "errors": count},
                    "diagnostics": [{"severity": "error", "tier0_rule": "A1"}],
                })

    def test_inconsistent_warning_counts_cannot_report_agreement(self):
        result = self.assert_protocol_failure({
            "summary": {"verdict": "reject", "errors": 1, "warnings": 0},
            "diagnostics": [{"severity": "error", "tier0_rule": "A1"},
                            {"severity": "warning", "code": "HYB-E8001"}],
        })
        self.assertIn("warnings", result.stdout)

    def test_valid_counts_preserve_agreement(self):
        result = self.run_payload({
            "summary": {"verdict": "reject", "errors": 1, "warnings": 1},
            "diagnostics": [{"severity": "error", "tier0_rule": "A1"},
                            {"severity": "warning", "code": "HYB-E8001"}],
        })
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("FULL AGREEMENT", result.stdout)


if __name__ == "__main__":
    unittest.main()
