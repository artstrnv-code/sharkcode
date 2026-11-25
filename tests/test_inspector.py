"""Python-only regression tests: python -m unittest discover -s tests -v"""
import json
import subprocess
import sys
import unittest
from pathlib import Path

RUNNER = Path(__file__).resolve().parents[1] / 'runtime' / 'inspector.py'


def check(task, source, tests=None):
    result = subprocess.run([sys.executable, str(RUNNER)], text=True,
        input=json.dumps({'task': task, 'source': source,
                          'tests': tests or [{'inputs': []}]}),
        capture_output=True, timeout=5, check=True)
    return json.loads(result.stdout)


class InspectorTests(unittest.TestCase):
    def test_shyguy(self):
        self.assertTrue(check('shyguy', 'print("Hello world!")')['evaluation']['passed'])
        self.assertFalse(check('shyguy', 'print("hello world")')['evaluation']['passed'])
        self.assertFalse(check('shyguy', 'print("goodbye")')['evaluation']['passed'])

    def test_echo(self):
        self.assertTrue(check('echo', 'for i in range(1,8): print("бух" * i)')['evaluation']['passed'])

    def test_chest_without_stdout(self):
        good = 'a = 10\nb = "hello"\nbuffer = a\na = b\nb = buffer'
        compact = 'a = 10\nb = "hello"\na, b = b, a'
        cheat = 'a = 10\nb = "hello"\na = "hello"\nb = 10'
        for source in (good, compact):
            self.assertTrue(check('chest', source)['evaluation']['passed'])
        self.assertFalse(check('chest', cheat)['evaluation']['passed'])
        self.assertFalse(check('chest', compact+'\nprint("done")')['evaluation']['passed'])

    def test_snapshot(self):
        r = check('chest', 'a=10\nb="hello"\na,b=b,a')
        self.assertEqual(r['runs'][0]['state']['a']['value'], "'hello'")
        self.assertEqual(r['runs'][0]['state']['b']['value'], '10')

    def test_hello_secret(self):
        # Syntax recognition is independent of whether this CPython version
        # exposes __hello__.main().
        r = check('shyguy','import __hello__\n__hello__.main()')
        self.assertTrue(r['analysis']['grot_trigger'])

    def test_secret_requires_executed_call(self):
        # Dead code must not unlock the Grot, even if the AST contains the spell.
        never = 'import __hello__\nif False:\n    __hello__.main()'
        r = check('shyguy', never)
        self.assertFalse(r['analysis']['grot_trigger'])
        # A real executed call does unlock, including if it fails the battle task.
        real = check('shyguy', 'import __hello__\n__hello__.main()')
        self.assertTrue(real['analysis']['grot_trigger'])

    def test_bounded_console_output(self):
        r = check('shyguy', 'print("x" * 1000000)')
        self.assertLessEqual(len(r['runs'][0]['stdout']), 10000)
        self.assertFalse(r['evaluation']['passed'])

    def test_chest_rejects_fake_final_state(self):
        r = check('chest', 'a=10\nb="hello"\na="hello"\nb=10')
        self.assertFalse(r['evaluation']['passed'])

    def test_multiple_inputs_are_fresh(self):
        tests = [{'inputs': [], 'expected': '3'}, {'inputs': [], 'expected': '3'}]
        r = check('counter', 'x=1\nprint(x+2)', tests)
        self.assertTrue(r['evaluation']['passed'])
        self.assertEqual(len(r['runs']), 2)


if __name__ == '__main__': unittest.main()


class TransportTests(unittest.TestCase):
    """Runner must emit UTF-8-compatible JSON on Windows with non-UTF-8 locale."""

    def test_cyrillic_report_is_ascii_safe(self):
        request = {'task': 'egg', 'source': 'print("бух бух бух ")',
                   'tests': [{'inputs': []}]}
        data = subprocess.run([sys.executable, '-X', 'utf8', '-I', str(RUNNER)],
                              input=json.dumps(request).encode('ascii'),
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              timeout=5, check=True).stdout
        data.decode('ascii')  # no locale-dependent output encoding
        report = json.loads(data.decode('utf-8'))
        self.assertEqual(report['runs'][0]['stdout'], 'бух бух бух \n')
        self.assertTrue(report['evaluation']['passed'])

class DiagnosticsTests(unittest.TestCase):
    def test_expected_and_observed_are_reported_even_on_success(self):
        result = check('shyguy', 'print("Hello world!")')
        self.assertEqual(result['evaluation']['tests_passed'], 1)
        self.assertEqual(result['runs'][0]['stdout'], 'Hello world!\n')
        self.assertEqual(result['runs'][0]['expected_display'], 'Hello world!\n')
        self.assertTrue(result['runs'][0]['passed'])

    def test_failed_result_has_expected_actual(self):
        result = check('shyguy', 'print("goodbye")')
        self.assertEqual(result['evaluation']['tests_passed'], 0)
        self.assertEqual(result['runs'][0]['stdout'], 'goodbye\n')
        self.assertEqual(result['runs'][0]['expected_display'], 'Hello world!\n')

    def test_pep8_spacing_report(self):
        bad = check('chest', 'a=10\nb="hello"\na,b=b,a')
        self.assertTrue(bad['evaluation']['passed'])
        self.assertFalse(bad['evaluation']['pep8_ok'])
        self.assertTrue(bad['evaluation']['pep8_issues'])
        self.assertNotEqual(bad['evaluation']['rank'], 'S++')

    def test_pep8_clean_contextual_excellence(self):
        good = check('chest', 'a = 10\nb = "hello"\na, b = b, a\n')
        self.assertTrue(good['evaluation']['passed'])
        self.assertTrue(good['evaluation']['pep8_ok'])
        self.assertEqual(good['evaluation']['rank'], 'S++')

    def test_zero_stdout_chest_display(self):
        report = check('chest', 'a = 10\nb = "hello"\na, b = b, a')
        self.assertEqual(report['runs'][0]['stdout'], '')
        self.assertEqual(report['runs'][0]['expected_display'], '')
        self.assertEqual(report['evaluation']['tests_passed'], 1)

    def test_multiple_tests_count(self):
        report = check('counter', 'print(3)', [
            {'inputs': [], 'expected': '3'},
            {'inputs': [], 'expected': '4'},
        ])
        self.assertEqual(report['evaluation']['tests_passed'], 1)
        self.assertFalse(report['evaluation']['passed'])
