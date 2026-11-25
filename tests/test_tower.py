"""Regression tests for original Infinite Tower and validator infrastructure."""
import json
import subprocess
import sys
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INSPECTOR = ROOT / 'runtime' / 'inspector.py'


def run(task, source, tests, validator=None):
    payload = {'task': task, 'source': source, 'tests': tests,
               'validator': validator if validator is not None else f'{task}.py'}
    result = subprocess.run([sys.executable, str(INSPECTOR)], input=json.dumps(payload),
                            text=True, capture_output=True, timeout=12, cwd=ROOT)
    if result.returncode:
        raise AssertionError(result.stderr)
    return json.loads(result.stdout)


class TowerTests(unittest.TestCase):
    def test_toml(self):
        for file in (ROOT / 'content/nodes').glob('tower_*.toml'):
            with self.subTest(file=file.name):
                self.assertEqual(tomllib.loads(file.read_text(encoding='utf-8'))['META']['id'], file.stem)

    def test_joker(self):
        code = ('while True:\n    a = input()\n    if a == "1":\n        continue\n'
                '    if a == "2":\n        print("two pizza")\n'
                '    if a == "3":\n        print("неверно!")\n        break')
        tests = [{'inputs': ['1', '2', '2', '3'], 'expected': 'two pizza\ntwo pizza\nневерно!'}]
        self.assertTrue(run('tower_joker', code, tests)['evaluation']['passed'])
        self.assertFalse(run('tower_joker', 'print("two pizza")', tests)['evaluation']['passed'])

    def test_fortune(self):
        code = ('x = input()\nfor i in range(1, 51):\n    if str(i) == x:\n'
                '        print("чудеса")\n        continue')
        self.assertTrue(run('tower_fortune', code, [{'inputs': ['13'], 'expected': 'чудеса'}])['evaluation']['passed'])
        self.assertFalse(run('tower_fortune', code+'\nbreak', [{'inputs': ['13'], 'expected': 'чудеса'}])['evaluation']['passed'])

    def test_novice_allows_intermediate_dialogue(self):
        code = ('while True:\n    s=input()\n    if s=="да":\n'
                '        print("ура!")\n        break\n    print("где ты?")')
        tests = [{'inputs': ['нет', 'нет', 'да'], 'expected': 'ура!'}]
        self.assertTrue(run('tower_novice', code, tests)['evaluation']['passed'])

    def test_garaon(self):
        code = ('x = True\nwhile x == True:\n    for i in range(5):\n'
                '        print("бух")\n    break')
        tests = [{'inputs': [], 'expected': 'бух\nбух\nбух\nбух\nбух'}]
        self.assertFalse(run('tower_garaon', code, tests)['evaluation']['passed'])
        correct = ('x = True\nwhile x == True:\n    print("бух\\n" * 4 + "бух")\n    break')
        self.assertTrue(run('tower_garaon', correct, tests)['evaluation']['passed'])

    def test_infrastructure_error_is_not_a_failed_player_test(self):
        report = run('tower_novice', 'print("ура!")',
                     [{'inputs': ['да'], 'expected': 'ура!'}], 'not_existing.py')
        ev = report['evaluation']
        self.assertFalse(ev['passed'])
        self.assertTrue(ev.get('infrastructure_error'))
        self.assertEqual(ev['tests_passed'], 1)

    def test_original_cardinality_and_route(self):
        script = (ROOT/'src/tower.rs').read_text(encoding='utf-8')
        self.assertIn('for _ in 0..4', script)
        self.assertIn('choose(&mut seed, 5) == 0', script)
        self.assertIn('river_to_tower', script)
        main = (ROOT/'src/main.rs').read_text(encoding='utf-8')
        self.assertIn('Location::Tower', main)
        self.assertIn('tower_discovered', main)

    def test_scrolls_have_real_newlines(self):
        for name in ['river_branching','river_comparison','river_logic','tower_while','tower_lore']:
            content = (ROOT/'content/scrolls'/f'{name}.txt').read_text(encoding='utf-8')
            self.assertIn('\n', content)
            self.assertNotIn('\\n', content)

if __name__ == '__main__':
    unittest.main()
