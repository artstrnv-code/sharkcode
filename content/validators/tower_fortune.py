"""У Цыганки запрещён break, как в оригинальном задании."""
import ast


def validate(context):
    tree = ast.parse(context['source'])
    forbidden = any(isinstance(n, ast.Break) for n in ast.walk(tree))
    return {'accepted': not forbidden,
            'diagnostics': ['Цыганка запрещает break.'] if forbidden else []}
