"""Контрольные контракты новых узлов, без зависимости от Bevy/Cargo."""
import importlib.util
import json
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('judge051', ROOT / 'runtime' / 'judge051.py')
judge051 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(judge051)

class Contracts(unittest.TestCase):
    def judge(self, task, source, cases):
        return judge051.judge({'task':task, 'source':source, 'tests':cases})['evaluation']

    def test_harry(self):
        cases = [
            {'inputs':['12345'],'expected':str(__import__('math').factorial(24))},
            {'inputs':['13579'],'expected':'1'},
            {'inputs':['802'],'expected':str(__import__('math').factorial(802))},
        ]
        code = '''import math

def solve():
    text = input()
    even = ''.join(c for c in text if c in '02468')
    print(math.factorial(int(even or 0)))

solve()
'''
        self.assertTrue(self.judge('if_harry',code,cases)['passed'])
        self.assertFalse(self.judge('if_harry','print(1)\n',cases)['passed'])

    def test_algorithms(self):
        cases = [{'inputs':['1','-5','6'],'a':1,'b':-5,'c':6}]
        code = '''from math import sqrt

def calc(a, b, c):
    d = b*b - 4*a*c
    if d == 0:
        print(-b/(2*a))
    else:
        print((-b-sqrt(d))/(2*a), (-b+sqrt(d))/(2*a))

a = int(input())
b = int(input())
c = int(input())
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
calc(a,b,c)
'''
        self.assertTrue(self.judge('if_algorithms',code,cases)['passed'])
        self.assertFalse(self.judge('if_algorithms','for i in range(10): print(i)\n',cases)['passed'])

    def test_fulione(self):
        good = '''def dish(number):
    return f"{number} / {number*number + 123}"
'''
        self.assertTrue(self.judge('if_fulione',good,[{}])['passed'])
        self.assertFalse(self.judge('if_fulione','def dish(x):\n    return str(x) + "hello"\n',[{}])['passed'])

    def test_mimic(self):
        cases = [{'inputs':['a','a','b','']}, {'inputs':['wolf','wolf','wolf','']}]
        good = '''values = set()
while True:
    s = input()
    if not s:
        break
    values.add(s)
print(values)
'''
        self.assertTrue(self.judge('if_mimic',good,cases)['passed'])
        self.assertFalse(self.judge('if_mimic','a=[]\nb=set()\nprint(b)\n',cases)['passed'])

    def test_boss(self):
        good = '''accounts = {}

def register():
    raw = input().split()
    name, surname = raw[:2]
    age = int(raw[2])
    mail = raw[-1].split('@')[0]
    print(f"Привет, {name}! Добро пожаловать")
    if age % 100 in (11, 12, 13, 14):
        ending = "лет"
    elif age % 10 == 1:
        ending = "год"
    elif age % 10 in (2, 3, 4):
        ending = "года"
    else:
        ending = "лет"
    print(f"Тебе {age} {ending}, придумай пароль!")
    while True:
        password = input()
        if (len(password) >= 8 and any(c.isdigit() for c in password)
                and any(c in '&#₽%@*<>' for c in password)):
            break
    accounts[name + ' ' + surname] = {'email': mail, 'age': age,
                                      'password': password, 'name': name}

def login():
    email, password = input().split()
    for user in accounts.values():
        if user['email'] == email.split('@')[0] and user['password'] == password:
            print('Здравствуйте, ' + user['name'])
            return
    print('Пусто ;(')
    print('Зарегистрироваться?')
    if input() == 'да':
        register()
'''
        result = self.judge('if_boss',good,[{}])
        self.assertTrue(result['passed'], result['diagnostics'])
        self.assertFalse(self.judge('if_boss','def register():\n    print("Привет")\n',[{}])['passed'])

if __name__ == '__main__': unittest.main()
