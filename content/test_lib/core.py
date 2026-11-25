"""Открытые функции учебного пакета для Грота веры."""
from math import isqrt


def something():
    print("some evil")


def sqrt(value):
    return value ** 0.5


def _to_bool(value):
    # input() возвращает строку: явно сопоставляем True/False,
    # не используем bool("False"), который равен True.
    if value is True or value == "True":
        return True
    if value is False or value == "False":
        return False
    raise ValueError("Введите True или False")


def read(value):
    return _to_bool(value)


def write(value):
    return _to_bool(value)


def learn(value):
    return _to_bool(value)


def roar():
    print("roar")


def sad():
    print("sad")
