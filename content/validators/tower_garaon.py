"""Условия оригинального босса: один while, без for и подмены x."""
import ast


def validate(context):
    source = context['source']
    diagnostics = []
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return {'accepted': False, 'diagnostics': ['Синтаксическая ошибка.']}
    loops = [node for node in ast.walk(tree) if isinstance(node, ast.While)]
    if len(loops) != 1:
        diagnostics.append('Разрешён только один while.')
    if any(isinstance(node, (ast.For, ast.AsyncFor)) for node in ast.walk(tree)):
        diagnostics.append('Нельзя использовать for.')
    if len(tree.body) < 2 or not isinstance(tree.body[0], ast.Assign) or not isinstance(tree.body[1], ast.While):
        diagnostics.append('Начало решения должно содержать x = True и while x == True:')
    else:
        first = tree.body[0]
        if not (len(first.targets) == 1 and isinstance(first.targets[0], ast.Name)
                and first.targets[0].id == 'x' and isinstance(first.value, ast.Constant)
                and first.value.value is True):
            diagnostics.append('Нельзя менять стартовое x = True.')
        if ast.unparse(tree.body[1].test) != 'x == True':
            diagnostics.append('Сохрани исходное условие while x == True.')
    for node in ast.walk(tree):
        if isinstance(node, (ast.Assign, ast.AnnAssign, ast.AugAssign, ast.NamedExpr)) and (not tree.body or node is not tree.body[0]):
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            if any(isinstance(target, ast.Name) and target.id == 'x' for target in targets):
                diagnostics.append('Нельзя изменять x ради выхода из цикла.')
                break
    if not any(isinstance(node, ast.Break) for node in ast.walk(tree)):
        diagnostics.append('Выйди из цикла через break.')
    return {'accepted': not diagnostics, 'diagnostics': diagnostics}
