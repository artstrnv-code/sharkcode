"""По авторскому заданию нужны два списка."""
import ast


def validate(context):
    tree = ast.parse(context['source'])
    list_nodes = [node for node in ast.walk(tree) if isinstance(node, ast.List)]
    # Допускаем создание двух списков через list() и генераторы.
    list_calls = [node for node in ast.walk(tree) if isinstance(node, ast.Call)
                  and isinstance(node.func, ast.Name) and node.func.id == 'list']
    # Срез списка создаёт новый список и тоже удовлетворяет «двум массивам».
    slices = [node for node in ast.walk(tree) if isinstance(node, ast.Subscript)
              and isinstance(node.slice, ast.Slice)]
    copies = [node for node in ast.walk(tree) if isinstance(node, ast.Call)
              and isinstance(node.func, ast.Attribute) and node.func.attr == 'copy']
    count = len(list_nodes) + len(list_calls) + len(slices) + len(copies) + sum(
        isinstance(node, ast.ListComp) for node in ast.walk(tree))
    return {'accepted': count >= 2,
            'diagnostics': [] if count >= 2 else ['По ТЗ нужны два отдельных списка.']}
