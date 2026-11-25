"""SHARKCODE 0.2.47: explicit rules for region puzzles and experiment boss.

This module runs inside the existing CPython inspector process. It is NOT
an OS sandbox. No test data is imported from player-controlled modules.
"""
from __future__ import annotations
import ast
import builtins
import contextlib
import io
import math
import re
import traceback
from typing import Any

PUZZLES = {
    'field_new_chest', 'mills_scalar', 'mills_good_bad', 'mills_chest',
    'mills_boss', 'mills_monkeys', 'mills_elf', 'mills_lovelace',
    'mills_specialties', 'mills_names', 'mills_lookalike',
}


def inspect_ast(task: str, tree: ast.AST) -> list[str]:
    errors: list[str] = []
    nodes = list(ast.walk(tree))
    if task == 'project_converter':
        for n in nodes:
            if isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == 'int':
                if len(n.args) != 1 or n.keywords:
                    errors.append('int() разрешён только с одним аргументом.')
                    break
    if task == 'mills_good_bad' and sum(isinstance(n, (ast.For, ast.AsyncFor, ast.comprehension)) for n in nodes) > 1:
        errors.append('Разрешён максимум один цикл for.')
    if task == 'mills_elf' and any(isinstance(n, (ast.List, ast.ListComp)) or
        isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == 'list'
        for n in nodes):
        errors.append('Использование списков запрещено.')
    if task == 'mills_boss':
        if any(isinstance(n, (ast.For, ast.AsyncFor, ast.ListComp, ast.SetComp,
                             ast.DictComp, ast.GeneratorExp)) for n in nodes):
            errors.append('Запрещено перебирать кортеж.')
        if any(isinstance(n, ast.Compare) and any(isinstance(op, (ast.In, ast.NotIn))
                for op in n.ops) for n in nodes):
            errors.append('Оператор in запрещён.')
        if any(isinstance(n, ast.Subscript) and isinstance(n.value, ast.Name)
                and n.value.id == 'cortezh' for n in nodes):
            errors.append('Обращение по индексам запрещено.')
        if any(isinstance(n, ast.Attribute) and isinstance(n.value, ast.Name)
                and n.value.id == 'cortezh' for n in nodes):
            errors.append('Нельзя обращаться к методам для извлечения элементов кортежа.')
        if any(isinstance(n, ast.Call) and isinstance(n.func, ast.Name)
                and n.func.id in ('print', 'str', 'repr', 'list', 'tuple', 'iter', 'next',
                                  'sorted', 'enumerate', 'reversed', 'sum', 'min', 'max',
                                  'any', 'all')
                and any(isinstance(arg, ast.Name) and arg.id == 'cortezh'
                        for arg in n.args) for n in nodes):
            errors.append('Нельзя извлекать или отображать элементы cortezh.')
    return errors


def execute(code: Any, inputs: list[str], extra: dict | None = None) -> tuple[dict, str, str | None]:
    stream = io.StringIO()
    values = iter(inputs)
    def read(prompt=''):
        return next(values)
    namespace = {'__name__': '__main__', '__builtins__': dict(vars(builtins))}
    namespace['__builtins__']['input'] = read
    if extra: namespace.update(extra)
    err = None
    with contextlib.redirect_stdout(stream):
        try:
            exec(code, namespace, namespace)
        except BaseException as exc:
            err = f'{type(exc).__name__}: {exc}'
    return namespace, stream.getvalue()[:10000], err


def execute_stdin(code: Any, data: str) -> tuple[dict, str, str | None]:
    """Use a finite, pre-created stdin stream; input() and sys.stdin share it."""
    import sys
    stream = io.StringIO()
    stdin = io.StringIO(data)
    namespace = {'__name__': '__main__', '__builtins__': dict(vars(builtins))}
    previous = sys.stdin
    error = None
    try:
        sys.stdin = stdin
        with contextlib.redirect_stdout(stream):
            exec(code, namespace, namespace)
    except BaseException as exc:
        error = f'{type(exc).__name__}: {exc}'
    finally:
        sys.stdin = previous
    return namespace, stream.getvalue()[:10000], error


def check_state(task: str, ns: dict, output: str, tree: ast.AST) -> tuple[bool,str]:
    if output and task != 'mills_chest':
        return False, 'Вывод не требуется.'
    values = {k:v for k,v in ns.items() if not k.startswith('__')}
    if task == 'field_new_chest':
        m = values.get('massive')
        if type(m) is not list:
            return False, 'Переменная massive должна быть списком.'
        try:
            ok = (type(m[3]) is list and type(m[3][2]) is list and
                  type(m[3][2][1]) is int and m[3][2][1] == 7)
        except (IndexError, TypeError, KeyError):
            ok=False
        return ok, 'massive[3][2][1] должен возвращать 7.'
    if task == 'mills_scalar':
        ok = any(type(v) is tuple and len(v)==1 and type(v[0]) in (int,float)
                 for v in values.values())
        return ok, 'Сохрани одноэлементный кортеж с числом в переменной.'
    if task == 'mills_good_bad':
        tuples=[]
        for v in values.values():
            candidates = v if type(v) in (list,tuple) else [v]
            if type(v) is tuple and len(v)==2 and all(type(x) is float for x in v):
                candidates=[v]
            for item in candidates:
                if type(item) is tuple and len(item)==2 and all(type(x) is float for x in item):
                    if item not in tuples: tuples.append(item)
        if len(tuples) < 5:
            return False, 'Нужно создать пять двухэлементных float-кортежей.'
        for dictionary in (v for v in values.values() if type(v) is dict):
            if len(dictionary) != 5 or not all(type(k) is float and type(v) is float
                                                 for k,v in dictionary.items()):
                continue
            if not all(dictionary.get(a,object())==b for a,b in tuples[:5]):
                continue
            total=sum(k+v for k,v in dictionary.items())
            if any(type(v) in (int,float) and math.isclose(v,total,rel_tol=1e-10,abs_tol=1e-10)
                   for v in values.values()):
                return True, ''
        return False, 'Нужен словарь из пяти пар и сохранённая общая сумма ключей и значений.'
    if task == 'mills_chest':
        # No AI claim: objective signs of a self-written algorithm using a dict.
        dictionaries=[v for v in values.values() if isinstance(v,dict)]
        has_dict=bool(dictionaries) or any(isinstance(n,(ast.Dict,ast.DictComp))
                                            for n in ast.walk(tree))
        if not has_dict:
            return False, 'Нужен работающий алгоритм со словарём.'
        operations=sum(isinstance(n,(ast.For,ast.While,ast.If,ast.DictComp,
                                     ast.FunctionDef,ast.Call,ast.Subscript)) for n in ast.walk(tree))
        return operations>=2, 'Нужно решить собственную задачу, а не просто создать словарь.'
    return False, 'Нет правила проверки.'


def judge(request: dict) -> dict:
    task = request['task']
    source=request['source']
    try:
        tree=ast.parse(source)
        code=compile(tree,'<SHARKCODE>','exec')
    except (SyntaxError,ValueError) as exc:
        return {'evaluation':{'passed':False,'rank':None,'diagnostics':[str(exc)],
                              'diagnostic_probe':task=='mills_boss'},'runs':[],'analysis':{}}
    errors=inspect_ast(task,tree)
    diagnostics=list(errors)
    tests=request.get('tests') or [{'inputs':[]}]
    runs=[]
    matches=0
    for index,case in enumerate(tests):
        extras={}
        if task=='mills_boss':
            # Expr is generated ONLY by Rust and interpreted as a literal.
            extras['cortezh']=ast.literal_eval(case['cortezh'])
        if task == 'project_void':
            namespace,stdout,error=execute_stdin(code,case['stdin'])
        else:
            namespace,stdout,error=execute(code,case.get('inputs',[]),extras)
        run={'inputs':case.get('inputs',[]),'stdout':stdout,'error':error}
        if task in ('field_new_chest','mills_scalar','mills_good_bad','mills_chest'):
            ok,note=check_state(task,namespace,stdout,tree)
        elif task=='mills_boss':
            ok=not error and stdout.strip()==case['expected']
            note='Неверный окончательный ответ.'
        elif case.get('unknown_name'):
            ok=not error;note='Неизвестное имя не должно вызывать ошибку.'
        else:
            ok=not error and stdout.rstrip('\n')==case['expected']
            note='Неверный ответ.'
            # Valid output must match but also use requested tuple for two lessons.
            if ok and task=='mills_monkeys':
                ok=any(isinstance(v,tuple) and len(v)==10 for v in namespace.values())
                note='Нужно создавать кортеж из десяти элементов.'
            if ok and task=='mills_elf':
                ok=any(isinstance(v,tuple) and len(v)==4 and v[-1]=='эльф'
                       for v in namespace.values())
                note='Нужно создать новый кортеж из четырёх элементов.'
            if ok and task=='mills_lovelace':
                ok=any(isinstance(v,tuple) and len(v)==3 and
                       all(str(x)==case['inputs'][3] for x in v)
                       for v in namespace.values())
                note='Нужно создать кортеж из трёх копий четвёртого ввода.'
        run['passed']=bool(ok)
        runs.append(run)
        matches+=bool(ok)
        if not ok:
            if error: diagnostics.append(f'Тест {index+1}: {error}')
            elif not (task=='mills_boss' and stdout.strip() in ('yes','no')):
                diagnostics.append(f'Тест {index+1}: {note}')
    if task=='mills_boss':
        # Only an explicit wrong yes/no answer kills; failed experiments don't.
        answer=runs[0]['stdout'].strip() if runs else ''
        diagnostic_probe=not answer
        fatal=bool(answer) and (answer!=tests[0]['expected'] or bool(errors) or bool(runs[0]['error']))
    else:
        diagnostic_probe=False;fatal=False
    passed=bool(runs) and matches==len(runs) and not errors
    # Style scoring; never claim to estimate creativity objectively.
    try:
        import tokenize
        pep8=[] if not source or source.endswith('\n') else ['PEP8 W292: отсутствует перевод строки в конце файла']
    except Exception:pep8=[]
    diagnostics.extend(pep8)
    score=100-7*len(pep8)
    if passed and task=='mills_chest':
        # Heuristic only: parsing, functions and iterative transformations count.
        nodes=list(ast.walk(tree))
        if not any(isinstance(n,ast.FunctionDef) for n in nodes) and not any(
                isinstance(n,(ast.For,ast.While,ast.DictComp)) for n in nodes):
            score=min(score,80)
            diagnostics.append('Простая реализация. Для высокого ранга нужен алгоритм обработки данных.')
    rank=('S++' if score>=99 else 'S' if score>=90 else 'A' if score>=76
          else 'B' if score>=55 else 'C') if passed else None
    return {'evaluation':{'passed':passed,'rank':rank,'score':score if passed else 0,
            'diagnostics':diagnostics,'tests_passed':matches,
            'diagnostic_probe':diagnostic_probe,'instant_death':fatal,
            'performance':{'execution_ns':0,'peak_python_bytes':0}},
            'runs':runs,'analysis':{'created_names':sorted({n.id for n in ast.walk(tree) if isinstance(n,ast.Name) and isinstance(n.ctx,ast.Store)})}}
