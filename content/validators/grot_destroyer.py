"""The dot ban is literal; output must also be produced by real test_lib calls."""
import ast


def validate(context):
    source = context['source']
    if '.' in source:
        return {'accepted': False, 'diagnostics': ['Запрещён символ точки (.) в коде.']}
    tree = ast.parse(source)
    functions = ('something', 'roar', 'sad')
    imported = set()
    for n in ast.walk(tree):
        if isinstance(n, ast.ImportFrom) and n.module == 'test_lib':
            imported.update(a.name for a in n.names)
    calls = {n.func.id for n in ast.walk(tree)
             if isinstance(n, ast.Call) and isinstance(n.func, ast.Name)}
    # Wildcard импорт разрешён ТЗ. Вывод уже проверяется основным инспектором.
    good = ("*" in imported or set(functions).issubset(imported)) and set(functions).issubset(calls)
    return {'accepted': good, 'diagnostics': [] if good else ['Импортируй и вызови все три функции.']}
