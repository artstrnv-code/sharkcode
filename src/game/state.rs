#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CodexPage { Base, BestAssignment, Libraries1, Libraries2, RiverBranching, RiverComparison, RiverLogic, ForBase, ForAdvanced, TowerWhile, TowerLore, Walrus, Notes0, Notes1, MatchCase, FieldLists1, FieldLists2, FieldAdvanced, FieldAdvanced2, MillsTuples, MillsDicts, IfFunctions, IfLambda, IfSets, RoofRandom, KnightsIntro, KnightsNames, KnightsComments, KnightsSpaces, KnightsNewline, KnightsConditions }

#[derive(Component)] struct LocationMap(Location);
#[derive(Component)] struct LocationTitle;

#[derive(Resource)]
struct Game {
    stages: Vec<Stage>,
    selected: usize,
    cleared: HashSet<usize>,
    codex: HashSet<String>,
    // Once earned, membership survives a new run.
    knights_unlocked: bool,
    ever_visited_mills: bool,
    grot_unlocked: bool,
    message: String,
    runs: u32,
    attempt_id: u64,
    run_seed: u64,
    codex_open: bool,
    codex_page: CodexPage,
    // Лучший результат и количество ответов врага-теста в текущем забеге.
    quiz_attempts: HashMap<usize, u32>,
    ranks: HashMap<usize, String>,
    // Full result of a cleared encounter, restored when revisiting its node.
    finished_reports: HashMap<usize, String>,
    inventory: HashMap<String, u32>,
    dev_mode: bool,
    console_open: bool,
    console_output: String,
    inventory_open: bool,
    completion: Option<String>,
    new_scrolls_run: HashSet<String>,
    rare_won_run: bool,
    new_treasures_run: HashSet<String>,
    hp: u32,
    location: Location,
    river_discovered: bool,
    for4_discovered: bool,
    tower_discovered: bool,
    field_discovered: bool,
    mills_discovered: bool,
    if_discovered: bool,
    roofs_discovered: bool,
    // Only the most recently executed tests are visible through Run Debug.
    last_report: Option<Value>,
}

impl Game {
    fn new() -> Self {
        let seed = fresh_seed();
        let mut game = Self {
            stages: build_stages(seed), selected: 0, cleared: HashSet::new(),
            codex: HashSet::new(), knights_unlocked: false, ever_visited_mills: false, grot_unlocked: false,
            message: "Долина переменн. Прочтение учебной скрижали необязательно.".into(),
            runs: 0, attempt_id: 0, run_seed: seed, codex_open: false, codex_page: CodexPage::Base, quiz_attempts: HashMap::new(), ranks: HashMap::new(), finished_reports: HashMap::new(), inventory: HashMap::new(),
            dev_mode: false, console_open: false, console_output: "F12 — консоль. dev on — тестовый режим. help — команды.".into(),
            inventory_open: false, completion: None, new_scrolls_run: HashSet::new(), rare_won_run: false, new_treasures_run: HashSet::new(), hp: 5,
            location: Location::Valley, river_discovered: false, for4_discovered: false, tower_discovered: false, field_discovered: false, mills_discovered: false, if_discovered: false, roofs_discovered: false, last_report: None,
        };
        game.load_meta();
        // Первая скрижаль лежит на начальном узле: подбирается без обязательного чтения.
        if game.codex.insert("База".into()) { game.new_scrolls_run.insert("База".into()); }
        game.save_meta();
        game
    }

    // Only playable encounters become read-only after a victory.
    fn finished_encounter(&self, index: usize) -> bool {
        self.cleared.contains(&index)
            && matches!(self.stages[index].node_type.as_str(),
                "enemy.code" | "enemy.quiz" | "enemy.project" | "chest.code")
    }

    fn encounter_result(&self, index: usize) -> String {
        if let Some(report) = self.finished_reports.get(&index) {
            return report.clone();
        }
        // Old saves only have ranks; avoid inventing missing test details.
        let rank = self.ranks.get(&index).map(String::as_str).unwrap_or("—");
        format!("Враг пройден. Ранг: {rank}\nПодробности прошлой попытки не сохранены.")
    }

    fn unlock_knights(&mut self) {
        if self.knights_unlocked { return; }
        self.knights_unlocked = true;
        for name in ["PEP8 — письмо", "Рыцари: Переменные",
            "Рыцари: Комментарии и длины", "Рыцари: Пробелы",
            "Рыцари: Перевод строки", "Рыцари: Условия"] {
            if self.codex.insert(name.into()) {
                self.new_scrolls_run.insert(name.into());
            }
        }
    }

    fn show_debug(&mut self) {
        if self.inventory.get("debug_gun").copied().unwrap_or(0) == 0 {
            self.message = "Защитное ружьё ещё не получено.".into();
            return;
        }
        let Some(report) = &self.last_report else {
            self.message = "Run Debug: сначала запусти проверку в текущем ноде.".into();
            return;
        };
        let mut lines = vec!["[RUN DEBUG]".to_string()];
        if let Some(runs) = report["runs"].as_array() {
            for (index, run) in runs.iter().enumerate() {
                let inputs = run["inputs"].as_array()
                    .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
                    .unwrap_or_else(|| "(входные данные недоступны)".into());
                lines.push(format!("ТЕСТ {} · input: {}", index + 1, inputs));
                lines.push(format!("stdout: {}", serde_json::to_string(&run["stdout"]).unwrap_or_default()));
                if let Some(error) = run["error"].as_str() {
                    lines.push(format!("Python: {error}"));
                }
            }
        }
        self.message = lines.join("\n");
    }

    fn toggle_codex(&mut self) {
        self.codex_open = !self.codex_open;
    }

    fn open_codex_page(&mut self, page: CodexPage) {
        self.codex_page = page;
        self.codex_open = true;
    }

    fn attempt(&mut self, source: &str) {
        if self.finished_encounter(self.selected) {
            self.message = self.encounter_result(self.selected);
            return;
        }
        if self.stages[self.selected].node_type == "entry" {
            self.use_entry(source);
            return;
        }
        let stage = &self.stages[self.selected];
        if stage.node_type == "enemy.quiz" {
            self.message = "Это враг-тест. Выбери один из четырёх ответов под заданием.".into();
            return;
        }
        if matches!(stage.node_type.as_str(), "scroll" | "region.clear" | "empty") {
            self.message = "Это не боевой нод. Открой «Кодекс знаний».".into();
            return;
        }
        self.attempt_id += 1;
        let task_id = if matches!(stage.kind, Encounter::GrotQuest) { stage.id.as_str() }
            else { stage.task_id() };
        let request = json!({"source":source,"task":task_id,"tests":stage.tests(),
            "validator":stage.validator,"ranking":{"pep8_coef":stage.pep8_coef,"strokes_coef":stage.strokes_coef},
            "computation_required": stage.computation_required,
            "text_tolerance": stage.text_tolerance,
            "starter_source": if stage.id == "tower_garaon" { "x = True\nwhile x == True:\n    " } else { "" }});
        match evaluate(&request, self.attempt_id) {
            Ok(report) => {
                self.last_report = Some(report.clone());
                let grot = report.pointer("/analysis/grot_trigger")
                    .and_then(Value::as_bool).unwrap_or(false);
                if self.selected == 1 && grot {
                    self.grot_unlocked = true;
                }
                let result = &report["evaluation"];
                let passed = result["passed"].as_bool().unwrap_or(false);
                let rank = result["rank"].as_str().unwrap_or("—");
                let first_win = passed && !self.cleared.contains(&self.selected);
                let guild_invitation = passed && matches!(rank, "S" | "S++")
                    && !self.knights_unlocked;
                if passed {
                    let first_clear = self.cleared.insert(self.selected);
                    if self.selected == 7 {
                        if self.codex.insert("Лучшее присваивание".into()) {
                            self.new_scrolls_run.insert("Лучшее присваивание".into());
                        }
                    }
                    if first_clear {
                        let scroll_reward = match self.stages[self.selected].id.as_str() {
                            "for4_mimic_loops" => Some("Заметки №0"),
                            "river_mimic_conditions" => Some("match/case"),
                            "if_mimic" => Some("ИФМОСТРОЙ-14: Множества"),
                            _ => None,
                        };
                        if let Some(name) = scroll_reward {
                            if self.codex.insert(name.into()) {
                                self.new_scrolls_run.insert(name.into());
                            }
                        }
                        if let Some(item) = self.stages[self.selected].reward.clone() {
                            self.new_treasures_run.insert(item.clone());
                            *self.inventory.entry(item.clone()).or_insert(0) += 1;
                        }
                        if matches!(self.stages[self.selected].kind, Encounter::Statue) {
                            self.rare_won_run = true;
                        }
                    }
                    if self.selected == 6 && first_clear {
                        self.runs += 1;
                    }
                }
                if passed {
                    self.ranks.insert(self.selected, rank.to_string());
                    if matches!(rank, "S" | "S++") { self.unlock_knights(); }
                }
                let notes = result["diagnostics"].as_array().map(|items| {
                    items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n")
                }).unwrap_or_default();
                self.message = format_report(&report, passed, rank, &notes);
                if guild_invitation {
                    self.message.push_str("\nГильдия PEP8 прислала письма. Открой Кодекс знаний!");
                }
                if first_win {
                    if let Some(item) = self.stages[self.selected].reward.clone() {
                        self.message.push_str(&format!("\nНаграда: {}",item_name(&item)));
                    }
                    let scroll = match self.stages[self.selected].id.as_str() {
                        "for4_mimic_loops" => Some("Заметки №0"),
                        "river_mimic_conditions" => Some("match/case"),
                        "if_mimic" => Some("ИФМОСТРОЙ-14: Множества"),
                        _ => None,
                    };
                    if let Some(scroll) = scroll {
                        self.message.push_str(&format!("\nНовая скрижаль: {scroll}"));
                    }
                }
                if grot { self.message.push_str("\nГрот веры открыт!"); }
                if passed && self.selected == 7 {
                    self.message.push_str("\nПолучена скрижаль «Лучшее присваивание». Открой «Кодекс знаний».");
                }
                if first_win && (self.stages[self.selected].id == "river_arcarh_elite" || self.stages[self.selected].id == "for4_accountant" || self.stages[self.selected].id == "field_slippers") {
                    self.rare_won_run = true;
                }
                if first_win && self.selected == 6 {
                    self.message.push_str(&format!("\nПолных прохождений Долины: {}", self.runs));
                    if self.runs == 5 {
                        self.message.push_str("\nКто-то посильнее завёлся в Долине!");
                    }
                }
                if passed && first_win {
                    self.finished_reports.insert(self.selected, self.message.clone());
                }
                if !passed && result["infrastructure_error"].as_bool() != Some(true) {
                    if result["diagnostic_probe"].as_bool() == Some(true) {
                        self.message.push_str("
Проба без ответа: HP не расходуется.");
                    } else if result["instant_death"].as_bool() == Some(true) {
                        self.new_run();
                        self.message = "[GAME OVER] Аварн: неверный ответ. Ты сгнил в тюрьме. Новый забег начат.".into();
                        return;
                    } else { self.lose_hp(self.selected == 11); }
                }
                self.save_meta();
            }
            Err(err) => self.message = format!("Ошибка проверки: {err}"),
        }
    }

    fn answer_quiz(&mut self, answer: usize) {
        if self.stages[self.selected].node_type != "enemy.quiz" { return; }
        if self.cleared.contains(&self.selected) {
            self.message = self.encounter_result(self.selected);
            return;
        }
        let current = self.quiz_attempts.entry(self.selected).or_insert(0);
        *current += 1;
        let attempt = *current;
        let (options, correct_answer) = valley::quiz_info(&self.stages[self.selected].id);
        let correct = answer == correct_answer;
        let rank = match attempt {
            1 => "S++", 2 => "A", 3 => "B", 4 => "C", _ => "D",
        };
        self.attempt_id += 1;
        let report = json!({"task":"card_player", "attempt":attempt,
            "answer":answer,"correct":correct,"expected_answer":correct_answer,
            "rank":if correct {rank} else {"—"}});
        let dir = data_dir().join("session");
        let _ = fs::create_dir_all(&dir);
        let _ = fs::write(dir.join(format!("quiz_{:06}.json", self.attempt_id)),
            serde_json::to_vec_pretty(&report).unwrap_or_default());
        if correct {
            let guild_invitation = matches!(rank, "S" | "S++") && !self.knights_unlocked;
            self.cleared.insert(self.selected);
            self.ranks.insert(self.selected, rank.into());
            if matches!(rank, "S" | "S++") { self.unlock_knights(); }
            self.message = format!("ПОБЕДА · {rank} RANK\nПройдено тестов: 1/1\nОтвет: {}\nПопыток: {attempt}",
                options.get(correct_answer).map_or("—", String::as_str));
            if guild_invitation {
                self.message.push_str("\nГильдия PEP8 прислала письма. Открой Кодекс знаний!");
            }
            self.finished_reports.insert(self.selected, self.message.clone());
        } else {
            self.message = format!("НЕ ПРОЙДЕНО · попытка {attempt}\n\
                Ожидалось: {}\nВыбран вариант №{}\nПройдено тестов: 0/1",
                options.get(correct_answer).map_or("—", String::as_str), answer + 1);
            self.lose_hp(false);
        }
        self.save_meta();
    }

    fn new_run(&mut self) {
        self.run_seed = fresh_seed();
        self.stages = build_stages(self.run_seed);
        self.selected = 0;
        self.location = Location::Valley;
        self.river_discovered = false;
        self.for4_discovered = false;
        self.tower_discovered = false;
        self.field_discovered = false;
        self.mills_discovered = false;
        self.if_discovered = false;
        self.roofs_discovered = false;
        self.codex_open = false;
        self.cleared.clear();
        self.finished_reports.clear();
        self.quiz_attempts.clear();
        self.ranks.clear();
        self.new_scrolls_run.clear();
        self.rare_won_run = false;
        self.new_treasures_run.clear();
        self.completion = None;
        self.hp = 5;
        self.last_report = None;
        self.save_meta(); // Сброс забега должен пережить перезапуск игры.
        self.message = "Новый полный забег по Долине начат.".into();
        // Codex и история анлоков сохраняются между забегами.
    }

    fn lose_hp(&mut self, free_first_reader: bool) {
        if free_first_reader {
            let misses = self.quiz_attempts.entry(self.selected).or_insert(0);
            *misses += 1;
            if *misses == 1 {
                self.message.push_str("\nПервая ошибка: HP не расходуется.");
                return;
            }
        }
        self.hp = self.hp.saturating_sub(1);
        self.message.push_str(&format!("\n−1 HP · осталось {}/5", self.hp));
        if self.hp == 0 {
            self.new_run();
            self.message = "[GAME OVER] HP 0/5. Забег начат заново.".into();
        }
    }

    fn attempt_project(&mut self, files: &[String; 4]) {
        if self.finished_encounter(self.selected) {
            self.message = self.encounter_result(self.selected);
            return;
        }
        if self.stages[self.selected].id != "grot_boss" { return; }
        self.attempt_id += 1;
        let validator = self.stages[self.selected].validator.clone();
        let request = json!({"task":"grot_boss", "source":"", "validator":validator,
            "project_files": {"pyproject.toml":files[0].as_str(), "README.md":files[1].as_str(),
                "__init__.py":files[2].as_str(), "core.py":files[3].as_str()}});
        match evaluate(&request, self.attempt_id) {
            Ok(report) => {
                let evaluation = &report["evaluation"];
                let passed = evaluation["passed"].as_bool().unwrap_or(false);
                let rank = evaluation["rank"].as_str().unwrap_or("—");
                let notes = evaluation["diagnostics"].as_array()
                    .map(|values|values.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n"))
                    .unwrap_or_default();
                self.message = format_report(&report, passed, rank, &notes);
                if passed {
                    self.cleared.insert(self.selected);
                    self.ranks.insert(self.selected,rank.into());
                    if matches!(rank, "S" | "S++") { self.unlock_knights(); }
                    self.finished_reports.insert(self.selected, self.message.clone());
                } else {
                    self.lose_hp(false);
                }
                self.save_meta();
            }
            Err(error) => self.message = format!("Ошибка проверки: {error}"),
        }
    }

    fn console_command(&mut self, raw: &str) {
        let words: Vec<&str> = raw.split_whitespace().collect();
        self.console_output = match words.as_slice() {
            ["help"] => "dev on/off · map · tp <index|id> · items · give <id> [count] · win · rank <rank> · runs <n> · seed <n> · unlock <index>".into(),
            ["dev", "on"] => {
                if !self.dev_mode {
                    self.save_meta();
                    self.dev_mode = true;
                    if self.save_path().exists() { self.load_meta(); } else { self.save_meta(); }
                }
                "DEV MODE ВКЛЮЧЕН — тестовое сохранение отдельно".into()
            }
            ["dev", "off"] => {
                if self.dev_mode {
                    self.save_meta(); self.dev_mode = false; self.load_meta();
                    self.selected = 0; self.location = Location::Valley;
                    self.completion = None;
                }
                "DEV MODE ВЫКЛЮЧЕН".into()
            }
            ["map"] if self.dev_mode => self.stages.iter().enumerate()
                .map(|(i, s)| format!("{i}: {} ({})", s.label, s.id))
                .collect::<Vec<_>>().join(" | "),
            ["items"] => available_items(),
            ["tp", node] if self.dev_mode => {
                let found = node.parse::<usize>().ok().filter(|i| *i < self.stages.len())
                    .or_else(|| self.stages.iter().position(|s| s.id == *node));
                if let Some(i) = found { self.select(i); format!("Переход: {}", self.stages[i].label) }
                else { "Неизвестный нод".into() }
            }
            ["give", item] | ["give", item, _] if self.dev_mode => {
                let count = words.get(2).and_then(|n| n.parse::<u32>().ok()).unwrap_or(1).clamp(1,999);
                if item_exists(item) {
                    *self.inventory.entry((*item).into()).or_insert(0) += count;
                    self.save_meta(); format!("Добавлено: {} ×{count}", item_name(item))
                } else { format!("Неизвестный предмет: {item}") }
            }
            ["win"] if self.dev_mode => {
                self.cleared.insert(self.selected); self.ranks.insert(self.selected, "S++".into());
                self.unlock_knights();
                if let Some(item) = self.stages[self.selected].reward.clone() {
                    *self.inventory.entry(item).or_insert(0) += 1;
                }
                self.save_meta(); "Узел завершён тестовой победой".into()
            }
            ["rank", rank] if self.dev_mode => {
                if ["S++","S","A","B","C","D"].contains(rank) {
                    self.ranks.insert(self.selected, (*rank).into());
                    if matches!(*rank, "S" | "S++") { self.unlock_knights(); }
                    self.save_meta(); format!("Ранг: {rank}")
                } else { "Неизвестный ранг".into() }
            }
            ["runs", n] if self.dev_mode => match n.parse::<u32>() {
                Ok(v) => { self.runs = v; self.save_meta(); format!("Прохождений: {v}") },
                Err(_) => "Нужно целое число".into(),
            },
            ["seed", n] if self.dev_mode => match n.parse::<u64>() {
                Ok(v) => { self.run_seed = v; self.stages = build_stages(v); self.save_meta(); format!("Seed: {v}") },
                Err(_) => "Нужно целое число".into(),
            },
            ["unlock", n] if self.dev_mode => match n.parse::<usize>() {
                Ok(v) if v < self.stages.len() => { self.cleared.insert(v); self.save_meta(); format!("Открыт: {v}") },
                _ => "Неверный индекс".into(),
            },
            _ if !self.dev_mode => "Сначала включи режим командой dev on".into(),
            _ => "Неизвестная команда. help — список.".into(),
        };
    }


}

// Унифицированная диагностика: даже успешное испытание показывает stdout
// всех проверок, ожидаемые результаты, PEP 8 и причины оценки.
