"""Static regression checks for Bevy navigation. Not a substitute for cargo build."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1] / "src"
SOURCE = (ROOT / "main.rs").read_text(encoding="utf-8")
REGIONS = (ROOT / "regions.rs").read_text(encoding="utf-8")
MAP = (ROOT / "region_map.rs").read_text(encoding="utf-8")


class NavigationContract(unittest.TestCase):
    def test_region_is_not_the_same_as_node(self):
        self.assertIn("enum Location { Valley, Grot, River, For4, Tower, Field }", REGIONS)
        self.assertIn("struct LocationMap(Location)", SOURCE)
        self.assertIn("Action::UseEntry", SOURCE)
        self.assertIn("fn spawn_location_map(", MAP)

    def test_secret_grot_is_not_drawn_before_discovery(self):
        self.assertIn("button.0 == 8", MAP)
        self.assertIn("game.grot_unlocked || game.dev_mode", MAP)
        self.assertNotIn("Грот: скрыт", SOURCE)

    def test_changing_node_clears_editor_and_undo(self):
        self.assertIn("if project.last_selected != game.selected", SOURCE)
        self.assertIn("project.reset();\n        buffer.reset();", SOURCE)
        self.assertIn("self.undo.clear();", SOURCE)
        self.assertIn("self.redo.clear();", SOURCE)

    def test_switching_tabs_keeps_project_files(self):
        self.assertIn("project.files[active] = editor.source.clone();", SOURCE)
        self.assertIn("editor.source = project.files[*index].clone();", SOURCE)

    def test_existing_progress_and_hp_kept(self):
        self.assertIn("river_discovered", SOURCE)
        self.assertIn("self.grot_unlocked = value[\"grot\"]", SOURCE)
        self.assertIn("self.hp = value[\"hp\"]", SOURCE)


if __name__ == "__main__":
    unittest.main()
