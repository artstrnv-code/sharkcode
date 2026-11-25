"""Validation of the Grot's creative multi-file challenge.

This is a correctness heuristic, not an objective measure of usefulness.
Runs in the existing CPython worker. It is NOT a security sandbox.
"""
from __future__ import annotations

import ast
import contextlib
import importlib.util
import inspect
import io
import json
from pathlib import Path
import sys
import tempfile
import tomllib
from typing import Any

FILES = ('pyproject.toml', 'README.md', '__init__.py', 'core.py')
FUNCTIONS = ('dragon', 'fire', 'python')


def _uses_parameter(node: ast.FunctionDef) -> bool:
    params = {arg.arg for arg in [*node.args.posonlyargs, *node.args.args,
                                   *node.args.kwonlyargs]}
    if node.args.vararg:
        params.add(node.args.vararg.arg)
    if node.args.kwarg:
        params.add(node.args.kwarg.arg)
    return bool(params) and any(isinstance(child, ast.Name) and
                                child.id in params and isinstance(child.ctx, ast.Load)
                                for child in ast.walk(node))


def inspect_project(files: dict[str, str]) -> dict[str, Any]:
    diagnostics: list[str] = []
    if not isinstance(files, dict) or any(not isinstance(files.get(n), str) for n in FILES):
        return _bad('Нужны четыре файла: pyproject.toml, README.md, __init__.py, core.py')
    if sum(len(files[n]) for n in FILES) > 100_000:
        return _bad('Проект слишком большой для учебного задания (100 КБ).')
    try:
        metadata = tomllib.loads(files['pyproject.toml'])
        project = metadata['project']
        build = metadata['build-system']
        name = project['name']
        version = project['version']
        if name != 'name_lib' or not isinstance(version, str):
            return _bad('Имя проекта должно быть name_lib, version — строкой.')
        if build.get('build-backend') != 'hatchling.build' or 'hatchling' not in build.get('requires', []):
            return _bad('Используй Hatchling в build-system (см. скрижаль).')
        if project.get('readme') != 'README.md':
            return _bad('Укажи readme = "README.md".')
    except (tomllib.TOMLDecodeError, KeyError, TypeError, ValueError) as exc:
        return _bad(f'pyproject.toml: {exc}')

    try:
        init_ast = ast.parse(files['__init__.py'])
        core_ast = ast.parse(files['core.py'])
    except SyntaxError as exc:
        return _bad(f'Синтаксическая ошибка Python: {exc}')

    # Explicit exports: star imports permitted, but an import must come from .core.
    exports = [n for n in ast.walk(init_ast) if isinstance(n, ast.ImportFrom)
               and n.level == 1 and n.module == 'core']
    if not exports:
        return _bad('В __init__.py импортируй функции из .core (относительный импорт).')
    functions = {n.name: n for n in core_ast.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef))}
    missing = set(FUNCTIONS) - functions.keys()
    if missing:
        return _bad('Не найдены def: ' + ', '.join(sorted(missing)))
    # `async def` and generators are not plain functions returning a result for the boss.
    if any(isinstance(functions[n], ast.AsyncFunctionDef) for n in FUNCTIONS):
        return _bad('Здесь нужны обычные функции def, не async def.')
    if any(isinstance(n, (ast.Yield, ast.YieldFrom)) for fn in FUNCTIONS for n in ast.walk(functions[fn])):
        return _bad('Функции-генераторы не подходят: ожидается return значения.')
    for name in FUNCTIONS:
        fn = functions[name]
        args = [*fn.args.posonlyargs, *fn.args.args, *fn.args.kwonlyargs]
        if len(args) == 0 and fn.args.vararg is None:
            return _bad(f'{name}() должна принимать аргументы.')
        if not any(isinstance(n, ast.Return) and n.value is not None for n in ast.walk(fn)):
            return _bad(f'{name}() должна возвращать значение через return.')

    calls: list[dict[str, Any]] = []
    with tempfile.TemporaryDirectory(prefix='sharkcode_project_') as root:
        project_path = Path(root)
        package = project_path / 'name_lib'
        package.mkdir()
        (project_path / 'pyproject.toml').write_text(files['pyproject.toml'], encoding='utf-8')
        (project_path / 'README.md').write_text(files['README.md'], encoding='utf-8')
        (package / '__init__.py').write_text(files['__init__.py'], encoding='utf-8')
        (package / 'core.py').write_text(files['core.py'], encoding='utf-8')
        module_key = 'name_lib'
        old = sys.modules.pop(module_key, None)
        old_core = sys.modules.pop('name_lib.core', None)
        try:
            spec = importlib.util.spec_from_file_location(
                module_key, package / '__init__.py',
                submodule_search_locations=[str(package)])
            if not spec or not spec.loader:
                return _bad('Пакет не удалось импортировать.')
            module = importlib.util.module_from_spec(spec)
            sys.modules[module_key] = module
            with contextlib.redirect_stdout(io.StringIO()):
                spec.loader.exec_module(module)
            for name in FUNCTIONS:
                fn = getattr(module, name, None)
                if not callable(fn):
                    return _bad(f'Функция {name} недоступна через import name_lib.')
                sig = inspect.signature(fn)
                positional = [p for p in sig.parameters.values()
                              if p.kind in (inspect.Parameter.POSITIONAL_ONLY,
                                            inspect.Parameter.POSITIONAL_OR_KEYWORD)]
                mandatory = [p for p in positional if p.default is inspect.Parameter.empty]
                mandatory_kw = [p for p in sig.parameters.values()
                                if p.kind == inspect.Parameter.KEYWORD_ONLY
                                and p.default is inspect.Parameter.empty]
                count = max(1, len(mandatory))
                if count > 4 or len(mandatory_kw) > 4:
                    return _bad(f'{name}(): для учебного вызова поддерживается до 4 обязательных аргументов.')
                # Creative tasks can operate on numbers, strings, or booleans.
                # Choose a pair of successful calls of the same type.
                last_error = ''
                successful = None
                for first, second in [(2, 7), ('alpha', 'beta'), (True, False)]:
                    trial = []
                    for value in (first, second):
                        args = (value,) * count
                        kwargs = {p.name: value for p in mandatory_kw}
                        output = io.StringIO()
                        try:
                            with contextlib.redirect_stdout(output):
                                result = fn(*args, **kwargs)
                        except BaseException as exc:
                            last_error = f'{type(exc).__name__}: {exc}'
                            trial = []
                            break
                        if result is None:
                            last_error = 'возвращено None'
                            trial = []
                            break
                        trial.append({'function': name, 'argument': value,
                                      'args': list(args), 'result': repr(result)[:180],
                                      'type': type(result).__name__})
                    if len(trial) == 2:
                        successful = trial
                        break
                if successful is None:
                    return _bad(f'{name}(): не удалось вызвать с двумя вариантами входа: {last_error}')
                calls.extend(successful)
        finally:
            sys.modules.pop(module_key, None)
            sys.modules.pop('name_lib.core', None)
            if old is not None:
                sys.modules[module_key] = old
            if old_core is not None:
                sys.modules['name_lib.core'] = old_core

    # Static and behavioural signals are combined, not treated as proof of usefulness.
    dependency = {name: _uses_parameter(functions[name]) for name in FUNCTIONS}
    varied = {name: calls[2*i]['result'] != calls[2*i+1]['result'] for i,name in enumerate(FUNCTIONS)}
    transformed = {name: all(calls[2*i+j]['result'] != repr(calls[2*i+j]['argument'])
                             for j in (0, 1)) for i, name in enumerate(FUNCTIONS)}
    all_dependent = all(dependency.values()) and all(varied.values()) and all(transformed.values())
    complexity = sum(sum(isinstance(n, (ast.BinOp, ast.BoolOp, ast.If, ast.Compare, ast.Call,
                                       ast.Subscript, ast.IfExp, ast.For)) for n in ast.walk(functions[name]))
                     for name in FUNCTIONS)
    if not any(dependency.values()):
        rank = 'B' if len({c['result'] for c in calls}) <= 3 else 'C'
        diagnostics.append('Все функции игнорируют аргументы. Корректно, но мало пользы.')
    elif all_dependent:
        rank = 'S'
        diagnostics.append('Все функции используют входные данные и преобразуют результат.')
    else:
        rank = 'A'
        diagnostics.append('Не каждая функция демонстрирует зависимость результата от аргумента.')
    # S++ only where the signal is strong and code is PEP8-clean (approximation).
    from inspector import check_pep8  # same worker; no dependency on user files
    pep8_issues = check_pep8(files['core.py']) + check_pep8(files['__init__.py'])
    if rank == 'S' and complexity >= 6 and not pep8_issues:
        rank = 'S++'
    diagnostics.extend(pep8_issues)
    return {'evaluation': {'passed':True, 'rank':rank, 'diagnostics': diagnostics,
                           'tests_passed':len(calls), 'pep8_ok':not pep8_issues,
                           'no_overhead':True, 'no_bad_practice':True},
            'runs':[{'stdout':'', 'passed':True, 'expected_display':None,
                     'function':c['function'], 'args':c['args'],
                     'returned':c['result']} for c in calls],
            'analysis': {'dependency':dependency, 'varied':varied, 'transformed':transformed, 'complexity':complexity}}


def _bad(message: str) -> dict[str, Any]:
    return {'evaluation':{'passed':False,'rank':None,'diagnostics':[message],
                          'tests_passed':0}, 'runs':[], 'analysis':{}}
