"""The loop specialist forbids for and accepts flexible number formatting."""
import ast
import re


def validate(context):
    tree = ast.parse(context['source'])
    forbidden = any(isinstance(n, (ast.For, ast.AsyncFor, ast.comprehension))
                    for n in ast.walk(tree))
    runs = context['runs']
    numbers = [int(x) for x in re.findall(r'(?<!\d)\d+(?!\d)', runs[0]['stdout'])] if runs else []
    messages = []
    if forbidden:
        messages.append('Специалист по циклам: использование for запрещено.')
    if numbers != list(range(1, 101)):
        messages.append('Нужно вывести все числа от 1 до 100 по порядку, в любом оформлении.')
    return {'accepted': not messages, 'diagnostics': messages}
