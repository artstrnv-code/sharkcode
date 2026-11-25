"""Специальные проверки региона «Разрушенные крыши».

Использует существующий внешний Sandbox Core; не является изоляцией Python.
В каждом тесте выполняется пользовательский код заново.
"""
from __future__ import annotations

import ast
import builtins
import contextlib
import io
import random
from collections import Counter
from typing import Any

# Только генераторы значений: seed() и shuffle() сами значения не возвращают.
GENERATORS = (
    "randint", "randrange", "random", "uniform", "choice", "choices", "sample",
    "getrandbits", "randbytes", "triangular", "betavariate", "expovariate",
    "gammavariate", "gauss", "normalvariate", "lognormvariate",
    "vonmisesvariate", "paretovariate", "weibullvariate", "binomialvariate",
)


def execute(code: Any, inputs: list[str], record: bool = False) -> tuple[str, str | None, list[Any], list[tuple[list, list]]]:
    """Временно наблюдаем результаты random и перемешивания, не меняя код игрока."""
    values = iter(inputs)
    output = io.StringIO()
    results: list[Any] = []
    shuffles: list[tuple[list, list]] = []
    originals: dict[str, Any] = {}
    old_input = builtins.input

    def reader(prompt=""):
        if prompt:
            print(prompt, end="")
        try:
            return next(values)
        except StopIteration:
            raise EOFError("Ввод закончился") from None

    def wrap_value(fn):
        def wrapped(*args, **kwargs):
            result = fn(*args, **kwargs)
            results.append(result)
            return result
        return wrapped

    def wrap_shuffle(fn):
        def wrapped(seq, *args, **kwargs):
            before = list(seq)
            result = fn(seq, *args, **kwargs)
            shuffles.append((before, list(seq)))
            return result
        return wrapped

    # Полный возврат исходных функций обязателен: judge работает с несколькими тестами.
    try:
        builtins.input = reader
        if record:
            for name in GENERATORS + ("shuffle",):
                if not hasattr(random, name):
                    continue
                original = getattr(random, name)
                originals[name] = original
                setattr(random, name, wrap_shuffle(original) if name == "shuffle" else wrap_value(original))
        with contextlib.redirect_stdout(output):
            exec(code, {"__name__": "__main__"})
    except BaseException as exc:
        return output.getvalue(), f"{type(exc).__name__}: {exc}", results, shuffles
    finally:
        builtins.input = old_input
        for name, fn in originals.items():
            setattr(random, name, fn)
    return output.getvalue(), None, results, shuffles


def check_balls(code: Any, case: dict) -> tuple[bool, str, str, str | None]:
    raw = case["inputs"]
    colors = raw[:raw.index("")] if "" in raw else raw
    output, error, _, shuffles = execute(code, raw, record=True)
    if error:
        return False, "Ошибка выполнения.", output, error
    try:
        answer = int(output.strip())
    except ValueError:
        return False, "Нужно вывести одно целое число совпадений.", output, None
    # Сравниваем именно с тем списком, который программа передала в random.shuffle.
    matching = [after for before, after in shuffles if before == colors]
    if not matching:
        return False, "Нужно перемешать введённые цвета через random.shuffle().", output, None
    expected = sum(a == b for a, b in zip(colors, matching[-1]))
    return answer == expected, "Посчитай совпавшие позиции после перемешивания.", output, None


def check_hero(code: Any) -> tuple[bool, str, str, str | None]:
    # Несколько независимых запусков исключают случайное совпадение в одном тесте.
    for _ in range(4):
        output, error, results, _ = execute(code, [], record=True)
        if error:
            return False, "Ошибка выполнения.", output, error
        if output:
            return False, "Ничего выводить не нужно.", output, None
        if not any(sum(type(value) is type(other) and value == other
                       for other in results[i:]) >= 3
                   for i, value in enumerate(results)):
            return False, "Нужны три одинаковых результата вызовов random за один запуск.", output, None
    return True, "", "", None


def check_boss(code: Any, tree: ast.Module) -> tuple[bool, str, str, str | None]:
    nodes = list(ast.walk(tree))
    aliases = {alias.asname or alias.name for node in nodes if isinstance(node, ast.ImportFrom)
               and node.module == "random" for alias in node.names}
    random_names = {alias.asname or alias.name for node in nodes if isinstance(node, ast.Import)
                    for alias in node.names if alias.name == "random"}
    for node in nodes:
        if isinstance(node, ast.Call) and (
            isinstance(node.func, ast.Name) and node.func.id in aliases
            or isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name)
            and node.func.value.id in random_names
        ):
            return False, "Функции библиотеки random здесь запрещены.", "", None
    # Признаки собственного алгоритма. Не навязываем конкретный генератор.
    ops = [node.op for node in nodes if isinstance(node, ast.BinOp)]
    if not any(isinstance(op, (ast.Mult, ast.Pow, ast.BitXor, ast.LShift, ast.RShift)) for op in ops):
        return False, "Нужен собственный алгоритм преобразования сида.", "", None
    if not any(isinstance(op, (ast.Mod, ast.FloorDiv, ast.Div, ast.BitXor, ast.BitAnd, ast.RShift)) for op in ops):
        return False, "Добавь преобразование или смешивание полученного числа.", "", None

    # Повторные и разные входы, включая отрицательные сиды и границы.
    cases = [(0, 100000, seed) for seed in range(-23, 24, 2)]
    cases += [(-50, 75, -82), (11, 11, 777), (1, 2, 123), (-1000, 1000, 1),
              (100, 200, 993), (-500, -200, 4001)]
    values = []
    last_output = ""
    for low, high, seed in cases:
        args = [str(low), str(high), str(seed)]
        first, error, _, _ = execute(code, args)
        if error:
            return False, f"Ошибка на сиде {seed}.", first, error
        second, repeat_error, _, _ = execute(code, args)
        if repeat_error or first != second:
            return False, "Одинаковые входные данные должны давать одинаковый ответ.", first, repeat_error
        try:
            raw = first.strip()
            if not raw or raw.lstrip("-").isdigit() is False:
                raise ValueError()
            number = int(raw)
        except ValueError:
            return False, "Выведи ровно одно целое число.", first, None
        if not low <= number <= high:
            return False, "Число вышло за включительные границы.", first, None
        if low == 0 and high == 100000:
            values.append(number)
        last_output = first
    if len(set(values)) < 10:
        return False, "Разные сиды дают слишком мало разных результатов.", last_output, None
    # Не допускаем простого копирования нижней границы и других констант.
    if max(Counter(values).values()) > 8:
        return False, "Распределение слишком часто повторяет один результат.", last_output, None
    return True, "", last_output, None


def judge(request: dict) -> dict:
    task = request["task"]
    source = request["source"]
    try:
        tree = ast.parse(source)
        code = compile(tree, "<SHARKCODE>", "exec")
    except (SyntaxError, ValueError) as exc:
        return {"evaluation": {"passed": False, "rank": None, "score": 0,
                               "diagnostics": [str(exc)]}, "runs": [], "analysis": {}}
    runs: list[dict] = []
    diagnostics: list[str] = []
    if task == "roof_balls":
        cases = request.get("tests") or []
        for index, case in enumerate(cases):
            passed, message, output, error = check_balls(code, case)
            runs.append({"inputs": case["inputs"], "stdout": output, "error": error, "passed": passed})
            if not passed:
                diagnostics.append(f"Тест {index + 1}: {error or message}")
    elif task == "roof_hero":
        passed, message, output, error = check_hero(code)
        runs.append({"inputs": [], "stdout": output, "error": error, "passed": passed})
        if not passed:
            diagnostics.append(error or message)
    elif task == "roof_boss":
        passed, message, output, error = check_boss(code, tree)
        runs.append({"inputs": [], "stdout": output, "error": error, "passed": passed})
        if not passed:
            diagnostics.append(error or message)
    else:
        diagnostics.append("Неизвестное испытание Разрушенных крыш.")
    passed = bool(runs) and all(run["passed"] for run in runs)
    score = max(0, 100 - (7 if not source.endswith("\n") else 0))
    if task == "roof_hero" and passed:
        # Краткость влияет только на ранг, а не на факт победы.
        meaningful_lines = sum(bool(line.strip()) for line in source.splitlines())
        score -= min(30, max(0, meaningful_lines - 3) * 4)
    rank = ("S++" if score >= 99 else "S" if score >= 90 else "A" if score >= 76
            else "B" if score >= 55 else "C") if passed else None
    return {"evaluation": {"passed": passed, "rank": rank,
                           "score": score if passed else 0,
                           "tests_passed": sum(run["passed"] for run in runs),
                           "diagnostics": diagnostics,
                           "performance": {"execution_ns": 0, "peak_python_bytes": 0}},
            "runs": runs, "analysis": {}}
