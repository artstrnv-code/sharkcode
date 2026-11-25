"""Массив должен опустеть в цикле по условию оригинала."""
import ast


def validate(context):
    tree = ast.parse(context['source'])
    loops = any(isinstance(n, ast.While) for n in ast.walk(tree))
    pops = any(isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute)
               and n.func.attr in ('pop', 'remove') for n in ast.walk(tree))
    diagnostics = []
    if not loops: diagnostics.append('Требуется цикл while до опустошения массива.')
    if not pops: diagnostics.append('Элементы должны удаляться из массива.')
    return {'accepted': not diagnostics, 'diagnostics': diagnostics}
