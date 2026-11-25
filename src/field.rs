//! «Великая пашня» — перенос locations.js::fieldRun().
//! Шесть встреч, скрижаль, четыре встречи, дополнительная скрижаль,
//! последняя встреча и босс. Seed независим от предыдущих регионов.
use super::*;

pub(super) fn first(stages: &[Stage]) -> usize {
    stages.iter().position(|s| s.id == "field_entry").expect("field_entry is missing")
}
fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}
fn choose(seed: &mut u64, count: usize) -> usize { (next(seed) % count as u64) as usize }
fn test(inputs: &[String], expected: impl Into<String>) -> Value {
    json!({"inputs": inputs, "expected": expected.into()})
}
fn node(id: &str, cases: Vec<Value>) -> Stage {
    let mut stage = Stage {
        label:id.into(), lore:String::new(), task:String::new(),
        kind:Encounter::GrotQuest, expected:String::new(), id:id.into(),
        node_type:String::new(), validator:String::new(), pep8_coef:1.0,
        strokes_coef:1.0, reward:None, input:String::new(), computation_required: false, text_tolerance: false,
    }.with_content();
    stage.input = serde_json::to_string(&cases).unwrap_or_default();
    stage
}
pub(super) fn from_for4() -> Stage { node("for4_to_field", vec![]) }
pub(super) fn from_tower() -> Stage { node("tower_to_field", vec![]) }

fn caw(seed: &mut u64, armored: bool) -> Stage {
    let words = ["caw","leaf","grass","weed","herb","fern","moss","reed"];
    let id = if armored {"field_caw_armored"} else {"field_caw"};
    let middle = if armored {3} else {2};
    let cases = (0..3).map(|_| {
        let word = words[choose(seed,words.len())];
        let repeat = word.repeat(2);
        let mut items = vec![repeat.clone();if armored {7}else{6}];
        items[middle] = word.into();
        test(&[word.into()], format!("{:?}",items).replace('"',"'"))
    }).collect();
    node(id,cases)
}
fn median(seed:&mut u64) -> Stage {
    let cases = (0..4).map(|_| {
        let count = choose(seed,4)+4;
        let numbers:Vec<i32> = (0..count).map(|_|choose(seed,10) as i32).collect();
        let mut sorted = numbers.clone();
        sorted.sort();
        let median = if count%2==1 {sorted[count/2] as f64}
            else {(sorted[count/2-1]+sorted[count/2]) as f64/2.0};
        let expected = if median.fract()==0.0 {format!("{}",median as i32)}
            else {median.to_string()};
        test(&[numbers.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")],expected)
    }).collect();
    node("field_median",cases)
}
fn evil_eye(seed:&mut u64) -> Stage {
    let cases = (0..3).map(|_| {
        let count=choose(seed,4)+2;
        let mut inputs=vec!["продолжай".to_string();count];
        inputs.push("стоп".into());
        test(&inputs,format!("[{}]",vec!["'хватит'";count].join(", ")))
    }).collect();
    node("field_evil_eye",cases)
}
fn cherry() -> Stage {
    node("field_cherry", vec![test(&[],"['черно', 'тьма', 'вишня', 'знамя', 'арбуз', 'хлеб']")])
}
fn ghost(seed:&mut u64) -> Stage {
    let numbers:Vec<usize> = (0..5).map(|_|choose(seed,100)).collect();
    let mut stage=node("field_ghost", vec![test(&[],numbers.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"))]);
    // Исправлена неполнота оригинала: числа генерировались, но не сообщались игроку.
    stage.task.push_str(&format!("\nЧисла для твоего массива: {}.",numbers.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")));
    stage
}
fn slippers(seed:&mut u64) -> Stage {
    let cases = (0..3).map(|_| {
        let min_num=choose(seed,5)+1;
        let dust_count=min_num*(choose(seed,4)+1);
        let mut inputs=vec!["пыль".to_string();dust_count];
        inputs.push(min_num.to_string());
        for _ in 0..(choose(seed,3)+2) {inputs.push((choose(seed,10)+min_num).to_string());}
        let trash=["грязь","мусор","хлам","сор"];
        for _ in 0..(choose(seed,3)+1) {inputs.push(trash[choose(seed,4)].into());}
        // Детерминированно перемешиваем, сохраняя условие «боже» последним.
        for i in (1..inputs.len()).rev() {let j=choose(seed,i+1);inputs.swap(i,j);}
        inputs.push("боже".into());
        test(&inputs,if (dust_count/min_num)%2==0 {"четное"}else{"нечетное"})
    }).collect();
    node("field_slippers",cases)
}
fn enemy(seed:&mut u64) -> Stage {
    match choose(seed,7) {
        0=>caw(seed,false),1=>caw(seed,true),2=>median(seed),
        3=>evil_eye(seed),4=>cherry(),5=>ghost(seed),_=>slippers(seed)
    }
}
fn boss(seed:&mut u64) -> Stage {
    let cases=(0..4).map(|_| {
        let numbers:Vec<i32>=(0..7).map(|_|choose(seed,80) as i32-20).collect();
        let mut sorted=numbers.clone();sorted.sort();
        test(&[numbers.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")],
            format!("{:?}",sorted))
    }).collect();
    node("field_boss",cases)
}
pub(super) fn build(seed:u64) -> Vec<Stage> {
    let mut seed=seed ^ 0x5DC7_29A8_713E_4F02;
    if seed==0 {seed=0x9E37_79B9_7F4A_7C15;}
    let mut stages=vec![node("field_entry",vec![]),node("field_lists1",vec![])];
    for _ in 0..6 { stages.push(enemy(&mut seed)); }
    stages.push(node("field_lists2",vec![]));
    for _ in 0..4 { stages.push(enemy(&mut seed)); }
    stages.push(node(if choose(&mut seed,2)==0 {"field_advanced"}else{"field_advanced2"},vec![]));
    stages.push(enemy(&mut seed));
    stages.push(boss(&mut seed));
    stages.push(node("field_new_chest",vec![json!({"inputs":[]})]));
    stages.push(node("field_clear",vec![]));
    stages.push(node("field_to_mills",vec![]));
    stages
}
