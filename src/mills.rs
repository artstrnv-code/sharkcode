//! «Мельницы стариков»: кортежи, словари, босс и необязательный сундук.
use super::*;
pub(super) fn first(stages: &[Stage]) -> usize {
    stages.iter().position(|s| s.id == "mills_entry").expect("mills_entry is missing")
}
fn next(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13; *seed ^= *seed >> 7; *seed ^= *seed << 17;
    *seed
}
fn choose(seed: &mut u64, count: usize) -> usize { (next(seed) % count as u64) as usize }
fn node(id: &str, cases: Vec<Value>) -> Stage {
    let mut stage = Stage {
        label:id.into(), lore:String::new(), task:String::new(),
        kind: Encounter::GrotQuest, expected:String::new(), id:id.into(),
        node_type:String::new(), validator:String::new(), pep8_coef:1.0,
        strokes_coef:1.0, reward:None, input:String::new(),
        computation_required:false, text_tolerance:false,
    }.with_content();
    stage.input = serde_json::to_string(&cases).unwrap_or_default();
    stage
}
fn test(inputs: Vec<String>, expected: String) -> Value {
    json!({"inputs": inputs, "expected":expected})
}
fn tuple_monkey(seed:&mut u64)->Stage {
    let cases=(0..5).map(|i| {
        let animals=["обезьяна","кот","волк","ёж","хлеб","слон"];
        let words=(0..10).map(|j| {
            if (i==0 && j<3) || (i!=0 && choose(seed,4)==0) {"обезьяна"}
            else {animals[1+choose(seed,animals.len()-1)]}
        }).collect::<Vec<_>>();
        test(vec![words.join(" ")],words.iter().filter(|x| **x=="обезьяна").count().to_string())
    }).collect();
    node("mills_monkeys",cases)
}
fn tuple_elf(seed:&mut u64)->Stage {
    let cases=(0..1).map(|_| {
        let words=["хлеб","ель","кот","сон","волк","мир"];
        let parts=(0..3).map(|_|words[choose(seed,words.len())].to_string()).collect::<Vec<_>>();
        let mut result=parts.clone();result.push("эльф".into());
        test(vec![parts.join(" ")],format!("({})",result.iter().map(|x|format!("'{x}'")).collect::<Vec<_>>().join(", ")))
    }).collect(); node("mills_elf",cases)
}
fn tuple_lovelace(seed:&mut u64)->Stage {
    let cases=(0..4).map(|_| {
        let inputs=(0..4).map(|_|(choose(seed,100) as i64-40).to_string()).collect::<Vec<_>>();
        let last=inputs[3].clone();test(inputs,format!("('{0}', '{0}', '{0}')",last))
    }).collect();node("mills_lovelace",cases)
}
fn tuple_specialties(seed:&mut u64)->Stage {
    let cases=(0..4).map(|_| {
        let nums=(0..6).map(|_|choose(seed,100) as i64-50).collect::<Vec<_>>();
        test(nums.iter().map(ToString::to_string).collect(),nums.iter().sum::<i64>().to_string())
    }).collect();node("mills_specialties",cases)
}
fn dict_names()->Stage {
    let mapping=[("вася","ТОЛСТЯК"),("Вася","тощяк"),("ВАСЯ","ЖиРдЯй"),("вАСЯ","просто вася")];
    let mut cases=mapping.iter().map(|(a,b)|test(vec![a.to_string()],b.to_string())).collect::<Vec<_>>();
    cases.push(json!({"inputs":["john"],"unknown_name":true}));
    node("mills_names",cases)
}
fn dict_letters(seed:&mut u64)->Stage {
    let from="aeop cxyk".replace(' ',"");
    let to=["а","е","о","р","с","х","у","к"];
    let cases=(0..6).map(|i| {
        let word=if i==0 {"copy".into()} else if i==1 {"apple".into()}
            else {(0..(4+choose(seed,6))).map(|_| {
                let abc="abcdefghijklmnopqrstuvwxyz";
                abc.as_bytes()[choose(seed,abc.len())] as char
            }).collect::<String>()};
        let expected=word.chars().map(|ch|from.find(ch).map_or("_",|j|to[j])).collect::<String>();
        test(vec![word],expected)
    }).collect();node("mills_lookalike",cases)
}
fn boss(seed:&mut u64)->Stage {
    // Выполнить без чтения содержимого можно экспериментом с хешируемостью.
    // Генерируем ровно один кортеж на весь бой; вложенность и список случайны.
    let has_list=choose(seed,2)==1;
    let mut values=vec!["5".to_string(),"'мельница'".to_string(),"(2, 'свет')".to_string()];
    if has_list {values[choose(seed,3)]=if choose(seed,2)==0 {"[4, 8]".into()}else{"(2, [3])".into()};}
    for i in (1..values.len()).rev(){let j=choose(seed,i+1);values.swap(i,j);}
    let expr=format!("({},)",values.join(", "));
    node("mills_boss",vec![json!({"inputs":[],"cortezh":expr,"expected":if has_list{"yes"}else{"no"}})])
}
pub(super) fn build(seed:u64)->Vec<Stage> {
    let mut seed=seed ^ 0x13FA_D20B_C789_0047;
    if seed==0 {seed=0x9E37_79B9_7F4A_7C15;}
    let mut result=vec![node("mills_entry",vec![]),node("mills_tuples",vec![])];
    // Шесть видов противников; порядок один раз перемешивается в начале забега.
    let mut ids=[0,1,2,3,4,5];
    for i in (1..ids.len()).rev(){let j=choose(&mut seed,i+1);ids.swap(i,j);}
    for id in ids {result.push(match id {
        0=>tuple_monkey(&mut seed),1=>tuple_elf(&mut seed),2=>tuple_lovelace(&mut seed),
        3=>node("mills_clone",vec![]),4=>tuple_specialties(&mut seed),
        _=>node("mills_scalar",vec![json!({"inputs":[]})])
    });}
    result.push(node("mills_dicts",vec![]));
    result.push(dict_names());result.push(dict_letters(&mut seed));
    result.push(node("mills_good_bad",vec![json!({"inputs":[]})]));
    result.push(boss(&mut seed));
    result.push(node("mills_chest",vec![json!({"inputs":[]})]));
    result.push(node("mills_clear",vec![]));
    // Будущие направления на карте: пока недоступны, задания ещё не разработаны.
    result.push(node("mills_to_prison",vec![]));
    result.push(node("mills_to_matrix",vec![]));
    result
}
