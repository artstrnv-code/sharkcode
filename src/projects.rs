//! Optional project enemies, unlocked after the player has visited the Mills.
//! Appended after existing stages: old save indices are unchanged.
use super::*;

fn node(id: &str, cases: Vec<Value>) -> Stage {
    let mut stage = Stage {
        label: id.into(), lore: String::new(), task: String::new(),
        kind: Encounter::GrotQuest, expected: String::new(), id: id.into(),
        node_type: String::new(), validator: String::new(), pep8_coef: 1.0,
        strokes_coef: 1.0, reward: None, input: String::new(),
        computation_required: true, text_tolerance: false,
    }.with_content();
    stage.input = serde_json::to_string(&cases).unwrap_or_default();
    stage
}

fn numeral(mut number: u64, base: u64) -> String {
    const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    if number == 0 { return "0".into(); }
    let mut chars = Vec::new();
    while number != 0 {
        chars.push(DIGITS[(number % base) as usize] as char);
        number /= base;
    }
    chars.iter().rev().collect()
}

pub(super) fn build(seed: u64) -> Vec<Stage> {
    let bases = [2, 3, 7, 10, 12, 16, 26, 32, 35, 36];
    let nums = [0, 255, 104, 1, 360, 4095, 650, 1024, 1296, 12345];
    let cases = bases.iter().zip(nums.iter()).map(|(&base, &number)| {
        let number_text = numeral(number, base);
        json!({"inputs": [number_text, base.to_string()], "expected": number.to_string()})
    }).collect();
    // One opaque stdin payload per test, terminated by the first non-whitespace line.
    let targets = [65usize, 66, 67, 90, 97, 122];
    let void_cases = targets.iter().enumerate().map(|(i, &target)| {
        let tabs = target % 3;
        let remaining = target - tabs;
        let spaces = remaining % 2;
        let newlines = (remaining - spaces) / 2;
        let mut body = String::new();
        body.push_str(&"\t".repeat(tabs));
        body.push_str(&" ".repeat(spaces));
        body.push_str(&"\n".repeat(newlines));
        // Guarantee the final non-whitespace line cannot be counted.
        body.push_str("STOP\n");
        json!({"stdin":body, "expected":char::from_u32(target as u32).unwrap().to_string(), "variant":i})
    }).collect();
    let _ = seed;
    vec![node("project_converter", cases), node("project_void", void_cases)]
}
