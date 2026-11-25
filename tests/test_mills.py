"""0.2.47 tests: new puzzle contracts are not print-only checks."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('judge047', ROOT/'runtime/judge047.py')
JUDGE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(JUDGE)


def judge(task, source, tests=None):
    return JUDGE.judge({'task':task,'source':source, 'tests': tests or [{'inputs':[]}]})['evaluation']


class MillsPuzzles(unittest.TestCase):
    def test_field_chest(self):
        self.assertTrue(judge('field_new_chest', 'massive=[0,0,0,[0,0,[0,7]]]\n')['passed'])
        self.assertFalse(judge('field_new_chest', 'print(7)\n')['passed'])

    def test_scalar(self):
        self.assertTrue(judge('mills_scalar','a=(7,)\n')['passed'])
        self.assertFalse(judge('mills_scalar','a=(7)\n')['passed'])

    def test_good_bad(self):
        source='''pairs=[(1.1,2.2),(3.3,4.4),(5.5,6.6),(7.7,8.8),(9.9,10.1)]
result={}
total=0
for k,v in pairs:
    result[k]=v
    total+=k+v
'''
        self.assertTrue(judge('mills_good_bad',source)['passed'])
        bad='''pairs=[(1.1,2.2),(3.3,4.4),(5.5,6.6),(7.7,8.8),(9.9,10.1)]
result={}
for k,v in pairs: result[k]=v
total=sum(k+v for k,v in result.items())
'''
        self.assertFalse(judge('mills_good_bad',bad)['passed'])

    def test_elf_no_lists(self):
        cases=[{'inputs':['кот пес лес'],'expected':"('кот', 'пес', 'лес', 'эльф')"}]
        self.assertTrue(judge('mills_elf', 'a=tuple(input().split())\nb=a+("эльф",)\nprint(b)\n',cases)['passed'])
        self.assertFalse(judge('mills_elf', 'a=list(input().split())\na.append("эльф")\nprint(tuple(a))\n',cases)['passed'])

    def test_unknown_name(self):
        cases=[{'inputs':['петя'],'unknown_name':True}]
        self.assertTrue(judge('mills_names','print(None)\n',cases)['passed'])
        self.assertFalse(judge('mills_names','print({}[input()])\n',cases)['passed'])

    def test_boss_probes(self):
        has_list=[{'inputs':[],'cortezh':'(1,(2,[3]))','expected':'yes'}]
        no_list=[{'inputs':[],'cortezh':'(1,(2,3))','expected':'no'}]
        self.assertTrue(judge('mills_boss','d={}\nd[cortezh]=4\n',has_list)['diagnostic_probe'])
        self.assertTrue(judge('mills_boss','d={}\nd[cortezh]=4\n',no_list)['diagnostic_probe'])
        self.assertTrue(judge('mills_boss','print("yes")\n',has_list)['passed'])
        self.assertTrue(judge('mills_boss','print("no")\n',has_list)['instant_death'])
        self.assertFalse(judge('mills_boss','print(cortezh[0])\n',has_list)['passed'])
        self.assertFalse(judge('mills_boss','print(3 in cortezh)\n',has_list)['passed'])

    def test_inspector_isolated_python(self):
        request={'task':'mills_scalar','source':'a=(7,)\n','tests':[{'inputs':[]}]}
        process=subprocess.run([sys.executable,'-I','runtime/inspector.py'],
            cwd=ROOT, input=json.dumps(request),text=True,capture_output=True,timeout=10)
        self.assertEqual(process.returncode,0,process.stderr)
        self.assertTrue(json.loads(process.stdout)['evaluation']['passed'])


if __name__=='__main__':unittest.main()
