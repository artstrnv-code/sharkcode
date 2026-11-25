"""Exercise the real inspector process with the same -I flag Rust uses."""
import json
import pathlib
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
INSPECTOR = ROOT / 'runtime' / 'inspector.py'


class IsolatedImportTests(unittest.TestCase):
    def test_inspector_imports_sibling_quality_in_isolated_mode(self):
        payload = {
            'task': 'shyguy',
            'source': 'print("Hello world!")\n',
            'tests': [{'inputs': []}],
        }
        response = subprocess.run(
            [sys.executable, '-X', 'utf8', '-I', str(INSPECTOR)],
            input=json.dumps(payload), text=True,
            capture_output=True, timeout=10,
            cwd=ROOT,
        )
        self.assertEqual(response.returncode, 0, response.stderr)
        report = json.loads(response.stdout)
        self.assertTrue(report['evaluation']['passed'])
        self.assertEqual(report['evaluation']['rank'], 'S++')

    def test_import_does_not_change_python_search_path_globally(self):
        script = (
            'import runpy, sys; before = list(sys.path); '
            'runpy.run_path(sys.argv[1], run_name="__sharkcode_import_check__"); '
            'assert sys.path == before'
        )
        response = subprocess.run(
            [sys.executable, '-I', '-c', script, str(INSPECTOR)],
            text=True, capture_output=True, timeout=10, cwd=ROOT,
        )
        self.assertEqual(response.returncode, 0, response.stderr)


if __name__ == '__main__':
    unittest.main()
