"""Regression tests for SHARKCODE 0.2.54.

Run from SHARKCODE root: python -m unittest discover -s tests -p 'test_roofs_054.py'
"""
import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def inspect(task, source, tests=None):
    payload = {'task': task, 'source': source, 'tests': tests or [],
               'validator': '', 'ranking': {}}
    result = subprocess.run([sys.executable, '-I', str(ROOT / 'runtime' / 'inspector.py')],
                            input=json.dumps(payload), text=True, capture_output=True,
                            cwd=ROOT, timeout=30, check=True)
    return json.loads(result.stdout)['evaluation']


class RoofsTests(unittest.TestCase):
    def test_balls(self):
        source = '''import random
colors=[]
while True:
    color=input()
    if not color: break
    colors.append(color)
old=colors.copy()
random.shuffle(colors)
print(sum(a==b for a,b in zip(old,colors)))
'''
        cases = [{'inputs': ['зелёный', 'синий', 'зелёный', '']},
                 {'inputs': ['белый', '']}]
        self.assertTrue(inspect('roof_balls', source, cases)['passed'])
        self.assertFalse(inspect('roof_balls', 'print(0)\n', cases)['passed'])

    def test_hero(self):
        self.assertTrue(inspect('roof_hero', 'import random\nfor _ in range(3):\n    random.choice([1])\n')['passed'])
        self.assertFalse(inspect('roof_hero', 'import random\nrandom.choice([1])\n')['passed'])
        self.assertFalse(inspect('roof_hero', 'print(1)\n')['passed'])

    def test_boss(self):
        source = '''start=int(input())
end=int(input())
seed=int(input())
x=(seed*1103515245+12345)%2147483647
print(start+x%(end-start+1))
'''
        self.assertTrue(inspect('roof_boss', source)['passed'])
        self.assertFalse(inspect('roof_boss', 'a=int(input())\nb=int(input())\ns=int(input())\nprint(a)\n')['passed'])
        self.assertFalse(inspect('roof_boss', 'import random\na=int(input())\nb=int(input())\ns=int(input())\nprint(random.randint(a,b))\n')['passed'])


if __name__ == '__main__':
    unittest.main()
