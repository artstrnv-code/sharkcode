//! Разрушенные крыши: библиотеки и random; ответвление от Мельниц стариков.
use super::*;

pub(super) fn first(stages: &[Stage]) -> usize {
    stages.iter().position(|s| s.id == "roof_entry").expect("roof_entry missing")
}

fn node(id: &str, cases: Vec<Value>) -> Stage {
    let mut stage = Stage {
        label: String::new(), lore: String::new(), task: String::new(),
        kind: Encounter::GrotQuest, expected: String::new(), id: id.into(),
        node_type: String::new(), validator: String::new(),
        pep8_coef: 1.0, strokes_coef: 1.0, reward: None,
        input: String::new(), computation_required: false, text_tolerance: false,
    }.with_content();
    stage.input = serde_json::to_string(&cases).unwrap_or_default();
    stage
}

pub(super) fn exit() -> Stage { node("mills_to_roofs", vec![]) }

pub(super) fn build(_seed: u64) -> Vec<Stage> {
    vec![
        node("roof_entry", vec![]),
        node("roof_random", vec![]),
        // Тестовый конвейер для шариков передаёт ввод как последовательность строк.
        node("roof_balls", vec![
            json!({"inputs": ["зелёный", "красный", "синий", "жёлтый", ""]}),
            json!({"inputs": ["зелёный", "зелёный", "синий", "красный", ""]}),
            json!({"inputs": ["белый", ""]}),
            json!({"inputs": ["красный", "синий", "красный", "синий", ""]}),
        ]),
        node("roof_hero", vec![json!({"inputs": []})]),
        node("roof_boss", vec![json!({"inputs": []})]),
        node("roof_clear", vec![]),
    ]
}
