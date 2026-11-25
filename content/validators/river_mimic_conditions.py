"""Mimic checks all 81 sequences; any unique output is accepted."""
import ast


def validate(context):
    runs = context['runs']
    diagnostics = []
    if len(runs) != 81:
        return {'accepted': False, 'diagnostics': ['Нужны все 81 комбинация.']}
    outputs = [run['stdout'] for run in runs]
    if len(set(outputs)) != 81 or any(not output for output in outputs):
        diagnostics.append('Для каждой из 81 комбинаций нужен непустой уникальный вывод.')
    if any(run.get('error') for run in runs):
        diagnostics.append('Программа завершилась с ошибкой.')
    return {'accepted': not diagnostics, 'diagnostics': diagnostics}
