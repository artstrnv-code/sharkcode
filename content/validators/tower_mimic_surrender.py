"""Require input(question) instead of separate print(question)."""
import ast


def validate(context):
    tree = ast.parse(context['source'])
    question = 'ты еще настаиваешь?'
    proper_prompt = any(isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name) and node.func.id == 'input'
        and node.args and isinstance(node.args[0], ast.Constant)
        and node.args[0].value == question for node in ast.walk(tree))
    runs = context['runs']
    expect = [('нет', ['я не сдамся'] * 100),
              ('да', ['я не сдамся'] * 100 + ['сдаюсь'])]
    errors = []
    if not proper_prompt:
        errors.append('Вопрос должен быть аргументом input("ты еще настаиваешь?").')
    if len(runs) != 2:
        errors.append('Нужны обе ветви ответа.')
    else:
        for run, (answer, lines) in zip(runs, expect):
            if run.get('error') or run['stdout'].splitlines() != lines:
                errors.append(f'Неверный результат для ответа «{answer}».')
    return {'accepted': not errors, 'diagnostics': errors}
