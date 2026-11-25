"""Original PROJECT PY '4 4' contracts; no Rust compiler required for these tests."""
import json
import subprocess
import sys
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER = ROOT / 'runtime' / 'inspector.py'


def run(task, source, cases):
    request = {'source': source, 'task': task, 'tests': cases,
               'validator': task + '.py'}
    proc = subprocess.run([sys.executable, str(RUNNER)], input=json.dumps(request),
                          text=True, capture_output=True, cwd=ROOT, timeout=20)
    if proc.returncode:
        raise AssertionError(proc.stderr)
    return json.loads(proc.stdout)['evaluation']


class For4Tests(unittest.TestCase):
    def test_all_toml_nodes_valid(self):
        for file in (ROOT / 'content/nodes').glob('for4_*.toml'):
            with self.subTest(file=file.name):
                data = tomllib.loads(file.read_text(encoding='utf-8'))
                self.assertEqual(data['META']['id'], file.stem)

    def test_original_tutorial_present(self):
        self.assertIn('for i in range(1, 8, 1):',
            (ROOT/'content/scrolls/for_base.txt').read_text(encoding='utf-8'))

    def test_greeting(self):
        cases = [{'inputs':['папа'], 'expected':'привет\nпривет'},
                 {'inputs':['вода'], 'expected':''}]
        self.assertTrue(run('for4_greeting',
            'for x in input():\n    if x == "п":\n        print("привет")',cases)['passed'])
        self.assertFalse(run('for4_greeting', 'print("привет")', cases)['passed'])

    def test_carousel_order_independent_and_extra_rejected(self):
        case = [{'inputs':[], 'expected':'9×привет; 6×пока; 3×ппаоа'}]
        code = ('for word, count in [("пока",6),("привет",9),("ппаоа",3)]:\n'
                '    for _ in range(count):\n        print(word)')
        self.assertTrue(run('for4_carousel',code,case)['passed'])
        self.assertFalse(run('for4_carousel',code+'\nprint("лишнее")',case)['passed'])

    def test_kemerovo(self):
        cases = [{'inputs':[], 'expected':'2550'}]
        self.assertTrue(run('for4_kemerovo',
            'print(sum(range(2, 101, 2)))',cases)['passed'])
        self.assertFalse(run('for4_kemerovo','print(2500)',cases)['passed'])

    def test_accountant_and_constraint(self):
        cases = [{'inputs':['1050030000'],'expected':'9'},
                 {'inputs':['1234567890'],'expected':'45'}]
        code = 'n = input()\ntotal = 0\nfor char in n:\n    total += int(char)\nprint(total)'
        self.assertTrue(run('for4_accountant',code,cases)['passed'])
        invalid = 'number = int(input())\nprint(sum(map(int,str(number))))'
        result = run('for4_accountant', invalid, cases)
        self.assertFalse(result['passed'])
        self.assertTrue(any('вне цикла' in x for x in result['diagnostics']))

    def test_bronze_boss_four_hundred_values(self):
        expected = '\n'.join(str(a*b) for a in range(1,21) for b in range(1,21))
        code='for a in range(1,21):\n    for b in range(1,21):\n        print(a*b)'
        self.assertTrue(run('for4_boss',code,[{'inputs':[],'expected':expected}])['passed'])
        self.assertFalse(run('for4_boss', 'print(400)', [{'inputs':[],'expected':expected}])['passed'])

    def test_river_transition_and_dynamic_indices(self):
        source=(ROOT/'src/main.rs').read_text(encoding='utf-8') + (ROOT/'src/regions.rs').read_text(encoding='utf-8')
        generator=(ROOT/'src/for4.rs').read_text(encoding='utf-8')
        self.assertIn('river_to_for4(&self.stages)', source)
        self.assertIn('for4_first(&self.stages)', source)
        self.assertIn('Location::For4', source)
        self.assertIn('choose(&mut seed, 5) == 0', generator)
        self.assertIn('match choose(&mut seed, 4)',generator)


if __name__ == '__main__':
    unittest.main()
