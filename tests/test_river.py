"""Regression contracts for restored PROJECT PY River content.

Rust map-generation source is checked structurally because Cargo is not
installed in the build environment. Python runner behavior is exercised.
"""
import json
from pathlib import Path
import subprocess
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


def inspect(task, source, cases):
    node = tomllib.loads((ROOT / 'content/nodes' / (task + '.toml')).read_text())
    req = {'task': task, 'source': source, 'tests': cases,
           'validator': node.get('RANKING', {}).get('validator', '')}
    process = subprocess.run([sys.executable, str(ROOT / 'runtime/inspector.py')],
                             input=json.dumps(req), capture_output=True,
                             text=True, check=True, timeout=8)
    return json.loads(process.stdout)


class RiverContentTests(unittest.TestCase):
    def test_original_node_metadata(self):
        names = {'river_tutorial_base', 'river_mumbler', 'river_cane',
                 'river_secret', 'river_tutorial_comparison', 'river_tutorial_logic',
                 'river_arcarh_elite', 'river_boss', 'river_clear'}
        for name in names:
            with self.subTest(node=name):
                node = tomllib.loads((ROOT / 'content/nodes' / (name + '.toml')).read_text())
                self.assertEqual(node['META']['id'], name)
                self.assertEqual(node['META']['location'], 'holy_forest_river')
                if node['META']['type'] == 'enemy.code':
                    validator = node['RANKING']['validator']
                    self.assertTrue((ROOT / 'content/validators'/ validator).is_file())

    def test_tutorials_are_exact_original_texts(self):
        for name, text in [('river_branching.txt', 'input() всегда возвращает СТРОКУ!'),
                           ('river_comparison.txt', 'ДВА знака равно'),
                           ('river_logic.txt', 'ОБА условия верны')]:
            self.assertIn(text, (ROOT / 'content/scrolls' / name).read_text())

    def test_generator_has_original_random_rules(self):
        src = (ROOT / 'src/river.rs').read_text()
        self.assertIn('for _ in 0..4', src)
        self.assertIn('choose(&mut seed, 4) == 0', src)
        self.assertIn('choose(&mut seed, 100) < 18', src)
        self.assertIn('stages.push(boss(&mut seed))', src)
        self.assertIn('stages.push(simple("river_clear"))', src)
        self.assertIn('with_tests("river_mumbler"', src)

    def test_mumbler_requires_branching(self):
        cases = [{'inputs': ['привет!'], 'expected': 'hi'},
                 {'inputs': ['что-то странное'], 'expected': 'hgfhgf'}]
        code = 's = input()\nif s == "привет!":\n    print("hi")\nelse:\n    print("hgfhgf")\n'
        self.assertTrue(inspect('river_mumbler', code, cases)['evaluation']['passed'])
        self.assertFalse(inspect('river_mumbler', 'print("hi")', cases)['evaluation']['passed'])

    def test_cane_all_three_values_must_be_big(self):
        cases = [{'inputs': ['11','12','13'], 'expected': 'это сложная задача'},
                 {'inputs': ['11','8','13'], 'expected': 'легкотня'}]
        code = 'a = int(input())\nb = int(input())\nc = int(input())\nif a > 10 and b > 10 and c > 10:\n    print("это сложная задача")\nelse:\n    print("легкотня")\n'
        result = inspect('river_cane', code, cases)['evaluation']
        self.assertTrue(result['passed'])
        self.assertTrue(result['contextual_excellence'])

    def test_elite_numeric_division_and_other_operators(self):
        cases = [
            {'inputs': ['12','0','/'], 'expected': 'ошибка'},
            {'inputs': ['24','6','/'], 'expected': '4'},
            {'inputs': ['-7','11','+'], 'expected': '4'},
            {'inputs': ['9','4','-'], 'expected': '5'},
            {'inputs': ['8','7','*'], 'expected': '56'},
            {'inputs': ['3','5','?'], 'expected': 'неизвестный оператор'},
        ]
        code = '''a = int(input())
b = int(input())
op = input()
if op == "+":
    print(a + b)
elif op == "-":
    print(a - b)
elif op == "*":
    print(a * b)
elif op == "/":
    if b == 0:
        print("ошибка")
    else:
        print(a / b)
else:
    print("неизвестный оператор")
'''
        result = inspect('river_arcarh_elite', code, cases)
        self.assertTrue(result['evaluation']['passed'], result['evaluation']['diagnostics'])
        self.assertEqual(result['evaluation']['tests_passed'], 6)
        self.assertEqual(result['runs'][1]['inputs'], ['24','6','/'])

    def test_boss_multiple_operations(self):
        cases = [{'inputs': ['10','20','+'], 'expected': '30'},
                 {'inputs': ['10','20','-'], 'expected': '-10'},
                 {'inputs': ['10','20','*'], 'expected': '200'}]
        code = 'a = int(input())\nb = int(input())\nop = input()\nif op == "+":\n    print(a + b)\nelif op == "-":\n    print(a - b)\nelse:\n    print(a * b)\n'
        self.assertTrue(inspect('river_boss', code, cases)['evaluation']['passed'])

    def test_debug_reward_is_persistent(self):
        reward = tomllib.loads((ROOT / 'content/nodes/river_arcarh_elite.toml').read_text())
        self.assertEqual(reward['REWARD']['item'], 'debug_gun')
        item = tomllib.loads((ROOT / 'content/items/debug_gun.toml').read_text())
        self.assertTrue(item['persistent'])
        ui = (ROOT / 'src/main.rs').read_text()
        self.assertIn('Action::ShowDebug', ui)
        self.assertIn('self.last_report = Some(report.clone())', ui)


if __name__ == '__main__':
    unittest.main()
