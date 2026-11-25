//! Procedural Valley encounters, its optional Grot branch and quiz metadata.
use crate::{Encounter, Stage};
use std::{collections::HashSet, fs};

pub(super) fn build_valley_stages(initial_seed: u64) -> Vec<Stage> {
    let mut possible = [Encounter::Counter, Encounter::Wise, Encounter::Egg, Encounter::Statue];
    // Простое перемешивание без внешней библиотеки — внутри забега враги не повторяются.
    let mut seed = if initial_seed == 0 { 0x9E3779B97F4A7C15 } else { initial_seed };
    for i in (1..possible.len()).rev() {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
        possible.swap(i, seed as usize % (i + 1));
    }
    let mut result = vec![Stage { label: "Учебная скрижаль".into(),
        lore: "База".into(), task: "Прочитай скрижаль по желанию.".into(),
        kind: Encounter::Tutorial, expected: String::new(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false  },
        Stage { label: "Стесняга".into(),
        lore: "Он милый парень, но социопат. Поздоровайся с ним.".into(),
        task: "Выведи: Hello world!".into(), kind: Encounter::Shyguy, expected: "Hello world!".into(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false  }];
    // В четырёх случайных нодах три старых врага + новый «Игрок в карты».
    // Порядок перемешивается вместе с остальными встречами.
    let mut choices = [possible[0], possible[1], possible[2], Encounter::CardPlayer];
    for i in (1..choices.len()).rev() {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
        choices.swap(i, seed as usize % (i + 1));
    }
    for kind in choices { result.push(random_stage(kind, &mut seed)); }
    result.push(Stage { label: "Эхозмий".into(),
        lore: "Он повторяет всё, что слышит, но делает это с задержкой…".into(),
        task: "Выведи 7 строк: 'бух', 'бухбух', 'бухбухбух' и так далее до семи повторов.".into(),
        kind: Encounter::Echo, expected: (1..=7).map(|i| "бух".repeat(i)).collect::<Vec<_>>().join("\n"), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false  });
    result.push(Stage { label: "Старинный сундук".into(),
        lore: "Он ожидает вас...".into(),
        task: "задайте переменные a = 10, b = \"hello\". (Именна обязательно такие) поменяйте их местами (a = \"hello\", b = 10). ничего не выводите".into(),
        kind: Encounter::Chest, expected: String::new(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false  });
    result.push(Stage { label: "Грот веры".into(),
        lore: "Секретная ветвь Стесняги".into(),
        task: "Секреты импортов и устройства Python-библиотек.".into(),
        kind: Encounter::Grot, expected: String::new(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false  });
    let mut result: Vec<Stage> = result.into_iter().map(Stage::with_content).collect();
    // Branch follows the original Grot entrance. The valley completion is a separate node.
    for name in ["grot_scroll1", "grot_lover", "grot_reader", "grot_destroyer",
                 "grot_scroll2", "grot_boss", "grot_clear", "valley_clear", "river_entry",
                 "valley_to_river", "grot_to_river"] {
        result.push(Stage {
            label: name.into(), lore: String::new(), task: String::new(),
            kind: Encounter::GrotQuest, expected: String::new(), id: name.into(),
            node_type: String::new(), validator: String::new(), pep8_coef: 1.0,
            strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false,
        }.with_content());
    }
    result
}

fn random_stage(kind: Encounter, seed: &mut u64) -> Stage {
    let next = |s: &mut u64| { *s ^= *s << 13; *s ^= *s >> 7; *s ^= *s << 17; *s };
    let number = |s: &mut u64| 1 + (next(s) % 10) as u32;
    match kind {
        Encounter::Counter => {
            let coins = [("пятак", 5i64), ("четвертак", 4), ("серебряник", 50), ("золотник", 100)];
            let i = (next(seed) % 4) as usize;
            let j = (i + 1 + (next(seed) % 3) as usize) % 4;
            Stage { label: "Счётчик".into(), lore: "Он любит считать монеты. Посчитай их.".into(),
                task: format!("Номиналы: пятак — 5, четвертак — 4, серебряник — 50, золотник — 100. Сложи {} и {}. Выведи результат.", coins[i].0, coins[j].0),
                kind, expected: (coins[i].1 + coins[j].1).to_string(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false }
        }
        Encounter::Wise => {
            let x = number(seed); let y = number(seed); let z = number(seed);
            let op = ['+', '-', '*'][(next(seed) % 3) as usize];
            let base = (x as i64).pow(y);
            let expected = match op { '+' => base + z as i64, '-' => base - z as i64, _ => base * z as i64 };
            Stage { label: "Мудрец".into(), lore: "У него много мудрости! Но не интеллекта.".into(),
                task: format!("Посчитай {x}**{y}{op}{z}. Выведи результат."), kind, expected: expected.to_string(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false }
        }
        Encounter::Egg => Stage { label: "Яйцо змей".into(),
            lore: "Он стучит своим хвостом, когда стучишь ты!".into(),
            task: "Выведи в консоль \"бух бух бух \" (с пробелом в конце).".into(),
            kind, expected: "бух бух бух ".into(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false },
        Encounter::CardPlayer => Stage { label: "Игрок в карты".into(),
            lore: "Враг-тест: вместо написания кода нужно понять, что он сделает.".into(),
            task: "Что выведет код?\n\nprint(1, end=\"\")\nprint()\nprint(13455y31, end=\"\")\nprint(\"hello\")".into(),
            kind, expected: String::new(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false },
        _ => Stage { label: "Статуя".into(), lore: "Редкий враг. Посчитай мне мой пример!".into(),
            task: "Посчитай (1+2+3+4+5+6+7+7+8+8+9+10+11+12+13+14)**11.".into(),
            kind: Encounter::Statue, expected: (120u128.pow(11)).to_string(), id: String::new(), node_type: String::new(), validator: String::new(), pep8_coef: 1.0, strokes_coef: 1.0, reward: None, input: String::new(), computation_required: false, text_tolerance: false  },
    }
}

// Answers live in TOML; the previous card-player test remains compatible.
pub(super) fn quiz_info(id: &str) -> (Vec<String>, usize) {
    if let Ok(raw) = fs::read_to_string(format!("content/nodes/{id}.toml")) {
        if let Ok(parsed) = toml::from_str::<toml::Value>(&raw) {
            if let Some(game) = parsed.get("GAME") {
            if let (Some(options), Some(correct)) = (game.get("quiz_options").and_then(toml::Value::as_array),
                                                   game.get("correct_answer").and_then(toml::Value::as_integer)) {
                let values: Vec<String> = options.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect();
                if (2..=4).contains(&values.len()) && (0..values.len() as i64).contains(&correct) {return (values,correct as usize);}
            }
            }
        }
    }
    (vec!["1\n13455y31hello".into(), "1\n13455y31\nhello".into(),
          "1\"\"\n13455y31\"\"\nhello".into(), "Упадёт с ошибкой".into()],3)
}

// Legacy Valley/Grot routes retain their original indices and connections.
pub(super) fn can_enter(index: usize, cleared: &HashSet<usize>, grot_unlocked: bool, river_discovered: bool) -> bool {
    match index {
        0 => true,
        1..=6 => cleared.contains(&(index - 1)) || index == 1,
        7 => cleared.contains(&2),
        8 | 9 | 10 => grot_unlocked,
        11 => cleared.contains(&10),
        12 => cleared.contains(&11),
        13 | 14 => cleared.contains(&12),
        15 => cleared.contains(&14),
        16 => cleared.contains(&6),
        17 => river_discovered,
        18 => cleared.contains(&16),
        19 => cleared.contains(&15),
        _ => false,
    }
}
