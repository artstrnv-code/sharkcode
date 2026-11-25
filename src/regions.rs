//! Region identity, navigation and compatibility with legacy map indices.
use crate::{field, mills, ifmostroy, roofs, for4, river, tower, valley, Game, Stage, evaluate};
use serde_json::json;
use std::fs;

// Регион — это экран карты, не индекс нода. Секретные регионы нельзя
// отрисовывать до их открытия, даже как заблокированный пункт меню.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Location { Valley, Grot, River, For4, Tower, Field, Mills, Ifmostroy, Roofs }
impl Location {
    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Valley => "Долина переменн",
            Self::Grot => "Грот веры",
            Self::River => "Река святого леса",
            Self::For4 => "4 4",
            Self::Tower => "Бесконечная башня",
            Self::Field => "Великая пашня",
            Self::Mills => "Мельницы стариков",
            Self::Ifmostroy => "ИФМОСТРОЙ-14",
            Self::Roofs => "Разрушенные крыши Бесконечной башни",
        }
    }
    pub(super) fn entry(self, stages: &[Stage]) -> usize {
        match self { Self::Valley => 0, Self::Grot => 9, Self::River => 17, Self::For4 => for4::for4_first(stages), Self::Tower => tower::tower_first(stages), Self::Field => field::first(stages), Self::Mills => mills::first(stages), Self::Ifmostroy => ifmostroy::first(stages), Self::Roofs => roofs::first(stages) }
    }
    pub(super) fn key(self) -> &'static str {
        match self { Self::Valley => "variable_valley", Self::Grot => "grot_faith", Self::River => "holy_forest_river", Self::For4 => "for4", Self::Tower => "tower", Self::Field => "field", Self::Mills => "mills", Self::Ifmostroy => "ifmostroy", Self::Roofs => "roofs" }
    }
    pub(super) fn from_key(value: &str) -> Option<Self> {
        match value { "variable_valley" => Some(Self::Valley), "grot_faith" => Some(Self::Grot),
            "holy_forest_river" => Some(Self::River), "for4" => Some(Self::For4), "tower" => Some(Self::Tower), "field" => Some(Self::Field), "mills" => Some(Self::Mills), "ifmostroy" => Some(Self::Ifmostroy), "roofs" => Some(Self::Roofs), _ => None }
    }
}

// The current map layout and its existing node order are preserved.
pub(super) fn location_of(index: usize, stages: &[Stage]) -> Location {
    match index {
            index if stages[index].id.starts_with("project_") => Location::Valley,
            index if stages[index].id == "mills_to_roofs" => Location::Mills,
            index if stages[index].id.starts_with("roof_") => Location::Roofs,
            index if stages[index].id == "field_to_if" => Location::Field,
            index if stages[index].id.starts_with("if_") => Location::Ifmostroy,
            index if index >= mills::first(stages) => Location::Mills,
            index if stages[index].id == "field_to_mills" => Location::Field,
            index if index >= field::first(stages) => Location::Field,
            index if stages[index].id == "for4_to_field" => Location::For4,
            index if stages[index].id == "tower_to_field" => Location::Tower,
            index if index >= tower::tower_first(stages) => Location::Tower,
            index if index == tower::river_to_tower(stages) => Location::River,
            index if index >= for4::for4_first(stages) => Location::For4,
            index if index == for4::river_to_for4(stages) => Location::River,
            9..=15 | 19 => Location::Grot,
            17 => Location::River,
            index if index >= river::RIVER_FIRST => Location::River,
            _ => Location::Valley,
    }
}

impl Game {
    pub(super) fn can_enter(&self, index: usize) -> bool {
        if self.dev_mode { return index < self.stages.len(); }
        // Мимики — ответвления от босса. Выйти без сдачи решения можно
        // простым выбором другой доступной карты: HP не изменяется.
        if self.stages[index].id.ends_with("_mimic_loops")
            || self.stages[index].id.ends_with("_mimic_surrender") {
            return index > 0 && self.cleared.contains(&(index - 1));
        }
        if self.stages[index].id == "river_mimic_conditions" {
            return index > 0 && self.cleared.contains(&(index - 1))
                && self.codex.contains("Коллекции");
        }
        if self.stages[index].id.ends_with("_clear") && index >= 2
            && self.stages[index - 1].id.contains("_mimic_") {
            return self.cleared.contains(&(index - 2));
        }
        // Project enemies are optional Valley encounters, unlocked on a later run.
        if self.stages[index].id.starts_with("project_") {
            return self.ever_visited_mills;
        }
        if self.stages[index].id == "mills_to_roofs" {
            return self.cleared.iter().any(|&i| self.stages[i].id == "mills_clear");
        }
        if self.stages[index].id.starts_with("roof_") {
            if !self.roofs_discovered { return false; }
            let id = self.stages[index].id.as_str();
            if id == "roof_entry" || id == "roof_random" { return true; }
            return index > 0 && self.cleared.contains(&(index - 1));
        }
        if self.stages[index].id == "field_to_if" {
            return self.cleared.iter().any(|&i| self.stages[i].id == "field_clear");
        }
        if self.stages[index].id.starts_with("if_") {
            if !self.if_discovered { return false; }
            let id = self.stages[index].id.as_str();
            if id == "if_to_oop" { return false; }
            if id == "if_entry" || id == "if_functions" { return true; }
            if id == "if_mimic" { return self.cleared.iter().any(|&i| self.stages[i].id == "if_lambda"); }
            if id == "if_boss" { return self.cleared.iter().any(|&i| self.stages[i].id == "if_lambda"); }
            if id == "if_clear" { return self.cleared.iter().any(|&i| self.stages[i].id == "if_boss"); }
            if id == "if_lambda" { return self.cleared.iter().any(|&i| self.stages[i].id == "if_fulione"); }
            return index > 0 && self.cleared.contains(&(index-1));
        }
        if index >= mills::first(&self.stages) && index < ifmostroy::first(&self.stages) {
            if !self.mills_discovered { return false; }
            let id = self.stages[index].id.as_str();
            if id == "mills_to_prison" || id == "mills_to_matrix" { return false; }
            if id == "mills_chest" { return index >= 1 && self.cleared.contains(&(index-1)); }
            if id == "mills_clear" { return index >= 2 && self.cleared.contains(&(index-2)); }
            if index <= mills::first(&self.stages)+1 {return true;}
            // Пропускаем необязательный сундук при проходе к итогам.
            return self.cleared.contains(&(index-1));
        }
        if self.stages[index].id == "field_to_mills" {
            return self.cleared.iter().any(|&i| self.stages[i].id == "field_clear");
        }
        if index >= field::first(&self.stages) {
            if !self.field_discovered { return false; }
            if index <= field::first(&self.stages) + 1 { return true; }
            if self.stages[index].id == "field_new_chest" {
                return self.cleared.contains(&(index-1));
            }
            if self.stages[index].id == "field_clear" {
                return index >= 2 && self.cleared.contains(&(index-2));
            }
            return self.cleared.contains(&(index-1));
        }
        if self.stages[index].id == "for4_to_field" {
            return self.cleared.iter().any(|&i| self.stages[i].id == "for4_clear");
        }
        if self.stages[index].id == "tower_to_field" {
            return self.cleared.iter().any(|&i| self.stages[i].id == "tower_clear");
        }
        if index >= tower::tower_first(&self.stages) {
            if !self.tower_discovered { return false; }
            if index <= tower::tower_first(&self.stages) + 2 { return true; }
            return self.cleared.contains(&(index - 1));
        }
        if index == tower::river_to_tower(&self.stages) {
            return self.cleared.contains(&(for4::river_to_for4(&self.stages) - 1));
        }
        if index >= for4::for4_first(&self.stages) {
            if !self.for4_discovered { return false; }
            if index <= for4::for4_first(&self.stages) + 1 { return true; }
            return self.cleared.contains(&(index - 1));
        }
        if index == for4::river_to_for4(&self.stages) {
            return self.cleared.contains(&(for4::river_to_for4(&self.stages) - 1));
        }
        if index >= river::RIVER_FIRST {
            if !self.river_discovered { return false; }
            let first = river::RIVER_FIRST;
            if index == first { return true; }
            // Scrolls are optional to read; visiting the card completes them.
            return self.cleared.contains(&(index - 1));
        }
        valley::can_enter(index, &self.cleared, self.grot_unlocked, self.river_discovered)
    }

    pub(super) fn select(&mut self, index: usize) {
        if index >= self.stages.len() { return; }
        let target_region = location_of(index, &self.stages);
        // В обычном забеге кнопки карты не могут обойти однонаправленный entry.
        // Для тестирования Dev Console по-прежнему может телепортировать.
        if !self.dev_mode && target_region != self.location {
            self.message = "Назад пути нет. Переходи через входной узел или начни новый забег.".into();
            return;
        }
        if self.can_enter(index) {
            self.selected = index;
            self.location = target_region;
            if index == 17 {
                self.river_discovered = true;
                self.save_meta();
            }
            // River scroll cards are collected when reached, independent of reading.
            if self.stages[index].node_type == "scroll" && index >= river::RIVER_FIRST {
                self.cleared.insert(index);
                let name = match self.stages[index].id.as_str() {
                    "if_functions" => "ИФМОСТРОЙ-14: Функция".to_string(),
                    "if_lambda" => "ИФМОСТРОЙ-14: Лямбды".to_string(),
                    "roof_random" => "Разрушенные крыши: Библиотеки. random".to_string(),
                    _ => self.stages[index].label.clone(),
                };
                if self.codex.insert(name.clone()) { self.new_scrolls_run.insert(name.clone()); }
                // «Коллекции» — отметка знакомства с темой для мимика Реки.
                if self.stages[index].id == "field_lists1" { self.codex.insert("Коллекции".into()); }
                self.save_meta();
            }
            // Region-clear nodes are informational, but count as traversed
            // so the shared exit can become reachable without code execution.
            if index == 15 || index == 16 || self.stages[index].node_type == "region.clear" {
                self.cleared.insert(index);
                self.save_meta();
            }
            if index == 9 && self.codex.insert("Библиотеки (часть 1)".into()) {
                self.new_scrolls_run.insert("Библиотеки (часть 1)".into());
                self.save_meta();
            }
            if index == 13 && self.codex.insert("Библиотеки (часть 2)".into()) {
                self.new_scrolls_run.insert("Библиотеки (часть 2)".into());
                self.save_meta();
            }
            self.last_report = None;
            self.completion = match index {
                15 => Some(self.region_report("ГРОТ ВЕРЫ", 10..=14)),
                16 => Some(self.region_report("ДОЛИНА ПЕРЕМЕНН", 1..=6)),
                _ if self.stages[index].id == "river_clear" =>
                    Some(self.region_report("РЕКА СВЯТОГО ЛЕСА", river::RIVER_FIRST..=index.saturating_sub(1))),
                _ if self.stages[index].id == "for4_clear" =>
                    Some(self.region_report("4 4", for4::for4_first(&self.stages)..=index.saturating_sub(1))),
                _ if self.stages[index].id == "tower_clear" =>
                    Some(self.region_report("БЕСКОНЕЧНАЯ БАШНЯ", tower::tower_first(&self.stages)..=index.saturating_sub(1))),
                _ if self.stages[index].id == "roof_clear" =>
                    Some(self.region_report("РАЗРУШЕННЫЕ КРЫШИ", roofs::first(&self.stages)..=index.saturating_sub(1))),
                _ if self.stages[index].id == "if_clear" =>
                    Some(self.region_report("ИФМОСТРОЙ-14", ifmostroy::first(&self.stages)..=index.saturating_sub(1))),
                _ if self.stages[index].id == "mills_clear" =>
                    Some(self.region_report("МЕЛЬНИЦЫ СТАРИКОВ", mills::first(&self.stages)..=index.saturating_sub(1))),
                _ if self.stages[index].id == "field_clear" =>
                    Some(self.region_report("ВЕЛИКАЯ ПАШНЯ", field::first(&self.stages)..=index.saturating_sub(1))),
                _ => None,
            };
            // Новый узел всегда закрывает Кодекс и возвращает управление игре.
            self.codex_open = false;
            self.message = if self.finished_encounter(index) {
                self.encounter_result(index)
            } else if self.stages[index].node_type == "region.clear" {
                "Здесь были побеждены все враги. Результаты прохождения — в отчёте региона.".into()
            } else {
                String::new()
            };
        } else {
            self.message = "Путь ещё не открыт.".into();
        }
    }

    pub(super) fn entry_uses_code(&self) -> bool {
        if self.stages[self.selected].node_type != "entry" {return false;}
        let path = format!("content/nodes/{}.toml", self.stages[self.selected].id);
        fs::read_to_string(path).ok()
            .and_then(|raw|toml::from_str::<toml::Value>(&raw).ok())
            .and_then(|cfg|cfg.get("GAME").and_then(|v|v.get("interaction"))
                .and_then(toml::Value::as_str).map(|s|s=="code"))
            .unwrap_or(false)
    }

    // Универсальная точка перехода. В TOML узла задаются destination и interaction.
    // В обычной игре направление только вперёд: в прежний регион можно попасть
    // лишь новым забегом; разработчик может использовать tp.
    pub(super) fn use_entry(&mut self, source: &str) {
        let stage = &self.stages[self.selected];
        if stage.node_type != "entry" { return; }
        let path = format!("content/nodes/{}.toml", stage.id);
        let config = fs::read_to_string(&path).ok()
            .and_then(|raw| toml::from_str::<toml::Value>(&raw).ok());
        let game_cfg = config.as_ref().and_then(|c| c.get("GAME"));
        let destination = game_cfg.and_then(|v| v.get("destination"))
            .and_then(toml::Value::as_str).and_then(Location::from_key);
        let interaction = game_cfg.and_then(|v| v.get("interaction"))
            .and_then(toml::Value::as_str).unwrap_or("button");
        let expected = game_cfg.and_then(|v| v.get("wait_output"))
            .and_then(toml::Value::as_str).unwrap_or("go\n");
        let Some(destination) = destination else {
            self.message = format!("Некорректный destination в {path}");
            return;
        };
        if !self.can_enter(self.selected) {
            self.message = "Переход пока закрыт.".into(); return;
        }
        if interaction == "code" {
            // Исполнение кода проверяется тем же Python runner, что и обычные задания.
            self.attempt_id += 1;
            let request = json!({"source":source,"task":"none","tests":[
                {"inputs":[],"expected":expected.trim_end_matches('\n')}]});
            match evaluate(&request,self.attempt_id) {
                Ok(report) if report["evaluation"]["passed"].as_bool() == Some(true) => {},
                Ok(_) => { self.message = "Переход не активирован: ожидалось print(\"go\").".into(); return; },
                Err(err) => { self.message = format!("Ошибка проверки входа: {err}"); return; }
            }
        } else if interaction != "button" {
            self.message = format!("Неизвестный interaction: {interaction}"); return;
        }
        if destination == Location::River { self.river_discovered = true; }
        if destination == Location::For4 { self.for4_discovered = true; }
        if destination == Location::Tower { self.tower_discovered = true; }
        if destination == Location::Field { self.field_discovered = true; }
        if destination == Location::Ifmostroy { self.if_discovered = true; }
        if destination == Location::Roofs { self.roofs_discovered = true; }
        if destination == Location::Mills {
            self.mills_discovered = true;
            self.ever_visited_mills = true;
        }
        self.location = destination;
        self.selected = destination.entry(&self.stages);
        self.last_report = None;
        self.codex_open = false;
        self.completion = None;
        self.message = format!("Вход в регион: {}",destination.title());
        self.save_meta();
    }

    fn region_report(&self, region: &str, indices: std::ops::RangeInclusive<usize>) -> String {
        let ranks = ["S++", "S", "A", "B", "C", "D"];
        let required = indices.filter(|i| self.stages[*i].node_type.starts_with("enemy")
            // Автоматрон — необязательный мимик и не снижает итог региона.
            && !(region == "ИФМОСТРОЙ-14" && self.stages[*i].id == "if_mimic"));
        let results: Vec<_> = required.map(|i| self.ranks.get(&i)).collect();
        let worst = results.iter().filter_map(|r| r.and_then(|r| ranks.iter().position(|x| *x == r.as_str())))
            .max().unwrap_or(5);
        let rank = if results.iter().all(|r| r.is_some()) { ranks[worst] } else { "—" };
        let mut lines = vec![format!("> region --clear \"{region}\""),
            "[OK] REGION CLEARED".into(), format!("[RANK] {rank}"),
            format!("[HP] {}/5", self.hp)];
        if !self.new_scrolls_run.is_empty() { lines.push("+ NEW SCROLL DISCOVERED".into()); }
        if !self.new_treasures_run.is_empty() { lines.push("+ TREASURE COLLECTED".into()); }
        if self.rare_won_run { lines.push("+ RARE ENEMY DEFEATED".into()); }
        lines.push(if region == "ИФМОСТРОЙ-14" { "> next: НИИ ВЗ и ООП (в разработке)".into() } else if region == "МЕЛЬНИЦЫ СТАРИКОВ" { "> next: Тюрьма империала / Сломанная матрица (в разработке)".into() } else if region == "ВЕЛИКАЯ ПАШНЯ" { "> next: Мельницы стариков".into() } else { "> next: Река святого леса".into() });
        lines.join("\n")
    }

}
