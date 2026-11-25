//! Бесконечная башня. Перенос оригинального `locations.js::towerRun()`.
//! Две обязательные скрижали, четыре случайных встречи (20% скрижаль), Гараон.
//! Все случайные решения воспроизводятся по seed полного забега.
use super::*;

pub(super) fn river_to_tower(stages: &[Stage]) -> usize {
    stages.iter().position(|stage| stage.id == "river_to_tower")
        .expect("river_to_tower entry missing")
}
pub(super) fn tower_first(stages: &[Stage]) -> usize { river_to_tower(stages) + 1 }

fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn choose(seed: &mut u64, len: u64) -> u64 { next(seed) % len }
fn test(inputs: &[String], output: impl Into<String>) -> Value {
    json!({"inputs": inputs, "expected": output.into()})
}
fn node(id: &str, cases: Vec<Value>) -> Stage {
    let mut stage = Stage {
        label: id.into(), lore: String::new(), task: String::new(),
        kind: Encounter::GrotQuest, expected: String::new(), id: id.into(),
        node_type: String::new(), validator: String::new(), pep8_coef: 1.0,
        strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false,
    }.with_content();
    stage.input = serde_json::to_string(&cases).unwrap_or_default();
    stage
}
pub(super) fn build_river_to_tower() -> Stage { node("river_to_tower", vec![]) }

fn joker(seed: &mut u64) -> Stage {
    let mut cases = Vec::new();
    for _ in 0..2 {
        let mut inputs = Vec::new();
        let mut count = 0;
        for _ in 0..49 {
            let value = if choose(seed, 2) == 0 { "1" } else { "2" };
            if value == "2" { count += 1; }
            inputs.push(value.to_owned());
        }
        inputs.push("3".into());
        let mut lines = vec!["two pizza"; count];
        lines.push("неверно!");
        cases.push(test(&inputs, lines.join("\n")));
    }
    node("tower_joker", cases)
}
fn fortune(seed: &mut u64) -> Stage {
    let cases = (0..3).map(|_| {
        let number = choose(seed, 50) + 1;
        test(&[number.to_string()], "чудеса")
    }).collect();
    node("tower_fortune", cases)
}
fn novice() -> Stage {
    node("tower_novice", vec![test(&["нет".into(), "нет".into(), "да".into()], "ура!")])
}
fn echo_dragon() -> Stage {
    node("tower_echo_dragon", vec![test(&[
        "бух".into(), "бухбухбухбух".into(), "бухбух".into(),
    ], "бухбух\nбухбух\nтрпрпр\nбам")])
}
fn garaon() -> Stage {
    node("tower_garaon", vec![test(&[], "бух\nбух\nбух\nбух\nбух")])
}

pub(super) fn build_tower_stages(run_seed: u64) -> Vec<Stage> {
    let mut seed = run_seed ^ 0x7A16_C423_8D1B_502F;
    if seed == 0 { seed = 0xA7F2_153D_2B15_4C07; }
    let mut stages = vec![
        node("tower_entry", vec![]),
        node("tower_tutorial_while", vec![]),
        node("tower_tutorial_lore", vec![]),
    ];
    for _ in 0..4 {
        let stage = if choose(&mut seed, 5) == 0 {
            let mut scroll_seed = seed ^ 0x0F73_B44C_59E2_18A6;
            let scroll = match choose(&mut scroll_seed, 3) {
                0 => "tower_tutorial_comparison",
                1 => "tower_tutorial_walrus",
                _ => "tower_tutorial_notes1",
            };
            node(scroll, vec![])
        } else {
            match choose(&mut seed, 4) {
                0 => joker(&mut seed),
                1 => fortune(&mut seed),
                2 => novice(),
                _ => echo_dragon(),
            }
        };
        stages.push(stage);
    }
    stages.push(garaon());
    // Необязательный мимик со специальной наградой.
    stages.push(node("tower_mimic_surrender", vec![
        test(&["нет".into()], "100 раз я не сдамся"),
        test(&["да".into()], "100 раз я не сдамся; сдаюсь"),
    ]));
    stages.push(node("tower_clear", vec![]));
    stages
}
