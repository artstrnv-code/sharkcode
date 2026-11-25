"""Only the actual imported learn() may satisfy the reader task."""
import ast


def validate(context):
    source = context['source']
    tree = ast.parse(source)
    calls = [n for n in ast.walk(tree) if isinstance(n, ast.Call)]
    used_learn = any(isinstance(c.func, ast.Name) and c.func.id == 'learn' for c in calls)
    imported = any(isinstance(n, ast.ImportFrom) and n.module == 'test_lib'
                   and any(a.name == 'learn' for a in n.names) for n in ast.walk(tree))
    ok = used_learn and imported
    return {'accepted': ok, 'diagnostics': [] if ok else ['Нужно импортировать и вызвать test_lib.learn().']}
