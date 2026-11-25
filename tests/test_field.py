"""Field port contracts and focused checks of the production inspector."""
from pathlib import Path
import ast
import json
import subprocess
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / 'src'


def check(task, source, cases, validator=''):
    req = {'task': task, 'source': source, 'tests': cases, 'validator': validator}
    process = subprocess.run(
        [sys.executable, '-I', 'runtime/inspector.py'], cwd=ROOT,
        input=json.dumps(req), text=True, encoding='utf-8', capture_output=True, timeout=12,
    )
    if process.returncode:
        raise AssertionError(process.stderr)
    return json.loads(process.stdout)['evaluation']


class FieldPortTests(unittest.TestCase):
    def test_content_and_map(self):
        main = (SRC/'main.rs').read_text()
        regions = (SRC/'regions.rs').read_text()
        map_file = (SRC/'region_map.rs').read_text()
        field = (SRC/'field.rs').read_text()
        self.assertIn('mod field;', main)
        self.assertIn('result.push(field::from_for4())', main)
        self.assertIn('result.push(field::from_tower())', main)
        self.assertIn('result.extend(field::build(initial_seed))', main)
        self.assertIn('Field =>', regions)
        self.assertIn('field_discovered', main)
        self.assertIn('Location::Field =>', map_file)
        self.assertIn('for _ in 0..6', field)
        self.assertIn('for _ in 0..4', field)
        self.assertIn('field_boss', field)

    def test_toml_and_scrolls(self):
        files = list((ROOT/'content/nodes').glob('field_*.toml'))
        self.assertGreaterEqual(len(files), 12)
        for f in files:
            content = tomllib.loads(f.read_text())
            self.assertEqual(f.stem, content['META']['id'])
        for name in ['field_lists1', 'field_lists2', 'field_advanced', 'field_advanced2']:
            text = (ROOT/'content/scrolls'/f'{name}.txt').read_text()
            self.assertGreater(len(text), 500)

    def test_boss_bubble_sort_passes(self):
        source = '''numbers = list(map(int, input().split()))
for i in range(len(numbers)):
    for j in range(len(numbers) - i - 1):
        if numbers[j] > numbers[j + 1]:
            numbers[j], numbers[j + 1] = numbers[j + 1], numbers[j]
print(numbers)
'''
        tests = [{'inputs': ["9 1 8 -4 3 6 0"], 'expected':'[-4, 0, 1, 3, 6, 8, 9]'},
                 {'inputs': ['0 0 0 -9 3 -7 5'], 'expected':'[-9, -7, 0, 0, 0, 3, 5]'}]
        self.assertTrue(check('field_boss', source, tests, 'field_boss.py')['passed'])
        hacked = 'print(sorted(map(int, input().split())))\n'
        self.assertFalse(check('field_boss', hacked, tests, 'field_boss.py')['passed'])

    def test_cherry_two_collections(self):
        expected = "['черно', 'тьма', 'вишня', 'знамя', 'арбуз', 'хлеб']"
        code = '''a = ["арбуз", "хлеб", "знамя", "вишня", "тьма", "черно"]
b = a[::-1]
b[-2], b[-1] = b[-1], b[-2]
print(b)
'''
        self.assertTrue(check('field_cherry', code, [{'inputs':[], 'expected':expected}], 'field_cherry.py')['passed'])
        self.assertFalse(check('field_cherry', f'print({expected})\n', [{'inputs':[], 'expected':expected}], 'field_cherry.py')['passed'])

    def test_median_both_numeric_representations(self):
        case = {'inputs':['1 2 3 4'], 'expected':'2.5'}
        self.assertTrue(check('field_median','a = sorted(map(int, input().split()))\nprint((a[1]+a[2])/2)\n',[case])['passed'])
        case = {'inputs':['1 3 3 5'], 'expected':'3'}
        self.assertTrue(check('field_median','a = sorted(map(int, input().split()))\nprint((a[1]+a[2])/2)\n',[case])['passed'])

    def test_legacy_entry_ids_preserved(self):
        main = (SRC/'main.rs').read_text()
        self.assertIn('result.extend(tower::build_tower_stages(initial_seed));', main)
        self.assertLess(main.index('result.extend(tower::build_tower_stages(initial_seed));'),
                        main.index('result.push(field::from_for4());'))

if __name__ == '__main__':
    unittest.main()

class AdditionalFieldTests(unittest.TestCase):
    def test_ghost_requires_consuming_list(self):
        code = '''values = [7, 9, 3, 1, 8]
while values:
    print(values.pop(0))
'''
        case = [{'inputs': [], 'expected': '7\n9\n3\n1\n8'}]
        self.assertTrue(check('field_ghost', code, case, 'field_ghost.py')['passed'])
        self.assertFalse(check('field_ghost', 'print(7,9,3,1,8,sep="\\n")\n',case,'field_ghost.py')['passed'])

    def test_evil_eye_inputs(self):
        code = '''words = []
while (word := input()) != "стоп":
    if word == "продолжай":
        words.append("хватит")
print(words)
'''
        case = [{'inputs': ['продолжай', 'продолжай', 'стоп'],
                 'expected': "['хватит', 'хватит']"}]
        self.assertTrue(check('field_evil_eye', code, case)['passed'])

    def test_slippers_count(self):
        code = '''things = []
while (word := input()) != "боже":
    things.append(word)
dust = things.count("пыль")
numbers = [int(x) for x in things if x.isdigit()]
print("четное" if (dust // min(numbers)) % 2 == 0 else "нечетное")
'''
        tests = [{'inputs': ['грязь', '2', 'пыль', 'пыль', 'пыль', 'пыль', '6', 'боже'], 'expected': 'четное'},
                 {'inputs': ['пыль', '2', 'пыль', 'пыль', 'пыль', 'пыль', 'пыль', 'боже'], 'expected': 'нечетное'}]
        self.assertTrue(check('field_slippers', code, tests)['passed'])
