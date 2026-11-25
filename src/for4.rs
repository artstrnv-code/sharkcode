//! Original PROJECT PY region "4 4": for / range.
//!
//! Layout matches locations.js::forRun(): mandatory tutorial, four random
//! encounters (20% advanced scroll; otherwise a member of enemyPoolFor), boss.
//! This region uses its own deterministic random stream, preserving the seed
//! of the Valley and River and enabling exact save restoration.
use super::*;

pub(super) fn river_to_for4(stages: &[Stage]) -> usize {
    stages.iter().position(|stage| stage.id == "river_to_for4")
        .expect("river_to_for4 entry missing")
}
pub(super) fn for4_first(stages: &[Stage]) -> usize { river_to_for4(stages) + 1 }

fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn choose(seed: &mut u64, range: u64) -> u64 { next(seed) % range }
fn test(inputs: &[String], expected: impl Into<String>) -> Value {
    json!({"inputs":inputs,"expected":expected.into()})
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

pub(super) fn build_river_to_for4() -> Stage { node("river_to_for4", vec![]) }

fn greeting(seed: &mut u64) -> Stage {
    // The original source produces thirty Cyrillic characters; each
    // occurrence of 'п' requires one response 'привет'.
    let alphabet: Vec<char> = "абвгдежзийклмнопрстуфхцчшщъыьэюя".chars().collect();
    let mut cases = Vec::new();
    for _ in 0..2 {
        let mut value = String::new();
        let mut count = 0;
        for _ in 0..30 {
            let character = if choose(seed, 5) == 0 { 'п' }
                else { alphabet[choose(seed, alphabet.len() as u64) as usize] };
            if character == 'п' { count += 1; }
            value.push(character);
        }
        cases.push(test(&[value], vec!["привет";count].join("\n")));
    }
    node("for4_greeting", cases)
}
fn carousel() -> Stage {
    // The original checks token multiplicities, not output order.
    node("for4_carousel", vec![test(&[], "9×привет; 6×пока; 3×ппаоа")])
}
fn kemerovo() -> Stage { node("for4_kemerovo", vec![test(&[], "2550")]) }
fn accountant(seed: &mut u64) -> Stage {
    let mut cases = Vec::new();
    for _ in 0..2 {
        let mut digits = String::new();
        let mut total = 0u32;
        for _ in 0..10 {
            let digit = choose(seed, 10) as u32;
            digits.push(char::from_digit(digit, 10).unwrap());
            total += digit;
        }
        cases.push(test(&[digits], total.to_string()));
    }
    node("for4_accountant", cases)
}
fn boss() -> Stage {
    // Original ordering: a=1..20 outside, b=1..20 inside, repeated products
    // are intentionally printed again; total 400 lines.
    let output = (1..=20).flat_map(|a| (1..=20).map(move |b| (a*b).to_string()))
        .collect::<Vec<_>>().join("\n");
    node("for4_boss", vec![test(&[],output)])
}

pub(super) fn build_for4_stages(run_seed: u64) -> Vec<Stage> {
    let mut seed = run_seed ^ 0x8FD2_7B0C_4A93_661D;
    if seed == 0 { seed = 0x9E37_79B9_7F4A_7C15; }
    let mut stages = vec![node("for4_entry", vec![]), node("for4_tutorial_base", vec![])];
    for _ in 0..4 {
        let next_stage = if choose(&mut seed, 5) == 0 {
            let mut scroll_seed = seed ^ 0x8A71_C19B_28D4_3042;
            let scroll = match choose(&mut scroll_seed, 3) {
                0 => "for4_tutorial_advanced",
                1 => "for4_tutorial_walrus",
                _ => "for4_tutorial_notes1",
            };
            node(scroll, vec![])
        } else {
            match choose(&mut seed, 4) {
                0 => greeting(&mut seed),
                1 => carousel(),
                2 => kemerovo(),
                _ => accountant(&mut seed),
            }
        };
        stages.push(next_stage);
    }
    stages.push(boss());
    // Побочный сундук не входит в обязательный маршрут региона.
    stages.push(node("for4_mimic_loops", vec![test(&[], "числа 1..100 в любой форме") ]));
    stages.push(node("for4_clear", vec![]));
    stages
}
