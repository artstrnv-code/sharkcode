// Сохранение/загрузка. Формат JSON и миграции прежние (save_schema=3).
impl Game {
    fn save_path(&self) -> PathBuf {
        data_dir().join(if self.dev_mode { "dev_save.json" } else { "save.json" })
    }

    fn save_meta(&self) {
        let save = json!({"save_schema":3,"valley_runs":self.runs,"grot":self.grot_unlocked,
            "codex":self.codex,"knights_unlocked":self.knights_unlocked,
            "ever_visited_mills":self.ever_visited_mills,"cleared":self.cleared,"attempt_id":self.attempt_id,
            "run_seed": self.run_seed,"quiz_attempts":self.quiz_attempts,"ranks":self.ranks,
            "finished_reports":self.finished_reports,
            "inventory":self.inventory,"new_scrolls_run":self.new_scrolls_run,"rare_won_run":self.rare_won_run,
            "new_treasures_run":self.new_treasures_run,"hp":self.hp,
            "river_discovered": self.river_discovered,"for4_discovered": self.for4_discovered,"tower_discovered": self.tower_discovered,"field_discovered": self.field_discovered,"mills_discovered":self.mills_discovered,"if_discovered":self.if_discovered,"roofs_discovered":self.roofs_discovered,"current_location":self.location.key()});
        let path = self.save_path();
        let _ = fs::create_dir_all(data_dir());
        if let Ok(contents) = serde_json::to_vec_pretty(&save) { let _ = fs::write(path, contents); }
    }

    fn load_meta(&mut self) {
        if let Ok(bytes) = fs::read(self.save_path()) {
            if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                self.runs = value["valley_runs"].as_u64().unwrap_or(0) as u32;
                self.hp = value["hp"].as_u64().unwrap_or(5).min(5) as u32;
                self.inventory = serde_json::from_value(value["inventory"].clone()).unwrap_or_default();
                self.new_scrolls_run = serde_json::from_value(value["new_scrolls_run"].clone()).unwrap_or_default();
                self.rare_won_run = value["rare_won_run"].as_bool().unwrap_or(false);
                self.new_treasures_run = serde_json::from_value(value["new_treasures_run"].clone()).unwrap_or_default();
                self.attempt_id = value["attempt_id"].as_u64().unwrap_or(0);
                if let Some(seed) = value["run_seed"].as_u64() {
                    self.run_seed = seed;
                    self.stages = build_stages(seed);
                }
                self.grot_unlocked = value["grot"].as_bool().unwrap_or(false);
                self.river_discovered = value["river_discovered"].as_bool().unwrap_or(false);
                self.for4_discovered = value["for4_discovered"].as_bool().unwrap_or(false);
                self.tower_discovered = value["tower_discovered"].as_bool().unwrap_or(false);
                self.field_discovered = value["field_discovered"].as_bool().unwrap_or(false);
                self.mills_discovered = value["mills_discovered"].as_bool().unwrap_or(false);
                self.if_discovered = value["if_discovered"].as_bool().unwrap_or(false);
                self.roofs_discovered = value["roofs_discovered"].as_bool().unwrap_or(false);
                self.ever_visited_mills = value["ever_visited_mills"].as_bool()
                    .unwrap_or(self.mills_discovered);
                self.knights_unlocked = value["knights_unlocked"].as_bool().unwrap_or(false);
                self.location = value["current_location"].as_str()
                    .and_then(Location::from_key).unwrap_or(Location::Valley);
                self.selected = self.location.entry(&self.stages);
                if let Ok(items) = serde_json::from_value(value["codex"].clone()) {
                    self.codex = items;
                }
                // 0.2.49 saves had no permanent visit flag. Previously collected
                // Mills tablets survive new runs and prove the region was visited.
                if self.codex.contains("Мельницы стариков: Кортежи")
                    || self.codex.contains("Мельницы стариков: Словари") {
                    self.ever_visited_mills = true;
                }
                if let Ok(items) = serde_json::from_value(value["cleared"].clone()) {
                    self.cleared = items;
                }
                if let Ok(items) = serde_json::from_value(value["quiz_attempts"].clone()) {
                    self.quiz_attempts = items;
                }
                if let Ok(items) = serde_json::from_value(value["ranks"].clone()) {
                    self.ranks = items;
                }
                self.finished_reports = serde_json::from_value(value["finished_reports"].clone())
                    .unwrap_or_default();
                // Old saves already containing an S/S++ rank unlock the guild too.
                if self.ranks.values().any(|rank| matches!(rank.as_str(), "S" | "S++")) {
                    self.unlock_knights();
                }
                // 0.2.47 вставляет необязательный сундук перед field_clear.
                // Переносим пройденный узел и ранг из схемы 2, иначе старое
                // сохранение ошибочно считает новый сундук уже открытым.
                if value["save_schema"].as_u64().unwrap_or(0) == 2 {
                    if let Some(new_clear) = self.stages.iter().position(|s|s.id=="field_clear") {
                        let old_clear = new_clear - 1;
                        let shift = |index:usize| if index >= old_clear {index+1}else{index};
                        self.cleared = self.cleared.iter().map(|&i|shift(i)).collect();
                        self.ranks = self.ranks.drain().map(|(i,v)|(shift(i),v)).collect();
                        self.quiz_attempts = self.quiz_attempts.drain().map(|(i,v)|(shift(i),v)).collect();
                        self.finished_reports = self.finished_reports.drain()
                            .map(|(i,v)|(shift(i),v)).collect();
                    }
                }
                // В 0.2.39 появились три необязательных нода внутри
                // последовательных массивов. Переносим индексы старых
                // сохранений, а не присваиваем чужим врагам пройденные ранги.
                if value["save_schema"].as_u64().unwrap_or(0) < 2 {
                    let river_new = self.stages.iter().position(|s|s.id=="river_mimic_conditions");
                    let for_new = self.stages.iter().position(|s|s.id=="for4_mimic_loops");
                    let tower_new = self.stages.iter().position(|s|s.id=="tower_mimic_surrender");
                    if let (Some(r),Some(f),Some(t)) = (river_new,for_new,tower_new) {
                        let old_positions = [r, f-1, t-2];
                        let translate = |index: usize| -> usize {
                            index + old_positions.iter().filter(|&&pos|index >= pos).count()
                        };
                        self.cleared = self.cleared.iter().map(|&i|translate(i)).collect();
                        self.ranks = self.ranks.drain().map(|(i,v)|(translate(i),v)).collect();
                        self.quiz_attempts = self.quiz_attempts.drain().map(|(i,v)|(translate(i),v)).collect();
                        self.finished_reports = self.finished_reports.drain()
                            .map(|(i,v)|(translate(i),v)).collect();
                    }
                }
            }
        }
    }
}
