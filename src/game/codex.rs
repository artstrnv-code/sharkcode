// Текст страниц Кодекса. Отдельный impl сохраняет прежний API Game.
impl Game {
    fn codex_text(&self) -> String {
        match self.codex_page {
            CodexPage::Base => {
                if self.codex.contains("База") {
                    format!("База\n\n{}", TUTORIAL)
                } else {
                    "Скрижаль ещё не найдена.".into()
                }
            }
            CodexPage::BestAssignment => {
                if self.codex.contains("Лучшее присваивание") {
                    format!("Лучшее присваивание\n\n{}", ASSIGNMENT)
                } else {
                    "Скрижаль «Лучшее присваивание» ещё не найдена.".into()
                }
            }
            CodexPage::Libraries1 => include_str!("../../content/scrolls/libraries_part1.txt").into(),
            CodexPage::Libraries2 => include_str!("../../content/scrolls/libraries_part2.txt").into(),
            CodexPage::RiverBranching => include_str!("../../content/scrolls/river_branching.txt").into(),
            CodexPage::RiverComparison => include_str!("../../content/scrolls/river_comparison.txt").into(),
            CodexPage::RiverLogic => include_str!("../../content/scrolls/river_logic.txt").into(),
            CodexPage::ForBase => include_str!("../../content/scrolls/for_base.txt").into(),
            CodexPage::ForAdvanced => include_str!("../../content/scrolls/for_advanced.txt").into(),
            CodexPage::TowerWhile => include_str!("../../content/scrolls/tower_while.txt").into(),
            CodexPage::TowerLore => include_str!("../../content/scrolls/tower_lore.txt").into(),
            CodexPage::Walrus => include_str!("../../content/scrolls/walrus.txt").into(),
            CodexPage::Notes0 => include_str!("../../content/scrolls/notes0.txt").into(),
            CodexPage::Notes1 => include_str!("../../content/scrolls/notes1.txt").into(),
            CodexPage::MatchCase => include_str!("../../content/scrolls/match_case.txt").into(),
            CodexPage::FieldLists1 => include_str!("../../content/scrolls/field_lists1.txt").into(),
            CodexPage::FieldLists2 => include_str!("../../content/scrolls/field_lists2.txt").into(),
            CodexPage::FieldAdvanced => include_str!("../../content/scrolls/field_advanced.txt").into(),
            CodexPage::FieldAdvanced2 => include_str!("../../content/scrolls/field_advanced2.txt").into(),
            CodexPage::MillsTuples => include_str!("../../content/scrolls/mills_tuples.txt").into(),
            CodexPage::MillsDicts => include_str!("../../content/scrolls/mills_dicts.txt").into(),
            CodexPage::IfFunctions => include_str!("../../content/scrolls/if_functions.txt").into(),
            CodexPage::IfLambda => include_str!("../../content/scrolls/if_lambda.txt").into(),
            CodexPage::IfSets => include_str!("../../content/scrolls/if_sets.txt").into(),
            CodexPage::RoofRandom => include_str!("../../content/scrolls/roof_random.txt").into(),
            CodexPage::KnightsIntro => include_str!("../../content/scrolls/knights_intro.txt").into(),
            CodexPage::KnightsNames => include_str!("../../content/scrolls/knights_names.txt").into(),
            CodexPage::KnightsComments => include_str!("../../content/scrolls/knights_comments.txt").into(),
            CodexPage::KnightsSpaces => include_str!("../../content/scrolls/knights_spaces.txt").into(),
            CodexPage::KnightsNewline => include_str!("../../content/scrolls/knights_newline.txt").into(),
            CodexPage::KnightsConditions => include_str!("../../content/scrolls/knights_conditions.txt").into(),
        }
    }
}
