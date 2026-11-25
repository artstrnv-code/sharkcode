"""SHARKCODE 0.2.51 — контракты испытаний ИФМОСТРОЙ-14.

Тестовый код выполняется в уже существующей Python-песочнице движка.
Этот модуль НЕ обеспечивает изоляцию от ОС и не заменяет sandbox.rs.
"""
from __future__ import annotations

import ast
import builtins
import contextlib
import dis
import io
import math
import re
import sys
import types


SPECIAL = set('&#₽%@*<>')


def error_report(message: str) -> dict:
    return {'evaluation': {'passed': False, 'rank': None, 'score': 0,
                            'diagnostics': [message]}, 'runs': [], 'analysis': {}}


def execute(source: types.CodeType, inputs: list[str]) -> tuple[dict, str, str | None]:
    """Общая среда stdin/stdout с конечной последовательностью input()."""
    values = iter(inputs)
    out = io.StringIO()
    namespace = {'__name__': '__main__'}
    def get_input(prompt=''):
        if prompt:
            print(prompt, end='')
        try:
            return next(values)
        except StopIteration:
            raise EOFError('Входные данные закончились') from None
    original = builtins.input
    try:
        builtins.input = get_input
        with contextlib.redirect_stdout(out):
            exec(source, namespace, namespace)
    except BaseException as exc:
        return namespace, out.getvalue(), f'{type(exc).__name__}: {exc}'
    finally:
        builtins.input = original
    return namespace, out.getvalue(), None


def check_harry(source: types.CodeType, case: dict) -> tuple[bool, str, str, str | None]:
    ns, output, err = execute(source, case['inputs'])
    ok = not err and output.strip() == case['expected']
    return ok, 'Неверный факториал чётных цифр.', output, err


def check_mimic(source: types.CodeType, case: dict) -> tuple[bool, str, str, str | None]:
    _, output, err = execute(source, case['inputs'])
    expected = set(case['inputs'][:-1])
    # Принимаем и print(set), и print через пробел/перенос строки.
    displayed = output.strip()
    try:
        value = ast.literal_eval(displayed)
        if not isinstance(value, (set, list, tuple)):
            raise ValueError()
        actual = list(value)
    except (ValueError, SyntaxError):
        actual = displayed.splitlines()
        if len(actual) == 1 and len(expected) > 1:
            actual = displayed.split()
    ok = not err and len(actual) == len(expected) and set(actual) == expected
    return ok, 'Выведи каждое непустое значение ровно один раз.', output, err


def check_algorithms(source: types.CodeType, case: dict) -> tuple[bool, str, str, str | None]:
    # Наблюдаем арифметические операции при фактическом выполнении.
    # Повторение print() без повторного вычисления недостаточно.
    binary_ops = 0
    def trace(frame, event, arg):
        nonlocal binary_ops
        if frame.f_code.co_filename != '<SHARKCODE>':
            return trace
        if event == 'call':
            frame.f_trace_opcodes = True
        elif event == 'opcode':
            instruction = dis.opname[frame.f_code.co_code[frame.f_lasti]]
            if instruction.startswith('BINARY_'):
                binary_ops += 1
        return trace
    old = sys.gettrace()
    try:
        sys.settrace(trace)
        _, output, err = execute(source, case['inputs'])
    finally:
        sys.settrace(old)
    if err:
        return False, 'Ошибка выполнения.', output, err
    if binary_ops < 10:
        return False, 'Нужно десять вычислений, а не десять копий готового ответа.', output, None
    a, b, c = (float(case[k]) for k in ('a', 'b', 'c'))
    discr = b*b - 4*a*c
    roots = [(-b - math.sqrt(discr))/(2*a)]
    if discr > 0:
        roots.append((-b + math.sqrt(discr))/(2*a))
    # Десять наборов чисел: для D>0 две величины за один расчёт,
    # для D=0 достаточно одного корня.
    tokens = re.findall(r'(?<![\w])[-+]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][-+]?\d+)?', output)
    try:
        nums = [float(v) for v in tokens]
    except ValueError:
        nums = []
    count = len(roots)
    ok = len(nums) == 10 * count and all(
        sorted(nums[i*count:(i+1)*count]) == sorted(roots)
        or all(math.isclose(x, y, abs_tol=1e-6, rel_tol=1e-6)
               for x, y in zip(sorted(nums[i*count:(i+1)*count]), sorted(roots)))
        for i in range(10)
    )
    return ok, 'Нужно вывести десять корректных решений уравнения.', output, None


def function_names(tree: ast.Module) -> list[str]:
    return [n.name for n in tree.body if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef))]


def check_fulione(source: types.CodeType, tree: ast.Module) -> tuple[bool, str, str, str | None]:
    # Не допускаем самостоятельный вызов входа, print и другие побочные действия.
    if any(not isinstance(n, (ast.FunctionDef, ast.Import, ast.ImportFrom,
                              ast.Assign, ast.AnnAssign)) for n in tree.body):
        return False, 'Оставь только определение функции и необходимые объявления.', '', None
    ns, output, err = execute(source, [])
    if err or output:
        return False, 'При объявлении функции не нужно ничего выводить.', output, err
    names = function_names(tree)
    for name in names:
        func = ns.get(name)
        if not callable(func):
            continue
        try:
            results = [func(n) for n in (5, 11, 42, 5)]
        except BaseException:
            continue
        if not all(type(r) is str and str(n) in r and len(r) > len(str(n))
                   for n, r in zip((5,11,42,5), results)):
            continue
        # Дополнительная часть должна зависеть от числа, не быть одной
        # неизменной добавкой «hello» для всех аргументов.
        extras = [r.replace(str(n), '', 1) for n, r in zip((5,11,42), results)]
        if len(set(extras)) < 3:
            continue
        return True, '', output, None
    return False, 'Не найдена функция с одним числовым аргументом и уникальным содержимым.', output, None


def test_accounts(source: types.CodeType, tree: ast.Module) -> tuple[bool, str, str, str | None]:
    """Вызываем предоставленные функции в одном namespace, сохраняя аккаунты.

    Функции выбираем по наблюдаемому поведению; названия не навязываем.
    """
    names = function_names(tree)
    if len(names) < 2:
        return False, 'Нужны отдельные функции регистрации и входа.', '', None
    def prepare():
        return execute(source, [])
    def call(ns, name, data):
        iterator = iter(data)
        out = io.StringIO()
        original = builtins.input
        def read(prompt=''):
            if prompt:
                print(prompt, end='')
            try:
                return next(iterator)
            except StopIteration:
                raise EOFError('Недостаточно данных для функции') from None
        try:
            builtins.input = read
            with contextlib.redirect_stdout(out):
                ns[name]()
            return out.getvalue(), None
        except BaseException as e:
            return out.getvalue(), f'{type(e).__name__}: {e}'
        finally:
            builtins.input = original

    users = [('Анна','Петрова',21,'anna@example.org','Ab1@','Abcdef1@x'),
             ('Игорь','Сидоров',12,'igor@gmail.com','12345678','Strong9#xx'),
             ('Мария','Лесная',25,'maria@test.ru','abcdefg@','Zxcvbn8%'),
             ('Алина','Ветрова',24,'alina@site.ru','1@','MadeUp3@'),
             ('Олег','Нечаев',11,'oleg@site.ru','nope','Passwd1@')]
    def reg_input(u):
        name, surname, age, mail, invalid, good = u
        return [f'{name} {surname} {age} лет {mail}',invalid,good]
    # Пробуем объявленные функции: должен корректно обслужить реальный stdin.
    for registration in names:
        ns, top_output, top_error = prepare()
        if top_error or top_output or not callable(ns.get(registration)):
            continue
        try:
            out, err = call(ns, registration, reg_input(users[0]))
        except BaseException:
            continue
        if err or 'Привет, Анна! Добро пожаловать' not in out or 'Тебе 21 год, придумай пароль!' not in out:
            continue
        ok = True
        for u, age_text in [(users[1], '12 лет'), (users[2], '25 лет'),
                            (users[3], '24 года'), (users[4], '11 лет')]:
            txt, e = call(ns, registration, reg_input(u))
            if e or f'Привет, {u[0]}! Добро пожаловать' not in txt or f'Тебе {age_text}, придумай пароль!' not in txt:
                ok=False
                break
        if not ok:
            continue
        # После нескольких вызовов должен существовать словарь с данными аккаунтов.
        dicts = [v for k, v in ns.items() if not k.startswith('__') and isinstance(v, dict)]
        snapshot = repr(dicts)
        if not dicts or any(str(age) not in snapshot or mail.split('@')[0] not in snapshot or pwd not in snapshot
                            for _,_,age,mail,_,pwd in users):
            continue
        for login in names:
            if login == registration:
                continue
            outs = []
            for values in [['anna@other.net Abcdef1@x'],
                           ['igor@gmail.com Strong9#xx'],
                           ['maria@test.ru Zxcvbn8%']]:
                txt, e = call(ns,login,values)
                outs.append(txt)
                if e:
                    break
            if len(outs)!=3 or not all(f'Здравствуйте, {u[0]}' in s for u,s in zip(users,outs)):
                continue
            fail, err = call(ns,login,['anna@example.org WRONG','нет'])
            if err or 'Пусто ;(' not in fail or 'Зарегистрироваться?' not in fail:
                continue
            new=('Павел','Зорин',31,'pavel@example.org','bad','NewPass7&')
            fail, err = call(ns,login,['unknown@anywhere.net nope','да']+reg_input(new))
            if err or 'Пусто ;(' not in fail or 'Привет, Павел! Добро пожаловать' not in fail or '31 год' not in fail:
                continue
            success, err = call(ns,login,['pavel@another.net NewPass7&'])
            if not err and 'Здравствуйте, Павел' in success:
                extra = ('Вера','Серова',101,'vera@example.org','bad','Strong1@vv')
                extended, extended_err = call(ns,registration,reg_input(extra))
                age_bonus = not extended_err and 'Тебе 101 год, придумай пароль!' in extended
                return True, 'three_digit_age' if age_bonus else '', '\n'.join(outs+[fail,success]), None
    return False, ('Функции должны сохранять несколько аккаунтов и обеспечивать '
                   'успешный вход, отказ и регистрацию по ответу «да».') , '', None


def judge(request: dict) -> dict:
    task = request['task']
    source = request['source']
    try:
        tree = ast.parse(source)
        compiled = compile(tree, '<SHARKCODE>', 'exec')
    except (SyntaxError, ValueError) as err:
        return error_report(f'{type(err).__name__}: {err}')
    nodes = list(ast.walk(tree))
    diagnostics = []
    if task == 'if_algorithms' and any(isinstance(x, (ast.For, ast.While,
                            ast.AsyncFor, ast.ListComp, ast.SetComp, ast.DictComp,
                            ast.GeneratorExp)) for x in nodes):
        diagnostics.append('Циклы запрещены.')
    if task == 'if_mimic':
        collections = sum(isinstance(x, (ast.List, ast.Set, ast.Dict,
                           ast.ListComp, ast.SetComp, ast.DictComp)) or
                           isinstance(x, ast.Call) and isinstance(x.func, ast.Name)
                           and x.func.id in ('list','dict','set') for x in nodes)
        if collections > 1:
            diagnostics.append('Разрешено создать только одну коллекцию.')
    if task == 'if_fulione' and any(isinstance(x, ast.Call) and isinstance(x.func, ast.Attribute)
                    and x.func.attr == 'seed' for x in nodes):
        # Разрешено, но поощряем решения без глобального random.seed().
        pass
    tests = request.get('tests') or [{}]
    runs = []
    age_bonus = False
    for i, case in enumerate(tests):
        if task == 'if_harry':
            ok, note, output, err = check_harry(compiled, case)
        elif task == 'if_algorithms':
            ok, note, output, err = check_algorithms(compiled, case)
        elif task == 'if_mimic':
            ok, note, output, err = check_mimic(compiled, case)
        elif task == 'if_fulione':
            ok, note, output, err = check_fulione(compiled, tree)
        elif task == 'if_boss':
            ok, note, output, err = test_accounts(compiled, tree)
            if ok and note == 'three_digit_age':
                age_bonus = True
        else:
            return error_report('Неизвестное испытание ИФМОСТРОЙ-14.')
        runs.append({'inputs':case.get('inputs',[]),'stdout':output, 'error':err,'passed':ok})
        if not ok:
            diagnostics.append(f'Тест {i+1}: {err or note}')
    passed = all(r['passed'] for r in runs) and not diagnostics
    style = [] if source.endswith('\n') else ['PEP8 W292: отсутствует перевод строки в конце файла']
    score = 100 - 7*len(style)
    if task == 'if_harry':
        # Строки тел функций не участвуют в бонусе, но код продолжает
        # проверяться целиком. Не поощряем программу без объявленной функции.
        outside = sum(1 for stmt in tree.body if not isinstance(stmt, (ast.FunctionDef,
                          ast.AsyncFunctionDef, ast.Import, ast.ImportFrom)))
        if not function_names(tree):
            score -= 15
        elif outside > 1:
            score -= min(15, (outside-1)*4)
    if task == 'if_fulione' and any(isinstance(x, ast.Call) and isinstance(x.func, ast.Attribute)
                             and x.func.attr == 'seed' for x in nodes):
        score -= 5
    if task == 'if_boss':
        # 2-значный возраст — полное выполнение ТЗ; 3-значный — бонус изящности.
        score -= 5 if not age_bonus else 0
    diagnostics.extend(style)
    score = max(0, score)
    rank = ('S++' if score>=99 else 'S' if score>=90 else 'A' if score>=76 else
            'B' if score>=55 else 'C') if passed else None
    return {'evaluation': {'passed':passed,'rank':rank,'score':score if passed else 0,
                           'tests_passed':sum(x['passed'] for x in runs),
                           'diagnostics':diagnostics,'performance':{'execution_ns':0,'peak_python_bytes':0}},
            'runs':runs,'analysis':{}}
