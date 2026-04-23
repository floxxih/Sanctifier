import json
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]


class ResolveActionVersionTests(unittest.TestCase):
    def test_fixture_cases(self) -> None:
        from scripts.resolve_action_version import resolve_version

        cases_path = ROOT / "tests" / "action" / "fixtures" / "version-cases.json"
        cases = json.loads(cases_path.read_text(encoding="utf-8"))

        for case in cases:
            with self.subTest(case=case):
                got = resolve_version(
                    action_ref=case["action_ref"],
                    requested=case["requested"],
                )
                self.assertEqual(got, case["expected"])


if __name__ == "__main__":
    unittest.main()

