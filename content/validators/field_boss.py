"""Запрет встроенной сортировки в бою со Стражем."""
import ast


def validate(context):
    tree = ast.parse(context['source'])
    prohibited = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Call):
            if isinstance(node.func, ast.Name) and node.func.id == 'sorted':
                prohibited.append('sorted()')
            if isinstance(node.func, ast.Attribute) and node.func.attr == 'sort':
                prohibited.append('.sort()')
    return {'accepted': not prohibited,
            'diagnostics': ['Нельзя использовать ' + ', '.join(sorted(set(prohibited)))] if prohibited else []}
