"""0.2.50 — project-enemy contracts, deterministic test data and guild unlocks."""
import ast
import importlib.util
import json
import subprocess
import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('judge047', ROOT / 'runtime/judge047.py')
JUDGE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(JUDGE)


class ProjectContentTests(unittest.TestCase):
    def test_converter_base_variants(self):
        tests = [
            {'inputs': ['0', '2'], 'expected': '0'},
            {'inputs': ['1011', '2'], 'expected': '11'},
            {'inputs': ['FF', '16'], 'expected': '255'},
            {'inputs': ['Z', '36'], 'expected': '35'},
            {'inputs': ['123', '8'], 'expected': '83'},
        ]
        source = '''digits = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"
num = input().upper()
base = int(input())
result = 0
for char in num:
    result = result * base + digits.index(char)
print(result)
'''
        request = {'task': 'project_converter', 'source': source, 'tests': tests}
        self.assertTrue(JUDGE.judge(request)['evaluation']['passed'])
        bypass = request | {'source': 'print(int(input(), int(input())))\n'}
        report = JUDGE.judge(bypass)['evaluation']
        self.assertFalse(report['passed'])
        self.assertIn('одним аргументом', ' '.join(report['diagnostics']))
        hardcode = request | {'source': 'print(0)\n'}
        self.assertFalse(JUDGE.judge(hardcode)['evaluation']['passed'])

    def test_void_read_stream(self):
        tests = [
            {'stdin': ' \t\n' * 21 + '\nSTOP\n', 'expected': chr(65)},
            {'stdin': '\t' * 65 + '\nSTOP\n', 'expected': chr(67)},
            {'stdin': '\n' * 45 + '\t\nSTOP\n', 'expected': chr(93)},
        ]
        # each first case score: (space=0,tab=1,newline=2)*21 + 2=65
        # second case score: 65+2=67
        # third case score: 90+1+2=93
        source = '''import sys
value = 0
for line in sys.stdin:
    if line.strip():
        break
    value += line.count("\\t")
    value += 2 * line.count("\\n")
print(chr(value))
'''
        request = {'task': 'project_void', 'source': source, 'tests': tests}
        result = JUDGE.judge(request)['evaluation']
        self.assertTrue(result['passed'], result)
        alt = request | {'source': '''import sys
value = 0
while True:
    char = sys.stdin.read(1)
    if not char or (char != "\\n" and not char.isspace()):
        break
    value += {"\\t": 1, "\\n": 2}.get(char, 0)
print(chr(value))
'''}
        self.assertTrue(JUDGE.judge(alt)['evaluation']['passed'])

    def test_isolated_inspector(self):
        req = {'task': 'project_converter', 'source': 'print(255)\n',
               'tests': [{'inputs': ['FF', '16'], 'expected': '255'}]}
        result = subprocess.run([sys.executable, '-I', 'runtime/inspector.py'],
                                input=json.dumps(req), cwd=ROOT, text=True,
                                capture_output=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(json.loads(result.stdout)['evaluation']['passed'])

    def test_guild_pages_exist_and_rust_declares_them(self):
        main = (ROOT / 'src/main.rs').read_text()
        for name in ['intro', 'names', 'comments', 'spaces', 'newline', 'conditions']:
            text = ROOT / f'content/scrolls/knights_{name}.txt'
            self.assertTrue(text.exists(), name)
            self.assertIn(f'knights_{name}.txt', main)
        self.assertIn('matches!(rank, "S" | "S++")', main)
        self.assertIn('"ever_visited_mills":self.ever_visited_mills', main)

    def test_elf_single_test_case(self):
        mills = (ROOT / 'src/mills.rs').read_text()
        self.assertIn('let cases=(0..1).map(|_| {', mills)


if __name__ == '__main__':
    unittest.main()
