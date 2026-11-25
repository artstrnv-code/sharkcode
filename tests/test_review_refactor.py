"""Regression contracts for code-review refactoring (not a substitute for cargo)."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
MAIN = (ROOT / "src/main.rs").read_text(encoding="utf-8")
VALLEY = (ROOT / "src/valley.rs").read_text(encoding="utf-8")
REGIONS = (ROOT / "src/regions.rs").read_text(encoding="utf-8")
MAP = (ROOT / "src/region_map.rs").read_text(encoding="utf-8")


class ReviewRefactor(unittest.TestCase):
    def test_valley_generation_is_encapsulated(self):
        self.assertIn("valley::build_valley_stages(initial_seed)", MAIN)
        self.assertIn("fn random_stage", VALLEY)
        self.assertNotIn("fn random_stage", MAIN)
        self.assertIn('"grot_scroll1"', VALLEY)
        self.assertIn('"grot_to_river"', VALLEY)
        self.assertIn("result.extend(river::build_river_stages(initial_seed))", MAIN)

    def test_region_navigation_encapsulated(self):
        self.assertIn("fn location_of", REGIONS)
        self.assertIn("let target_region = location_of(index, &self.stages);", REGIONS)
        self.assertNotIn("regions::location_of(index, &self.stages)", REGIONS)
        self.assertIn("fn can_enter", REGIONS)
        self.assertIn("fn use_entry", REGIONS)
        self.assertIn("17 => river_discovered", VALLEY)
        self.assertNotIn("fn use_entry", MAIN)
        self.assertIn("region_map::spawn_location_map", MAIN)
        self.assertIn("region_map::update_location_map", MAIN)
        self.assertIn("fn spawn_location_map", MAP)

    def test_cleared_encounters_are_read_only(self):
        self.assertIn("fn finished_encounter", MAIN)
        self.assertIn('"enemy.quiz" | "enemy.project" | "chest.code"', MAIN)
        self.assertIn("if self.finished_encounter(self.selected)", MAIN)
        self.assertIn("!game.finished_encounter(game.selected)", MAIN)
        self.assertIn("!game_mut.finished_encounter(game_mut.selected)", MAIN)
        self.assertIn("fn encounter_result", MAIN)

    def test_results_saved_and_reset_with_run(self):
        self.assertIn('"finished_reports":self.finished_reports', MAIN)
        self.assertIn('self.finished_reports.clear();', MAIN)
        self.assertIn('value["finished_reports"]', MAIN)
        self.assertIn('self.finished_reports.insert(self.selected, self.message.clone())', MAIN)
        self.assertIn('Подробности прошлой попытки не сохранены', MAIN)

    def test_sandbox_unmodified(self):
        sandbox = (ROOT / "src/sandbox.rs").read_text(encoding="utf-8")
        self.assertIn("SHARKCODE_ALLOW_UNSAFE_NATIVE", sandbox)
        self.assertIn("#[cfg(not(debug_assertions))]", sandbox)


if __name__ == "__main__":
    unittest.main()
