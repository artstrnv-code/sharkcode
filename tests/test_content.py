"""Tests for TOML contracts, inventory content, and external Python validators."""
import json
import subprocess
import sys
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class ContentTests(unittest.TestCase):
    def test_all_existing_nodes_are_toml(self):
        names = ('tutorial', 'stesnyaga', 'counter', 'wise', 'egg',
                 'statue', 'card_player', 'echo', 'chest', 'grot')
        for name in names:
            with self.subTest(node=name):
                data = tomllib.loads((ROOT / 'content' / 'nodes' /
                                      f'{name}.toml').read_text())
                meta = data['META']
                self.assertEqual(meta['id'], name)
                self.assertIn(meta['type'], ('scroll', 'empty', 'enemy.code',
                                             'enemy.quiz', 'chest.code', 'entry'))
                if meta['type'].endswith('.code'):
                    val = data['RANKING']['validator']
                    self.assertTrue((ROOT / 'content' / 'validators' / val).exists())

    def test_statue_recovers_original_reward(self):
        data = tomllib.loads((ROOT / 'content/nodes/statue.toml').read_text())
        item = data['REWARD']['item']
        self.assertEqual(item, 'double_shield')
        reward = tomllib.loads((ROOT / 'content/items/double_shield.toml').read_text())
        self.assertEqual(reward['name'], 'Двойной щит')

    def test_external_validator_is_called(self):
        req = {'source': 'print("Hello world!")', 'task': 'shyguy',
               'validator': 'stesnyaga.py', 'ranking': {'pep8_coef': 1.0,
                                                     'strokes_coef': 1.0},
               'tests': [{'inputs': [], 'expected': 'Hello world!'}]}
        proc = subprocess.run([sys.executable, str(ROOT / 'runtime/inspector.py')],
                              input=json.dumps(req), text=True, check=True,
                              capture_output=True, timeout=5)
        result = json.loads(proc.stdout)
        self.assertTrue(result['evaluation']['passed'])

    def test_shyguy_requires_new_exact_output(self):
        """Both base inspection and the TOML validator reject the old text."""
        contract = tomllib.loads((ROOT / 'content/nodes/stesnyaga.toml').read_text(encoding='utf-8'))
        self.assertEqual(contract['GAME']['wait_output'], 'Hello world!')
        self.assertIn('Hello world!', contract['META']['tt'])
        for answer, passed in [('Hello world!', True), ('hello world', False),
                               ('Hello world', False), ('hello world!', False)]:
            with self.subTest(answer=answer):
                req = {'task': 'shyguy', 'source': f'print({answer!r})',
                       'validator': 'stesnyaga.py',
                       'tests': [{'inputs': [], 'expected': contract['GAME']['wait_output']}]}
                proc = subprocess.run([sys.executable, str(ROOT / 'runtime/inspector.py')],
                                      input=json.dumps(req), text=True, capture_output=True,
                                      timeout=5, check=True)
                result = json.loads(proc.stdout)
                self.assertEqual(result['evaluation']['passed'], passed)

    def test_shyguy_special_validator_respects_opted_in_typo_tolerance(self):
        """The core's accepted typo must not be vetoed by node validator."""
        req = {'source': 'print("hello world!")\n', 'task': 'shyguy',
               'validator': 'stesnyaga.py', 'text_tolerance': True,
               'tests': [{'inputs': [], 'expected': 'Hello world!'}]}
        proc = subprocess.run([sys.executable, '-I', str(ROOT / 'runtime/inspector.py')],
                              input=json.dumps(req), text=True, check=True,
                              capture_output=True, timeout=5)
        report = json.loads(proc.stdout)
        self.assertTrue(report['runs'][0]['passed'])
        self.assertTrue(report['evaluation']['validator_passed'])
        self.assertTrue(report['evaluation']['passed'])
        self.assertEqual(report['evaluation']['rank'], 'S++')

    def test_shyguy_special_validator_keeps_strict_default(self):
        req = {'source': 'print("hello world!")\n', 'task': 'shyguy',
               'validator': 'stesnyaga.py', 'text_tolerance': False,
               'tests': [{'inputs': [], 'expected': 'Hello world!'}]}
        proc = subprocess.run([sys.executable, '-I', str(ROOT / 'runtime/inspector.py')],
                              input=json.dumps(req), text=True, check=True,
                              capture_output=True, timeout=5)
        report = json.loads(proc.stdout)
        self.assertFalse(report['runs'][0]['passed'])
        self.assertFalse(report['evaluation']['passed'])

    def test_shyguy_special_validator_rejects_large_mismatch(self):
        req = {'source': 'print("goodbye world!")\n', 'task': 'shyguy',
               'validator': 'stesnyaga.py', 'text_tolerance': True,
               'tests': [{'inputs': [], 'expected': 'Hello world!'}]}
        proc = subprocess.run([sys.executable, '-I', str(ROOT / 'runtime/inspector.py')],
                              input=json.dumps(req), text=True, check=True,
                              capture_output=True, timeout=5)
        report = json.loads(proc.stdout)
        self.assertFalse(report['runs'][0]['passed'])
        self.assertFalse(report['evaluation']['passed'])

    def test_external_validator_cannot_override_failed_core(self):
        req = {'source': 'print("Hello world!")', 'task': 'shyguy',
               'validator': 'stesnyaga.py',
               'tests': [{'inputs': [], 'expected': 'something else'}]}
        proc = subprocess.run([sys.executable, str(ROOT / 'runtime/inspector.py')],
                              input=json.dumps(req), text=True, check=True,
                              capture_output=True, timeout=5)
        self.assertFalse(json.loads(proc.stdout)['evaluation']['passed'])


if __name__ == '__main__':
    unittest.main()
