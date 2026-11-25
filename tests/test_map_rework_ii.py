"""Contracts of SHARKCODE 0.2.32 (static tests do not replace Rust compile)."""
from pathlib import Path
import tomllib
import unittest

ROOT=Path(__file__).resolve().parents[1]
SRC=(ROOT/'src/main.rs').read_text(encoding='utf-8')
NAV=(ROOT/'src/regions.rs').read_text(encoding='utf-8')
MAP=(ROOT/'src/region_map.rs').read_text(encoding='utf-8')

class MapRework(unittest.TestCase):
    def test_entries_are_data_driven_and_one_way(self):
        cases=[('grot','grot_faith'), ('valley_to_river','holy_forest_river'),
               ('grot_to_river','holy_forest_river')]
        for name,destination in cases:
            with self.subTest(name=name):
                data=tomllib.loads((ROOT/f'content/nodes/{name}.toml').read_text(encoding='utf-8'))
                self.assertEqual(data['META']['type'],'entry')
                self.assertEqual(data['GAME']['destination'],destination)
                self.assertEqual(data['GAME']['interaction'],'button')
        self.assertIn('if !self.dev_mode && target_region != self.location',NAV)
        self.assertIn('self.location = destination;',NAV)
        self.assertIn('"current_location":self.location.key()',SRC)
        self.assertIn('fn use_entry(&mut self, source: &str)',NAV)

    def test_no_tabs_or_secret_disclosure(self):
        self.assertNotIn('Action::EnterLocation',SRC)
        self.assertIn('struct SecretRoute;',SRC)
        self.assertIn('button.0 == 8',MAP)
        self.assertIn('game.grot_unlocked || game.dev_mode',MAP)

    def test_destroyer_accepts_star_import(self):
        code=(ROOT/'content/validators/grot_destroyer.py').read_text(encoding='utf-8')
        scope={}
        exec(code,scope)
        check=scope['validate']
        good='from test_lib import *\nsomething()\nroar()\nsad()'
        self.assertTrue(check({'source':good})['accepted'])
        self.assertFalse(check({'source':good+'\n# .'})['accepted'])

    def test_previous_quiz_message_is_not_hardcoded(self):
        self.assertNotIn('"Игрок в карты уже побеждён в этом забеге."',SRC)

if __name__=='__main__':unittest.main()
