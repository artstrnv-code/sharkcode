"""SHARKCODE runner v0.2.6: AST + repeated CPython tests + final-state snapshots.

Input: one JSON request on stdin. Output: one JSON report on stdout.
This is NOT a security sandbox: never run untrusted third-party code with it.
"""
from __future__ import annotations

import ast
import builtins
import contextlib
import io
import json
import importlib.util
from pathlib import Path
import re
import tokenize
import sys
import traceback
import time
import tracemalloc
import statistics
from typing import Any


def load_local_quality_module():
    """Load the bundled quality checker by file, even under Python's -I mode.

    -I deliberately removes the script directory from sys.path. Do not undo
    that isolation globally: resolve only our trusted sibling quality.py.
    """
    module_path = Path(__file__).resolve().with_name('quality.py')
    spec = importlib.util.spec_from_file_location('sharkcode_quality', module_path)
    if spec is None or spec.loader is None:
        raise ImportError(f'Не найден анализатор качества: {module_path}')
    module = importlib.util.module_from_spec(spec)
    # dataclasses (and similar decorators) look up the loading module by name.
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception:
        sys.modules.pop(spec.name, None)
        raise
    return module


quality = load_local_quality_module()


class DataInput:
    def __init__(self, values: list[str]):
        self._values = iter(values)

    def __call__(self, prompt: str = "") -> str:
        return next(self._values)


def analyze(source: str) -> dict[str, Any]:
    tree = ast.parse(source)
    assignments: set[str] = set()
    reads: set[str] = set()
    spans: dict[str, list[int]] = {}
    imported_hello = False
    hello_main = False
    for node in ast.walk(tree):
        if isinstance(node, ast.Name):
            if isinstance(node.ctx, (ast.Store, ast.Del)):
                assignments.add(node.id)
                spans.setdefault(node.id, []).append(node.lineno)
            elif isinstance(node.ctx, ast.Load):
                reads.add(node.id)
        elif isinstance(node, ast.Import):
            imported_hello |= any(alias.name == '__hello__' for alias in node.names)
        elif isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
            hello_main |= (isinstance(node.func.value, ast.Name) and
                           node.func.value.id == '__hello__' and node.func.attr == 'main')
    return {
        'created_names': sorted(assignments),
        'potentially_unused': sorted(assignments - reads),
        'assignment_lines': spans,
        'grot_trigger': imported_hello and hello_main,
        'ast_nodes': sum(1 for _ in ast.walk(tree)),
    }


def check_pep8(source: str, starter_source: str = "") -> list[str]:
    """Локальные проверки основных правил PEP 8, без сторонних зависимостей.

    Это *не полный* PEP 8 linter: выводим только правила, которые
    действительно проверяются. Дальнейшее подключение pycodestyle/ruff
    потребует отдельной интеграции.
    """
    problems: list[str] = []
    # Player cannot be penalized for the untouched prefix supplied by the game.
    protected_lines: set[int] = set()
    if starter_source and source.startswith(starter_source):
        # The supplied prefix may end midway through an indented line.
        # That line is written by the player too and must still be checked.
        complete_lines = starter_source.count('\n')
        protected_lines = set(range(1, complete_lines + 1))
    lines = source.splitlines()
    if source and not source.endswith(('\n', '\r')):
        problems.append('PEP8 W292: отсутствует перевод строки в конце файла')
    for number, line in enumerate(lines, 1):
        if number in protected_lines:
            continue
        prefix = line[:len(line)-len(line.lstrip(' \t'))]
        if '\t' in prefix:
            problems.append(f'PEP8 W191, строка {number}: табуляция в отступе')
        if prefix.replace('\t', '') and len(prefix) % 4:
            problems.append(f'PEP8 E111, строка {number}: отступ не кратен 4 пробелам')
        if line.rstrip(' \t') != line:
            problems.append(f'PEP8 W291, строка {number}: пробелы в конце строки')
        if len(line) > 79:
            problems.append(f'PEP8 E501, строка {number}: длина строки {len(line)} > 79')
    # Токенизация позволяет не путать оператор присваивания с текстом строки.
    try:
        tokens = list(tokenize.generate_tokens(io.StringIO(source).readline))
    except (tokenize.TokenError, IndentationError):
        return problems
    for token in tokens:
        if token.type != tokenize.OP or token.string not in ('=', '+', '-', '*', '/', '==', '!=', '<=', '>='):
            continue
        row, col = token.start
        if row > len(lines) or row in protected_lines:
            continue
        line = lines[row-1]
        end = token.end[1]
        # Default-параметры функций и keyword arguments имеют собственные
        # правила пробелов; для них не делаем ложных замечаний.
        if token.string in ('*', '-', '+') and (not col or line[col-1] in '([,:='):
            # Unary unpacking, *args and unary signs are not binary operators.
            continue
        if token.string == '=' and ('def ' in line or re.search(r'\w+\s*\(.*\w\s*=', line)):
            continue
        if col and end < len(line) and not line[col-1].isspace() and not line[end].isspace():
            problems.append(f'PEP8 E225, строка {row}: пробелы вокруг {token.string}')
    return problems


def has_temporary_overhead(analysis: dict, task: str) -> bool:
    # У сундука временная переменная допустима, но не идеальна по контексту.
    return bool(set(analysis['potentially_unused']) - {'_'}) or (task == 'chest' and
        len(analysis['created_names']) > 2)


def contextual_excellence(task: str, tree: ast.Module) -> bool:
    """Бонус за значимую для задачи конструкцию, а не число строк."""
    nodes = list(ast.walk(tree))
    if task == 'chest':
        return any(isinstance(n, ast.Assign) and any(
            isinstance(t, ast.Tuple) and [getattr(x, 'id', None) for x in t.elts] == ['a','b']
            for t in n.targets) and isinstance(n.value, ast.Tuple) and
            [getattr(x,'id',None) for x in n.value.elts] == ['b','a'] for n in nodes)
    if task == 'egg':
        return any(isinstance(n, ast.BinOp) and isinstance(n.op, ast.Mult)
                   for n in nodes)
    if task == 'statue':
        return any(isinstance(n, ast.BinOp) and isinstance(n.op, ast.Pow)
                   for n in nodes)
    if task == 'river_cane':
        return any(isinstance(n, ast.BoolOp) and isinstance(n.op, ast.And) for n in nodes)
    if task == 'river_secret':
        return any(isinstance(n, (ast.For, ast.While)) for n in nodes)
    if task == 'river_boss':
        return any(isinstance(n, ast.If) for n in nodes)
    if task == 'river_arcarh_elite':
        return any(isinstance(n, ast.Try) for n in nodes)
    if task == 'echo':
        return any(isinstance(n, ast.BinOp) and isinstance(n.op, ast.Mult)
                   for n in nodes)
    if task in ('counter','wise'):
        return any(isinstance(n, ast.BinOp) for n in nodes)
    return False


def forbids_accountant_conversion_outside_loop(tree: ast.Module) -> bool:
    """Original Счетовод constraint: no int(input()) / float(input())
    outside any for or while body. Other conversions remain allowed.
    """
    class Find(ast.NodeVisitor):
        def __init__(self):
            self.loop_depth = 0
            self.bad = False

        def visit_For(self, node):
            self.loop_depth += 1
            self.generic_visit(node)
            self.loop_depth -= 1

        visit_AsyncFor = visit_For
        visit_While = visit_For

        def visit_Call(self, node):
            outer = isinstance(node.func, ast.Name) and node.func.id in ('int', 'float')
            inner = any(isinstance(arg, ast.Call) and
                isinstance(arg.func, ast.Name) and arg.func.id == 'input'
                for arg in node.args)
            if outer and inner and self.loop_depth == 0:
                self.bad = True
            self.generic_visit(node)

    finder = Find()
    finder.visit(tree)
    return finder.bad


def initial_chest_assignments(tree: ast.Module) -> bool:
    """Require explicit starting a=10 and b='hello', in either order.
    Later overwrites may exist. In v0.1 the swap itself is verified by
    resulting state; robust provenance tracking is planned separately.
    """
    positions: dict[str, int] = {}
    for index, statement in enumerate(tree.body):
        if isinstance(statement, ast.Assign) and len(statement.targets) == 1:
            target = statement.targets[0]
            if isinstance(target, ast.Name) and target.id not in positions:
                try:
                    value = ast.literal_eval(statement.value)
                except (ValueError, TypeError, SyntaxError, MemoryError, RecursionError):
                    continue
                if ((target.id == 'a' and type(value) is int and value == 10) or
                        (target.id == 'b' and type(value) is str and value == 'hello')):
                    positions[target.id] = index
    return len(positions) == 2



def chest_has_real_swap(tree: ast.Module) -> bool:
    """Track origin labels through simple assignments in the introductory chest.

    a starts with origin A, b starts with origin B. A successful exchange
    needs both origins to cross, not just matching final literal values.
    The v0.1 analyzer conservatively handles Name, constants, tuples,
    binary expressions and basic assignment; complex flows need CFG later.
    """
    origins: dict[str, set[str]] = {}

    def read_origin(node: ast.AST) -> set[str]:
        if isinstance(node, ast.Name):
            return origins.get(node.id, set()).copy()
        if isinstance(node, ast.Tuple):
            result = set()
            for part in node.elts:
                result.update(read_origin(part))
            return result
        if isinstance(node, ast.BinOp):
            return read_origin(node.left) | read_origin(node.right)
        return set()

    seen_initial: set[str] = set()
    for statement in tree.body:
        if not isinstance(statement, (ast.Assign, ast.AnnAssign)):
            continue
        targets = statement.targets if isinstance(statement, ast.Assign) else [statement.target]
        value = statement.value
        if value is None:
            continue
        # RHS values must be captured before any target is changed.
        values = ([read_origin(part) for part in value.elts]
                  if isinstance(value, ast.Tuple) else [read_origin(value)])
        if len(targets) == 1 and isinstance(targets[0], ast.Tuple):
            elements = targets[0].elts
            if len(elements) != len(values):
                continue
            pairs = zip(elements, values)
        else:
            pairs = ((target, values[0]) for target in targets)
        for target, provenance in pairs:
            if not isinstance(target, ast.Name):
                continue
            if target.id in ('a', 'b') and target.id not in seen_initial:
                try:
                    literal = ast.literal_eval(value)
                except (ValueError, TypeError, SyntaxError):
                    literal = None
                if (target.id == 'a' and type(literal) is int and literal == 10) or (target.id == 'b' and literal == 'hello'):
                    seen_initial.add(target.id)
                    provenance = {'A' if target.id == 'a' else 'B'}
            origins[target.id] = provenance
    return seen_initial == {'a', 'b'} and origins.get('a') == {'B'} and origins.get('b') == {'A'}


class GrotInstrumentation(ast.NodeTransformer):
    """Record an actually executed __hello__.main() call, not dead AST code."""

    def visit_Call(self, node: ast.Call) -> ast.AST:
        self.generic_visit(node)
        func = node.func
        if (isinstance(func, ast.Attribute) and func.attr == 'main'
                and isinstance(func.value, ast.Name) and func.value.id == '__hello__'):
            return ast.copy_location(ast.Call(
                func=ast.Name(id='__shark_grot_call__', ctx=ast.Load()),
                args=[func, *node.args], keywords=node.keywords), node)
        return node


def compile_traced(source: str):
    tree = ast.parse(source)
    instrumented = GrotInstrumentation().visit(tree)
    ast.fix_missing_locations(instrumented)
    return compile(instrumented, '<sharkcode-snapshot>', 'exec')


class CappedOutput(io.StringIO):
    """Keep memory bounded when the player's code prints too much."""
    def write(self, text: str) -> int:
        original_size = len(text)
        room = max(0, 10000 - self.tell())
        if room:
            super().write(text[:room])
        return original_size

def state_snapshot(namespace: dict[str, Any]) -> dict[str, dict[str, str]]:
    snapshot = {}
    for name, value in namespace.items():
        if name.startswith('__'):
            continue
        # Avoid dangerous / expensive arbitrary repr() on user instances.
        if type(value) in (int, float, bool, str, type(None)):
            serialized = repr(value)[:500]
        else:
            serialized = f'<{type(value).__name__}>'
        snapshot[name] = {'type': type(value).__name__, 'value': serialized}
    return snapshot


def run_case(code, inputs: list[str]) -> dict[str, Any]:
    output = CappedOutput()
    grot_calls = [0]
    def grot_call(func, *args, **kwargs):
        result = func(*args, **kwargs)
        grot_calls[0] += 1
        return result
    namespace = {'__name__': '__main__', '__builtins__': dict(vars(builtins)),
                 '__shark_grot_call__': grot_call}
    namespace['__builtins__']['input'] = DataInput(inputs)
    error = None
    # CPU/execution telemetry excludes compilation and initial interpreter startup.
    # Python allocations are noisy: report peak, but don't treat tiny differences
    # between short educational programs as a meaningful performance defect.
    tracing_owned = not tracemalloc.is_tracing()
    if tracing_owned:
        tracemalloc.start()
    before_current, before_peak = tracemalloc.get_traced_memory()
    started = time.perf_counter_ns()
    with contextlib.redirect_stdout(output):
        try:
            exec(code, namespace, namespace)
        except BaseException as exc:
            error = f'{type(exc).__name__}: {exc}'
    elapsed_ns = time.perf_counter_ns() - started
    after_current, after_peak = tracemalloc.get_traced_memory()
    if tracing_owned:
        tracemalloc.stop()
    peak_delta = max(0, after_peak - before_current)
    return {'stdout': output.getvalue(),
            'elapsed_ns': elapsed_ns, 'peak_python_bytes': peak_delta,
            'grot_call_executed': grot_calls[0] > 0,
            'error': error,
            'state': state_snapshot(namespace)}


def rank_from_score(score: int) -> str:
    """Single rating scale for static quality and performance deductions."""
    return ('S++' if score >= 99 else 'S' if score >= 90
            else 'A' if score >= 76 else 'B' if score >= 55 else 'C')


def _edit_distance_with_limit(actual: str, expected: str, limit: int) -> int:
    """Bounded Levenshtein distance; no fuzzy matching of missing output lines."""
    if abs(len(actual) - len(expected)) > limit:
        return limit + 1
    previous = list(range(len(expected) + 1))
    for index, char in enumerate(actual, 1):
        current = [index]
        for j, wanted in enumerate(expected, 1):
            current.append(min(current[-1] + 1, previous[j] + 1,
                               previous[j - 1] + (char != wanted)))
        previous = current
        if min(previous) > limit:
            return limit + 1
    return previous[-1]


def compare_literal_text(actual: str, expected: str) -> tuple[bool, str | None]:
    """One tiny typo is free; several produce rank caps; large mismatch fails.

    Only use for nodes explicitly opted in as *constant textual output*.
    Do not normalize numbers, line counts, or structured output.
    """
    got, wanted = actual.rstrip('\n'), expected.rstrip('\n')
    if got == wanted:
        return True, None
    if got.count('\n') != wanted.count('\n') or not wanted.strip():
        return False, None
    if len(wanted) < 4 or any(ch.isdigit() for ch in wanted):
        return False, None
    # Protect short phrases: a single edit can be tolerated, but a heavily
    # changed string cannot be accepted simply because both are short.
    limit = min(4, max(1, (len(wanted) + 3) // 4))
    distance = _edit_distance_with_limit(got, wanted, limit)
    if distance > limit:
        return False, None
    if distance == 1:
        return True, 'Мелкая опечатка в константном тексте (без штрафа)'
    if distance == 2 and len(wanted) >= 8:
        return True, 'Опечатки в тексте: ранг не выше A'
    if distance == 3 and len(wanted) >= 12:
        return True, 'Опечатки в тексте: ранг не выше B'
    return False, None


def evaluate(task: str, runs: list[dict[str, Any]], analysis: dict, source: str,
             ranking: dict | None = None, starter_source: str = "",
             computation_required: bool = False, text_tolerance: bool = False) -> dict:
    expected = {
        'shyguy': 'Hello world!',
        'egg': 'бух бух бух ',
        'statue': str(120 ** 11),
        'echo': '\n'.join('бух' * i for i in range(1, 8)),
        'grot_destroyer': 'some evil\nroar\nsad',
    }
    ranking = ranking or {}
    pep8_coef = max(0.0, min(float(ranking.get('pep8_coef', 1.0)), 10.0))
    strokes_coef = max(0.0, min(float(ranking.get('strokes_coef', 1.0)), 10.0))
    hints: list[str] = []
    passed_tests = 0
    typo_caps: list[str] = []
    try:
        tree = ast.parse(source)
    except SyntaxError as exc:
        return {'passed': False, 'rank': None,
                'diagnostics': [str(exc)], 'tests_passed': 0}

    pep8_issues = check_pep8(source, starter_source)
    for index, case in enumerate(runs):
        if task == 'river_mimic_conditions':
            ok = not case['error'] and bool(case['stdout'])
            case['expectation_kind'] = 'rule'
            case['expected_display'] = 'Для всех 81 комбинаций — непустой уникальный вывод'
        elif task == 'for4_mimic_loops':
            import re as _re
            numbers = [int(x) for x in _re.findall(r'(?<!\d)\d+(?!\d)', case['stdout'])]
            ok = not case['error'] and numbers == list(range(1, 101))
            case['expectation_kind'] = 'rule'
            case['expected_display'] = 'Числа 1–100 по порядку (форма свободная)'
        elif task == 'tower_mimic_surrender':
            expected_lines = ['я не сдамся'] * 100
            if case.get('inputs') == ['да']:
                expected_lines.append('сдаюсь')
            ok = not case['error'] and case['stdout'].splitlines() == expected_lines
            case['expectation_kind'] = 'rule'
            case['expected_display'] = '100 строк и условное «сдаюсь»'
        elif task == 'tower_novice':
            # Original task asks for conversation on 'нет', but the final
            # required answer is always 'ура!'. Earlier dialogue is allowed.
            lines = case['stdout'].strip().splitlines()
            ok = not case['error'] and bool(lines) and lines[-1] == 'ура!' and lines.count('ура!') == 1
            case['expected_display'] = 'Сообщения на «нет», затем «ура!»'
            if not ok:
                hints.append('Послушник: при «да» заверши цикл и скажи «ура!».')
        elif task == 'for4_carousel':
            from collections import Counter
            counts = Counter(case['stdout'].split())
            ok = not case['error'] and counts == {'привет': 9, 'пока': 6, 'ппаоа': 3}
            case['expectation_kind'] = 'rule'
            case['expected_display'] = '9×привет; 6×пока; 3×ппаоа'
            if not ok:
                hints.append('Карусели: требуется ровно 9 привет, 6 пока, 3 ппаоа; порядок свободный.')
        elif task == 'chest':
            values = case['state']
            ok = (not case['error'] and not case['stdout'] and
                  values.get('a') == {'type':'str', 'value':"'hello'"} and
                  values.get('b') == {'type':'int','value':'10'} and
                  initial_chest_assignments(tree) and chest_has_real_swap(tree))
            case['expectation_kind'] = 'rule'
            case['expected_display'] = ''
            if not ok:
                hints.append('Сундук: нужны исходные a=10 и b="hello", обмен и отсутствие вывода.')
        else:
            target = case.get('expected', expected.get(task))
            if target is None:
                hints.append(f'Для задачи {task} отсутствует контракт проверки')
                ok = False
            else:
                # Сравнение сохраняет внутренние пробелы, но разрешает один
                # заключительный перенос строки от print().
                ok = not case['error'] and case['stdout'].rstrip('\n') == str(target)
                # Original elite arithmetic accepts both `4` and `4.0` for a
                # numerical quotient. Only this enemy uses numeric equivalence.
                if not ok and task in ('river_arcarh_elite', 'field_median') and not case['error']:
                    try:
                        ok = float(case['stdout'].strip()) == float(target)
                    except (ValueError, OverflowError):
                        pass
                if (not ok and text_tolerance and not case['error'] and
                    not case.get('inputs') and isinstance(target, str)):
                    ok, notice = compare_literal_text(case['stdout'], target)
                    if notice:
                        hints.append(f'Тест {index + 1}: {notice}')
                        if 'не выше A' in notice:
                            typo_caps.append('A')
                        elif 'не выше B' in notice:
                            typo_caps.append('B')
                case['expected_display'] = str(target) + '\n' 
        case['passed'] = bool(ok)
        passed_tests += int(bool(ok))
        if case['error']:
            hints.append(f'Тест {index+1}: {case["error"]}')
        if not ok and task != 'chest' and case.get('expected_display') is not None:
            hints.append(f'Тест {index+1}: результат не совпадает с ожидаемым')

    hints.extend(pep8_issues)
    unused = [name for name in analysis['potentially_unused'] if name != '_']
    if unused:
        hints.append('Возможно неиспользуемые имена: ' + ', '.join(unused))
    passed = bool(runs) and passed_tests == len(runs)
    if task == 'for4_accountant' and forbids_accountant_conversion_outside_loop(tree):
        passed = False
        hints.append('Счетовод: int(input()) и float(input()) вне цикла запрещены.')
    no_overhead = not has_temporary_overhead(analysis, task)
    no_bad_practice = not any(
        isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id in ('eval','exec')
        for n in ast.walk(tree))
    pep8_ok = not pep8_issues
    excellence = contextual_excellence(task, tree)
    # v3: ranking by confirmed shortcomings, never by similarity to one template.
    # A working one-line solution stays S++ if it genuinely has no issues.
    quality_reasons: list[str] = []
    score = 100 if passed else 0
    meaningful_lines = [line for line in source.splitlines()
                        if line.strip() and not line.lstrip().startswith('#')]
    findings = quality.inspect(source, starter_source)
    # The chest explicitly grades final variable state, not only printed output.
    if task == 'chest':
        findings = [finding for finding in findings
                    if finding.code != 'UNUSED_CALCULATION']
    if passed:
        if pep8_issues:
            cost = min(24, round(len(pep8_issues) * 7 * pep8_coef))
            score -= cost
            quality_reasons.append(f'Стиль PEP 8: {len(pep8_issues)} замечаний (−{cost})')
        if not no_bad_practice:
            score -= 26
            quality_reasons.append('Используются eval/exec без необходимости (−26)')
        # Unused variables are suggestions until scope-aware data flow can prove
        # them redundant. Do not punish legitimate loop counters or API names.
        if not no_overhead:
            quality_reasons.append('Возможные неиспользуемые имена: требуется проверка контекста (без штрафа)')
        if task == 'river_mimic_conditions':
            uses_else = any(
                isinstance(node, (ast.If, ast.For, ast.While, ast.Try))
                and bool(node.orelse)
                and not (isinstance(node, ast.If) and len(node.orelse) == 1
                         and isinstance(node.orelse[0], ast.If))
                for node in ast.walk(tree))
            if uses_else:
                score -= 17
                quality_reasons.append('Повелитель условий: использование else (−17)')
        if computation_required and quality.numeric_answer_hardcoded(source):
            score -= 24
            quality_reasons.append(
                'Вычислительная задача: готовый числовой ответ вместо вычисления (−24)')
        for finding in findings:
            score -= finding.penalty
            quality_reasons.append(
                f'{finding.code}, строка {finding.line}: {finding.message} (−{finding.penalty})')
        # A genuinely excessive number of independently written branches is a
        # maintainability problem. A small amount of branching is not.
        if_count = sum(isinstance(node, ast.If) for node in ast.walk(tree))
        if if_count > 12:
            cost = min(28, (if_count - 12) * 2)
            score -= cost
            quality_reasons.append(f'Избыточное число ветвлений if: {if_count} (−{cost})')
        # Report exact repeated statements only in blocks and only beyond two;
        # repeated output is often required by a beginner's task. Do not treat
        # this heuristic as a proof of a bad algorithm.
        duplicate_statements = 0
        for block in [tree, *[n for n in ast.walk(tree)
                               if isinstance(n, (ast.If, ast.For, ast.While, ast.With, ast.Try))]]:
            body = getattr(block, 'body', [])
            seen: dict[str, int] = {}
            for stmt in body:
                if isinstance(stmt, (ast.Expr, ast.Pass)):
                    continue
                key = ast.dump(stmt, include_attributes=False)
                seen[key] = seen.get(key, 0) + 1
            duplicate_statements += sum(max(0, count - 2) for count in seen.values())
        if duplicate_statements:
            cost = min(20, 5 * duplicate_statements)
            score -= cost
            quality_reasons.append(f'Многократно дублируются инструкции: {duplicate_statements} (−{cost})')
        if task == 'tower_garaon' and any(
                isinstance(n, ast.BinOp) and isinstance(n.op, ast.Mult)
                for n in ast.walk(tree)):
            quality_reasons.append('Допустимое решение Гараона: повторение строки вместо дополнительных итераций')
        score = max(0, min(100, score))
        rank = rank_from_score(score)
        if typo_caps:
            limit = 'B' if 'B' in typo_caps else 'A'
            before_cap = score
            score = min(score, 65 if limit == 'B' else 85)
            if score != before_cap:
                quality_reasons.append(f'Опечатки в константном тексте: −{before_cap - score} баллов')
            rank = rank_from_score(score)
            rank_order = ('S++', 'S', 'A', 'B', 'C')
            limit = 'B' if 'B' in typo_caps else 'A'
            if rank_order.index(rank) < rank_order.index(limit):
                rank = limit
            quality_reasons.append(f'Допуск опечаток: максимальный ранг {limit}')
        if rank != 'S++':
            quality_reasons.append(f'До S++: {100 - score} баллов; причины перечислены выше')
        else:
            quality_reasons.append('S++: проверенные критерии выполнены; существенных дефектов не обнаружено')
    else:
        rank = None
    hints.extend(quality_reasons)
    # Время и память измеряются отдельно. Микрозадачи слишком короткие для
    # достоверного ранжирования по единичному запуску: без benchmark-эталона
    # эти данные не изменяют ранг.
    elapsed = [case.get('elapsed_ns', 0) for case in runs]
    memory = [case.get('peak_python_bytes', 0) for case in runs]
    performance = {'execution_ns': elapsed, 'peak_python_bytes': memory,
                   'ranking_applied': False,
                   'reason': 'Сырой замер. Без повторных сравнительных запусков влияние на ранг отключено'}
    bonus = 0
    return {'passed': passed, 'rank': rank, 'score': score,
            'tests_passed': passed_tests, 'diagnostics': hints,
            'pep8_ok': pep8_ok, 'pep8_issues': pep8_issues,
            'no_overhead': no_overhead, 'no_bad_practice': no_bad_practice,
            'contextual_excellence': excellence, 'contextual_bonus': bonus,
            'typo_rank_cap': ('B' if 'B' in typo_caps else 'A' if 'A' in typo_caps else None),
            'meaningful_lines': len(meaningful_lines), 'quality_reasons': quality_reasons,
            'performance': performance}


# Trustworthy reference implementations used for conservative runtime comparisons.
# Only tasks with a prescribed, exactly comparable stdout are benchmarked.
# These are examples, not mandatory templates for S++.
BENCHMARK_REFERENCES = {
    'shyguy': 'print("Hello world!")\n',
    'for4_kemerovo': 'print(sum(range(2, 101, 2)))\n',
    'for4_boss': (
        'for a in range(1, 21):\n'
        '    for b in range(1, 21):\n'
        '        print(a * b)\n'),
    'tower_mimic_surrender': (
        'for _ in range(100):\n'
        '    print("я не сдамся")\n'
        'if input("ты еще настаиваешь?") == "да":\n'
        '    print("сдаюсь")\n'),
}


def compare_performance(task: str, user_code, first_run: dict,
                        test_inputs: list[str]) -> dict[str, Any]:
    """Compare medians, not a single stopwatch tick.

    Reruns apply only to deterministic educational challenges with a trusted
    reference. Uncertain timings never penalise the user. The existing
    execution contract is explicitly NOT an OS sandbox.
    """
    reference = BENCHMARK_REFERENCES.get(task)
    if not reference:
        return {'ranking_applied': False, 'reason': 'Нет сопоставимого эталона'}
    try:
        reference_code = compile_traced(reference)
        reference_runs = [run_case(reference_code, test_inputs) for _ in range(3)]
        if any(run['error'] for run in reference_runs):
            return {'ranking_applied': False, 'reason': 'Эталон не прошёл выполнение'}
        if reference_runs[0]['stdout'] != first_run['stdout']:
            return {'ranking_applied': False,
                    'reason': 'Разные форматы вывода; сравнение производительности некорректно'}
        user_runs = [first_run] + [run_case(user_code, test_inputs) for _ in range(2)]
        if any(run['error'] or run['stdout'] != first_run['stdout'] for run in user_runs):
            return {'ranking_applied': False,
                    'reason': 'Повторные тесты дали разные результаты; сравнение отключено'}
        user_ns = [run['elapsed_ns'] for run in user_runs]
        reference_ns = [run['elapsed_ns'] for run in reference_runs]
        user_memory = [run['peak_python_bytes'] for run in user_runs]
        reference_memory = [run['peak_python_bytes'] for run in reference_runs]
        user_median = statistics.median(user_ns)
        ref_median = statistics.median(reference_ns)
        user_peak = statistics.median(user_memory)
        ref_peak = statistics.median(reference_memory)
        # Cannot reliably rank sub-millisecond workloads or unstable samples.
        stable = (min(user_ns + reference_ns) > 0 and
                  max(user_ns) / min(user_ns) <= 1.8 and
                  max(reference_ns) / min(reference_ns) <= 1.8)
        time_disadvantage = (stable and ref_median >= 1_000_000 and
            user_median - ref_median > 1_500_000 and user_median > ref_median * 2.2)
        memory_disadvantage = (ref_peak >= 64 * 1024 and
            user_peak - ref_peak > 128 * 1024 and user_peak > ref_peak * 3)
        return {'ranking_applied': bool(time_disadvantage or memory_disadvantage),
                'time_disadvantage': bool(time_disadvantage),
                'memory_disadvantage': bool(memory_disadvantage),
                'user_median_ns': int(user_median), 'reference_median_ns': int(ref_median),
                'user_peak_bytes': int(user_peak), 'reference_peak_bytes': int(ref_peak),
                'stable': stable,
                'reason': 'Существенный подтверждённый проигрыш' if (time_disadvantage or memory_disadvantage)
                           else 'Малая или нестабильная разница; штраф не применяется'}
    except Exception as exc:
        return {'ranking_applied': False,
                'reason': f'Невозможно надёжно сравнить: {type(exc).__name__}'}


def run_external_validator(filename: str, context: dict[str, Any]) -> dict[str, Any]:
    """Подключаем только валидаторы из доверенного каталога игры.

    Это не sandbox. Подключение сторонних скриптов из mods/ запрещено,
    пока не появится изолированный исполнитель.
    """
    if not filename:
        return {}
    if Path(filename).name != filename or not filename.endswith('.py'):
        return {'accepted': False, 'infrastructure_error': True, 'diagnostics': ['Некорректное имя валидатора']}
    path = Path(__file__).resolve().parent.parent / 'content' / 'validators' / filename
    if not path.is_file():
        return {'accepted': False, 'infrastructure_error': True, 'diagnostics': [f'Не найден валидатор: {filename}']}
    try:
        spec = importlib.util.spec_from_file_location('shark_validator', path)
        if spec is None or spec.loader is None:
            raise ImportError(f'Невозможно загрузить {filename}')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        response = module.validate(context)
        if not isinstance(response, dict):
            raise TypeError('validate() должен возвращать словарь')
        return response
    except Exception as exc:
        return {'accepted': False, 'infrastructure_error': True, 'diagnostics': [f'Ошибка валидатора {filename}: {type(exc).__name__}: {exc}']}


def main() -> None:
    request = json.load(sys.stdin)
    if request.get('task') == 'grot_boss':
        # Resolve a trusted validator from the node's TOML reference.
        # User-installed mods are not permitted to inject executable validators.
        validator = request.get('validator', 'grot_boss.py') or 'grot_boss.py'
        if validator != 'grot_boss.py':
            json.dump({'evaluation': {'passed': False, 'rank': None,
                       'diagnostics': ['Unknown project validator']},
                       'runs': [], 'analysis': {}}, sys.stdout)
            return
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        validator_path = Path(__file__).resolve().parents[1] / 'content' / 'validators' / validator
        spec = importlib.util.spec_from_file_location('shark_grot_boss_validator', validator_path)
        if spec is None or spec.loader is None:
            raise RuntimeError('Project validator unavailable')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        result = module.validate({'project_files': request.get('project_files', {})})
        json.dump(result, sys.stdout, ensure_ascii=True)
        return
    source = request['source']
    task = request['task']
    # Region 0.2.47 uses explicit Python-side contracts; no new Rust executor.
    if task.startswith('roof_'):
        # Новые испытания используют тот же доверенный конвейер проверки.
        spec = importlib.util.spec_from_file_location('shark_judge_roofs',
            Path(__file__).resolve().with_name('judge_roofs.py'))
        if spec is None or spec.loader is None:
            raise RuntimeError('roof judge unavailable')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        json.dump(module.judge(request), sys.stdout, ensure_ascii=True)
        return
    if task.startswith('if_'):
        spec = importlib.util.spec_from_file_location('shark_judge051',
            Path(__file__).resolve().with_name('judge051.py'))
        if spec is None or spec.loader is None:
            raise RuntimeError('0.2.51 judge missing')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        json.dump(module.judge(request), sys.stdout, ensure_ascii=True)
        return
    if task.startswith(('mills_', 'project_')) or task == 'field_new_chest':
        spec = importlib.util.spec_from_file_location('shark_judge047',
            Path(__file__).resolve().with_name('judge047.py'))
        if spec is None or spec.loader is None:
            raise RuntimeError('0.2.47 judge missing')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        result = module.judge(request)
        json.dump(result, sys.stdout, ensure_ascii=True)
        return
    if task in ('grot_reader', 'grot_destroyer'):
        # -I ignores PYTHONPATH. Explicitly load only our bundled teaching library.
        # Note: this does NOT isolate the user program from the host OS.
        library_root = Path(__file__).resolve().parents[1] / 'content'
        sys.path.insert(0, str(library_root))
    try:
        analysis = analyze(source)
        code = compile_traced(source)
    except (SyntaxError, ValueError) as exc:
        json.dump({'evaluation': {'passed': False, 'rank': None,
                                   'diagnostics': [f'{type(exc).__name__}: {exc}']},
                   'analysis': {}, 'runs': []}, sys.stdout, ensure_ascii=True)
        return
    tests = request.get('tests') or [{'inputs': []}]
    runs = [run_case(code, case.get('inputs', [])) for case in tests]
    for run, case in zip(runs, tests):
        run['inputs'] = case.get('inputs', [])
    analysis['grot_trigger'] = (analysis['grot_trigger'] and
                                any(run['grot_call_executed'] for run in runs))
    for run, case in zip(runs, tests):
        if 'expected' in case:
            run['expected'] = case['expected']
    result = evaluate(task, runs, analysis, source, request.get('ranking'),
                      request.get('starter_source', ''),
                      bool(request.get('computation_required', False)),
                      bool(request.get('text_tolerance', False)))
    if result['passed'] and runs:
        # Performance is a secondary signal; never penalise noisy microtimings.
        perf = compare_performance(task, code, runs[0], tests[0].get('inputs', []))
        perf['execution_ns'] = result['performance']['execution_ns']
        perf['peak_python_bytes'] = result['performance']['peak_python_bytes']
        result['performance'] = perf
        if perf.get('ranking_applied'):
            deductions = []
            if perf.get('time_disadvantage'):
                deductions.append(('Существенно медленнее проверенного эталона', 12))
            if perf.get('memory_disadvantage'):
                deductions.append(('Существенно больше расход Python-памяти', 12))
            for explanation, cost in deductions:
                result['score'] = max(0, result['score'] - cost)
                result['diagnostics'].append(f'{explanation} (−{cost})')
            value = result['score']
            result['rank'] = ('S++' if value >= 99 else 'S' if value >= 90
                              else 'A' if value >= 76 else 'B' if value >= 55 else 'C')
            if result.get('typo_rank_cap'):
                order = ('S++', 'S', 'A', 'B', 'C')
                cap = result['typo_rank_cap']
                if order.index(result['rank']) < order.index(cap):
                    result['rank'] = cap
            result['diagnostics'].append(f'До S++: {100-value} баллов; включая измеренную эффективность')
    external = run_external_validator(request.get('validator', ''),
        {'task': task, 'source': source, 'runs': runs, 'analysis': analysis,
         'evaluation': result.copy(), 'ranking': request.get('ranking', {})})
    result['diagnostics'].extend(str(x) for x in external.get('diagnostics', []))
    if external.get('infrastructure_error'):
        result['infrastructure_error'] = True
        result['diagnostics'].append('Ошибка инфраструктуры проверки: HP не расходуется')
    # Узловой валидатор может отклонить решение, но не обойти тесты движка.
    if request.get('validator'):
        result['validator_passed'] = external.get('accepted', True) is not False
    if external.get('accepted') is False:
        result['passed'] = False
        result['rank'] = None
        result['diagnostics'].append('Условие специализированного валидатора не выполнено')
    # Внешний Python пока не изолирован: разрешён только доверенный content/validators.
    json.dump({'evaluation': result, 'analysis': analysis, 'runs': runs},
              sys.stdout, ensure_ascii=True)


if __name__ == '__main__':
    main()
