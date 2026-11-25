"""Шутник обучает управляющим операторам continue и break."""
import ast


def validate(context):
    tree = ast.parse(context['source'])
    nodes = tuple(ast.walk(tree))
    missing = []
    if not any(isinstance(n, ast.Continue) for n in nodes):
        missing.append('Нужен continue для ответа 1.')
    if not any(isinstance(n, ast.Break) for n in nodes):
        missing.append('Нужен break для ответа 3.')
    return {'accepted': not missing, 'diagnostics': missing}
