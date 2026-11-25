"""Regression tests for conservative, explainable SHARKCODE 0.2.40 quality scoring."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'runtime'))
import inspector
import quality


def grade(source, task='shyguy', expected=None, starter=''):
    compiled = inspector.compile_traced(source)
    case = inspector.run_case(compiled, [])
    if expected is not None:
        case['expected'] = expected
    return inspector.evaluate(task, [case], inspector.analyze(source), source,
                              starter_source=starter)


class QualityV3Tests(unittest.TestCase):
    def test_hello_minimal_stays_splusplus(self):
        result = grade('print("Hello world!")\n')
        self.assertEqual(result['rank'], 'S++')
        self.assertTrue(result['passed'])

    def test_dead_branch_costs_rank(self):
        result = grade('print("Hello world!")\nif False:\n    print("never")\n')
        self.assertEqual(result['rank'], 'A')
        self.assertTrue(any('DEAD_BRANCH' in x for x in result['quality_reasons']))

    def test_literal_comparison_dead_branch(self):
        result = grade('print("Hello world!")\nif 1 > 2:\n    print("never")\n')
        self.assertNotEqual(result['rank'], 'S++')

    def test_many_dead_branches_can_be_c(self):
        result = grade('print("Hello world!")\n' + 'if False:\n    print("never")\n' * 5)
        self.assertEqual(result['rank'], 'C')
        self.assertTrue(result['passed'])

    def test_unreachable_after_return(self):
        source = ('def hello():\n    return "Hello world!"\n'
                  '    print("never")\nprint(hello())\n')
        result = grade(source)
        self.assertTrue(result['passed'])
        self.assertNotEqual(result['rank'], 'S++')
        self.assertTrue(any('UNREACHABLE' in x for x in result['quality_reasons']))

    def test_constant_true_has_penalty(self):
        result = grade('if True:\n    print("Hello world!")\n')
        self.assertEqual(result['rank'], 'S')

    def test_while_true_with_break_not_flagged(self):
        result = grade('while True:\n    print("Hello world!")\n    break\n')
        self.assertEqual(result['rank'], 'S++')

    def test_docstring_is_not_dead_expression(self):
        result = grade('"""An educational script."""\nprint("Hello world!")\n')
        self.assertEqual(result['rank'], 'S++')

    def test_expression_with_no_effect_is_penalized(self):
        result = grade('42\nprint("Hello world!")\n')
        self.assertEqual(result['rank'], 'S')

    def test_starter_code_ignored_for_ast_penalties(self):
        starter = 'if True:\n    print("Hello world!")\n'
        result = grade(starter, starter=starter)
        self.assertEqual(result['rank'], 'S++')

    def test_more_lines_not_inherently_bad(self):
        source = ('message = "Hello world!"\n'
                  'if message:\n'
                  '    print(message)\n')
        result = grade(source)
        self.assertEqual(result['rank'], 'S++')

    def test_failed_execution_never_gets_rank(self):
        result = grade('print("wrong")\n')
        self.assertFalse(result['passed'])
        self.assertIsNone(result['rank'])

    def test_starter_preamble_does_not_protect_player_lines(self):
        starter = 'x = True\nwhile x == True:\n    '
        source = starter + 'print("Hello world!")\n    if False:\n        print("never")\n    break\n'
        findings = quality.inspect(source, starter)
        self.assertTrue(any(f.code == 'DEAD_BRANCH' for f in findings))


if __name__ == '__main__':
    unittest.main()
