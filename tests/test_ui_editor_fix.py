"""Static regression checks for the Bevy UI wiring (not a Rust runtime test)."""
from pathlib import Path
import unittest

SOURCE = (Path(__file__).resolve().parents[1] / "src" / "main.rs").read_text(encoding="utf-8")


class EditorAndCodexWiring(unittest.TestCase):
    def test_editor_and_codex_have_independent_scroll_areas(self):
        self.assertIn("EditorPanel, Button, RelativeCursorPosition::default(), ScrollPosition::default()", SOURCE)
        self.assertIn("CodexListScroll, ScrollPosition::default(), RelativeCursorPosition::default()", SOURCE)
        self.assertIn("CodexScroll, ScrollPosition::default(), RelativeCursorPosition::default()", SOURCE)
        self.assertIn("fn editor_scroll(", SOURCE)

    def test_cursor_is_rendered_with_text_and_not_as_global_ui_node(self):
        self.assertIn("displayed.insert(buffer.cursor, '│');", SOURCE)
        self.assertNotIn("struct EditorCursor", SOURCE)
        self.assertIn("text.0 = display_code(&buffer)", SOURCE)

    def test_codex_is_on_top_of_separate_ui_hierarchies(self):
        self.assertIn("CodexPanel, bevy::ui::GlobalZIndex(100)", SOURCE)
        self.assertIn("if game.codex_open", SOURCE)

    def test_no_paste_in_game_editor(self):
        self.assertIn('"v"=>{},', SOURCE)
        self.assertNotIn("cb.get_text()", SOURCE)

    def test_vertical_navigation_and_follow_caret(self):
        self.assertIn("Key::PageUp", SOURCE)
        self.assertIn("Key::PageDown", SOURCE)
        self.assertIn("if row_y < scroll.y", SOURCE)

class CodexScrollRegression(unittest.TestCase):
    def test_codex_scroll_content_is_vertical_and_not_reassigned_unnecessarily(self):
        self.assertIn('overflow: Overflow::scroll_y(), flex_direction: FlexDirection::Column,', SOURCE)
        self.assertIn('if text.0 != next { text.0 = next; }', SOURCE)
        self.assertIn('flex_grow: 1.0, flex_basis: px(0)', SOURCE)
