"""Grot content and creative project regressions."""
from pathlib import Path
import json
import subprocess
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
INSPECTOR = ROOT/'runtime'/'inspector.py'


def evaluate(task, source='', tests=None, validator=None, project_files=None):
    request = {'task':task,'source':source, 'tests': tests or [{'inputs':[]}],
               'validator':validator or '', 'project_files':project_files or {}}
    done = subprocess.run([sys.executable,'-I',str(INSPECTOR)],input=json.dumps(request),
                          text=True,capture_output=True,timeout=7,check=True)
    return json.loads(done.stdout)


def project(core):
    return {'pyproject.toml':'[build-system]\nrequires = ["hatchling"]\nbuild-backend = "hatchling.build"\n[project]\nname = "name_lib"\nversion = "0.0.0"\nreadme = "README.md"\n',
            'README.md':'', '__init__.py':'from .core import dragon, fire, python\n',
            'core.py':core}


class GrotTests(unittest.TestCase):
    def test_toml(self):
        for name in ('grot_scroll1','grot_lover','grot_reader','grot_destroyer',
                     'grot_scroll2','grot_boss','grot_clear','valley_clear','river_entry'):
            with self.subTest(name=name):
                config=tomllib.loads((ROOT/'content'/'nodes'/f'{name}.toml').read_text(encoding='utf8'))
                self.assertEqual(config['META']['id'],name)

    def test_library(self):
        sys.path.insert(0,str(ROOT/'content'))
        try:
            from test_lib import learn
            self.assertIs(learn('True'),True)
            self.assertIs(learn('False'),False)
            with self.assertRaises(ValueError):learn('anything')
        finally: sys.path.remove(str(ROOT/'content'))

    def test_reader(self):
        tests=[{'inputs':['True'],'expected':'True'}, {'inputs':['False'],'expected':'False'}]
        good='from test_lib import learn\nprint(learn(input()))'
        got=evaluate('grot_reader',good,tests,'grot_reader.py')
        self.assertTrue(got['evaluation']['passed'],got['evaluation']['diagnostics'])
        self.assertFalse(evaluate('grot_reader','print(input())',tests,'grot_reader.py')['evaluation']['passed'])

    def test_destroyer(self):
        tests=[{'inputs':[],'expected':'some evil\nroar\nsad'}]
        good='from test_lib import something, roar, sad\nsomething()\nroar()\nsad()'
        self.assertTrue(evaluate('grot_destroyer',good,tests,'grot_destroyer.py')['evaluation']['passed'])
        self.assertFalse(evaluate('grot_destroyer',good+'\n# .',tests,'grot_destroyer.py')['evaluation']['passed'])

    def test_boss_constant_is_b(self):
        src=''.join(f'def {name}(a):\n    return 0\n' for name in ('dragon','fire','python'))
        result=evaluate('grot_boss',project_files=project(src))
        self.assertTrue(result['evaluation']['passed'], result['evaluation']['diagnostics'])
        self.assertEqual(result['evaluation']['rank'],'B')

    def test_boss_useful(self):
        src='''def dragon(a):
    return a * 2


def fire(a):
    return a + 5


def python(a):
    return a ** 2
'''
        result=evaluate('grot_boss',project_files=project(src))
        self.assertTrue(result['evaluation']['passed'], result['evaluation']['diagnostics'])
        self.assertIn(result['evaluation']['rank'],('S','S++'))

    def test_boss_must_return(self):
        src=''.join(f'def {name}(a):\n    print(a)\n' for name in ('dragon','fire','python'))
        self.assertFalse(evaluate('grot_boss',project_files=project(src))['evaluation']['passed'])

    def test_boss_exposed(self):
        src=''.join(f'def {name}(a):\n    return a\n' for name in ('dragon','fire','python'))
        files=project(src);files['__init__.py']=''
        self.assertFalse(evaluate('grot_boss',project_files=files)['evaluation']['passed'])

if __name__ == '__main__': unittest.main()
