"""Regression tests for narrowly scoped typo tolerance and computation intent."""
import sys
import unittest
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'runtime'))
import inspector
import quality


def grade(source, task='shyguy', expected=None, **policies):
    code = inspector.compile_traced(source)
    run = inspector.run_case(code, [])
    if expected is not None:
        run['expected'] = expected
    return inspector.evaluate(task, [run], inspector.analyze(source), source, **policies)


class RankV4(unittest.TestCase):
    def test_literal_minimal_is_still_splusplus(self):
        result = grade('print("Hello world!")\n', text_tolerance=True)
        self.assertEqual(result['rank'], 'S++')

    def test_single_typo_is_free_opted_in(self):
        result = grade('print("Hella world!")\n', text_tolerance=True)
        self.assertEqual(result['rank'], 'S++')
        self.assertTrue(result['passed'])

    def test_single_typo_fails_without_optin(self):
        self.assertFalse(grade('print("Hella world!")\n')['passed'])

    def test_two_typos_cap_at_a(self):
        result = grade('print("Hella warld!")\n', text_tolerance=True)
        self.assertTrue(result['passed'])
        self.assertEqual(result['rank'], 'A')

    def test_three_typos_cap_at_b(self):
        result = grade('print("Hella warld?")\n', text_tolerance=True)
        self.assertTrue(result['passed'])
        self.assertEqual(result['rank'], 'B')

    def test_heavily_wrong_output_fails(self):
        self.assertFalse(grade('print("Different!!!")\n', text_tolerance=True)['passed'])

    def test_newline_lines_are_not_fuzzy_merged(self):
        self.assertFalse(inspector.compare_literal_text('hello world', 'hello\nworld')[0])

    def test_numeric_answer_is_not_fuzzed(self):
        self.assertFalse(inspector.compare_literal_text('2551', '2550')[0])

    def test_numeric_literal_in_computation_node_loses_splusplus(self):
        result = grade('print(2550)\n', task='for4_kemerovo', expected='2550', computation_required=True)
        self.assertTrue(result['passed'])
        self.assertNotEqual(result['rank'], 'S++')
        self.assertTrue(any('готовый числовой' in x for x in result['quality_reasons']))

    def test_numeric_literal_allowed_for_output_task(self):
        result = grade('print(2550)\n', task='for4_kemerovo', expected='2550')
        self.assertEqual(result['rank'], 'S++')

    def test_arithmetic_shortcut_not_hardcode(self):
        self.assertFalse(quality.numeric_answer_hardcoded('print(120 ** 11)\n'))
        result = grade('print(120 ** 11)\n', task='statue', expected=str(120 ** 11), computation_required=True)
        self.assertEqual(result['rank'], 'S++')

    def test_output_from_input_not_hardcode(self):
        self.assertFalse(quality.numeric_answer_hardcoded('print(int(input()))\n'))

    def test_dead_assignment_penalized(self):
        result = grade('a = 10\nb = 20\nc = a + b\nprint("Hello world!")\n')
        self.assertTrue(result['passed'])
        self.assertTrue(any('UNUSED_CALCULATION' in x for x in result['quality_reasons']))
        self.assertNotEqual(result['rank'], 'S++')

    def test_live_assignment_not_penalized(self):
        result = grade('msg = "Hello world!"\nprint(msg)\n')
        self.assertEqual(result['rank'], 'S++')

    def test_input_has_side_effect_not_dead(self):
        result = grade('a = input()\nprint("Hello world!")\n')
        self.assertFalse(any('UNUSED_CALCULATION' in x for x in result['quality_reasons']))

    def test_required_computation_does_not_remove_hp_for_correct_answer(self):
        result = grade('print(2550)\n', task='for4_kemerovo', expected='2550', computation_required=True)
        self.assertTrue(result['passed'])


if __name__ == '__main__':
    unittest.main()

# State-based tasks are graded by their validator and must not receive an
# output-only liveness penalty for variables intentionally left in state.
class StateContractTests(unittest.TestCase):
    def test_chest_state_assignments_not_deducted(self):
        source = 'a = 10\nb = "hello"\na, b = b, a\n'
        result = grade(source, task='chest')
        self.assertTrue(result['passed'])
        self.assertFalse(any('UNUSED_CALCULATION' in reason
                             for reason in result['quality_reasons']))
