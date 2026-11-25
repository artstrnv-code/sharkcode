"""Регрессия системы рангов и необязательных сундуков SHARKCODE 0.2.39.

Тесты вызывают Python-движок напрямую. Они НЕ заменяют сборку Bevy.
"""
import sys
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'runtime'))
import inspector


def grade(task, source, inputs=None, expected=None):
    inputs = inputs or [[]]
    analysis = inspector.analyze(source)
    compiled = inspector.compile_traced(source)
    cases = [inspector.run_case(compiled, values) for values in inputs]
    for case, values in zip(cases, inputs):
        case['inputs'] = values
        if expected is not None:
            case['expected'] = expected
    assessment = inspector.evaluate(task, cases, analysis, source)
    return cases, assessment


class RankingV2Tests(unittest.TestCase):
    def test_missing_final_newline_is_pep8_issue(self):
        _, missing = grade('shyguy', 'print("Hello world!")')
        _, correct = grade('shyguy', 'print("Hello world!")\n')
        self.assertTrue(missing['passed'])
        self.assertFalse(missing['pep8_ok'])
        self.assertEqual(missing['rank'], 'S')
        self.assertTrue(correct['pep8_ok'])
        self.assertEqual(correct['rank'], 'S++')

    def test_blank_lines_do_not_increase_length(self):
        _, plain = grade('shyguy', 'print("Hello world!")\n')
        _, separated = grade('shyguy', '\n\nprint("Hello world!")\n\n')
        self.assertEqual(plain['meaningful_lines'], separated['meaningful_lines'])

    def test_starter_code_pep8_exemption(self):
        source = 'x = True\nwhile x == True:\n    print("бух\\n" * 5, end="")\n    break\n'
        self.assertEqual(inspector.check_pep8(source, 'x = True\nwhile x == True:\n    '), [])
        bad = source.replace('print(\"бух\\n\" * 5, end=\"\")', 'print(\"бух\\n\"*5, end=\"\")')
        self.assertTrue(inspector.check_pep8(bad, 'x = True\nwhile x == True:\n    '))
        _, result = grade('tower_garaon', source, expected='бух\nбух\nбух\nбух\nбух')
        self.assertEqual(result['rank'], 'S++')

    def test_working_but_poor_code_can_be_c(self):
        source = ('if False:\n    pass\n' * 25) + 'print(3)\n'
        _, result = grade('counter', source, expected='3')
        self.assertTrue(result['passed'])
        self.assertIn(result['rank'], ('B', 'C'))

    def test_no_hardcoded_ast_pattern_for_splusplus(self):
        _, result = grade('for4_mimic_loops', 'print(*range(1, 101))\n')
        self.assertTrue(result['passed'])
        self.assertEqual(result['rank'], 'S++')
        self.assertEqual(result['pep8_issues'], [])

    def test_else_reduces_condition_mimic_rank(self):
        basic = 'print(input(), input(), input(), input())\n'
        with_else = 'if True:\n    print(input(), input(), input(), input())\nelse:\n    pass\n'
        inputs = [[str(a), str(b), str(c), str(d)]
                  for a in range(1, 4) for b in range(1, 4)
                  for c in range(1, 4) for d in range(1, 4)]
        runs_a, rating_a = grade('river_mimic_conditions', basic, inputs)
        runs_b, rating_b = grade('river_mimic_conditions', with_else, inputs)
        self.assertTrue(inspector.run_external_validator('river_mimic_conditions.py', {'runs': runs_a})['accepted'])
        self.assertTrue(inspector.run_external_validator('river_mimic_conditions.py', {'runs': runs_b})['accepted'])
        self.assertEqual(rating_a['rank'], 'S++')
        self.assertNotEqual(rating_b['rank'], 'S++')
        self.assertTrue(any('else' in issue for issue in rating_b['quality_reasons']))

    def test_surrender_checks_both_answers_and_prompt(self):
        source = ('for _ in range(100):\n'
                  '    print("я не сдамся")\n'
                  'if input("ты еще настаиваешь?") == "да":\n'
                  '    print("сдаюсь")\n')
        runs, ev = grade('tower_mimic_surrender', source, [['нет'], ['да']])
        self.assertTrue(ev['passed'])
        self.assertTrue(inspector.run_external_validator('tower_mimic_surrender.py',
                            {'source': source, 'runs': runs})['accepted'])

    def test_specialist_does_not_accept_range_repr(self):
        runs, ev = grade('for4_mimic_loops', 'print(range(1, 101))\n')
        self.assertFalse(ev['passed'])
        result = inspector.run_external_validator('for4_mimic_loops.py',
                          {'source': 'print(range(1, 101))\n', 'runs': runs})
        self.assertFalse(result['accepted'])

    def test_optional_mimics_and_rewards_exist(self):
        for name in ('river_mimic_conditions', 'for4_mimic_loops', 'tower_mimic_surrender'):
            item = tomllib.loads((ROOT / f'content/nodes/{name}.toml').read_text())
            self.assertEqual(item['META']['type'], 'chest.code')
            self.assertTrue((ROOT / 'content/validators' / item['RANKING']['validator']).exists())
        for name in ('justice_sword', 'justice_spear'):
            self.assertTrue((ROOT / f'content/items/{name}.toml').is_file())
        source = (ROOT / 'src/main.rs').read_text()
        region_rules = (ROOT / 'src/regions.rs').read_text()
        self.assertIn('index - 2', region_rules)
        self.assertIn('"save_schema":2', source)
        self.assertIn('CodexPage::Notes0', source)
        self.assertIn('CodexPage::MatchCase', source)


if __name__ == '__main__':
    unittest.main()
