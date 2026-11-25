//! Procedural River of the Holy Forest encounter generator.
//!
//! Encounter order and probabilities mirror the original PROJECT PY riverRun():
//! start -> mandatory tutorial -> mandatory Mumbler -> four random nodes
//! (25% side tutorial / 75% enemy) -> optional elite (18%) -> boss -> clear.
//! Every random decision depends only on the run seed so saved clears remain
//! attached to the same node indices when the game is reopened.

use super::*;

pub(super) const RIVER_FIRST: usize = 20;

fn random(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn choose(seed: &mut u64, count: u64) -> u64 { random(seed) % count }
fn test(inputs: &[String], expected: impl Into<String>) -> Value {
    json!({"inputs": inputs, "expected": expected.into()})
}
fn with_tests(id: &str, cases: Vec<Value>) -> Stage {
    let mut stage = Stage {
        label: id.into(), lore: String::new(), task: String::new(),
        kind: Encounter::GrotQuest, expected: String::new(), id: id.into(),
        node_type: String::new(), validator: String::new(), pep8_coef: 1.0,
        strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false,
    }.with_content();
    stage.input = serde_json::to_string(&cases).unwrap_or_default();
    stage
}
fn simple(id: &str) -> Stage { with_tests(id, Vec::new()) }

fn mumbler(seed: &mut u64) -> Stage {
    let greeting_first = choose(seed, 2) == 0;
    let greeting = test(&["привет!".into()], "hi");
    let strange = test(&["абракадабра".into()], "hgfhgf");
    let cases = if greeting_first {vec![greeting, strange]} else {vec![strange, greeting]};
    with_tests("river_mumbler", cases)
}
fn cane(seed: &mut u64) -> Stage {
    let mut large = Vec::new();
    for _ in 0..3 {large.push((11 + choose(seed, 90)).to_string());}
    let small = vec![(1 + choose(seed, 10)).to_string(),
                     (11 + choose(seed, 90)).to_string(),
                     (1 + choose(seed, 10)).to_string()];
    let mut cases = vec![test(&large, "это сложная задача"),
                         test(&small, "легкотня")];
    if choose(seed, 2) == 0 {cases.reverse();}
    with_tests("river_cane", cases)
}
fn secret(seed: &mut u64) -> Stage {
    // Fix carried forward from the original locations.js: nine identical
    // integers plus exactly one different integer. Do not use the flawed
    // initial enemies_base.js generator (which could make multiple uniques).
    let mut cases = Vec::new();
    for _ in 0..2 {
        let unique = 1 + choose(seed, 10);
        let mut filler = 1 + choose(seed, 10);
        while filler == unique { filler = 1 + choose(seed, 10); }
        let mut values = vec![filler.to_string(); 10];
        values[choose(seed, 10) as usize] = unique.to_string();
        cases.push(test(&values, unique.to_string()));
    }
    with_tests("river_secret", cases)
}
fn elite() -> Stage {
    // Original six test cases and outputs from locations.js.
    with_tests("river_arcarh_elite", vec![
        test(&["12".into(), "0".into(), "/".into()], "ошибка"),
        test(&["24".into(), "6".into(), "/".into()], "4"),
        test(&["-7".into(), "11".into(), "+".into()], "4"),
        test(&["9".into(), "4".into(), "-".into()], "5"),
        test(&["8".into(), "7".into(), "*".into()], "56"),
        test(&["3".into(), "5".into(), "?".into()], "неизвестный оператор"),
    ])
}
fn boss(seed: &mut u64) -> Stage {
    let mut cases = Vec::new();
    for _ in 0..4 {
        let a = 1 + choose(seed, 100) as i64;
        let b = 1 + choose(seed, 100) as i64;
        let operation = ["+", "-", "*"][choose(seed, 3) as usize];
        let expected = match operation {"+"=>a+b,"-"=>a-b,_=>a*b};
        cases.push(test(&[a.to_string(), b.to_string(), operation.into()], expected.to_string()));
    }
    with_tests("river_boss", cases)
}

pub(super) fn build_river_stages(run_seed: u64) -> Vec<Stage> {
    // Distinct stream from the Valley generator: no change to Valley layout.
    let mut seed = run_seed ^ 0xD6E8FEB86659FD93;
    if seed == 0 { seed = 0x9E3779B97F4A7C15; }
    let mut stages = vec![simple("river_tutorial_base"), mumbler(&mut seed)];
    for _ in 0..4 {
        let stage = if choose(&mut seed, 4) == 0 {
            if choose(&mut seed, 2) == 0 { simple("river_tutorial_comparison") }
            else { simple("river_tutorial_logic") }
        } else {
            match choose(&mut seed, 3) {
                0 => mumbler(&mut seed),
                1 => cane(&mut seed),
                _ => secret(&mut seed),
            }
        };
        stages.push(stage);
    }
    if choose(&mut seed, 100) < 18 { stages.push(elite()); }
    stages.push(boss(&mut seed));
    // Все 3^4 комбинации проверяются независимо от генератора обычных врагов.
    let mut mimic_tests = Vec::new();
    for a in 1..=3 { for b in 1..=3 { for c in 1..=3 { for d in 1..=3 {
        mimic_tests.push(json!({"inputs":[a.to_string(),b.to_string(),c.to_string(),d.to_string()]}));
    }}}}
    stages.push(with_tests("river_mimic_conditions", mimic_tests));
    stages.push(simple("river_clear"));
    stages
}
