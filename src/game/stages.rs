#[derive(Clone, Copy, Debug)]
enum Encounter { Tutorial, Shyguy, Counter, Wise, Egg, Statue, CardPlayer, Echo, Chest, Grot, GrotQuest }

#[derive(Clone)]
struct Stage { label: String, lore: String, task: String, kind: Encounter, expected: String,
    id: String, node_type: String, validator: String, pep8_coef: f64, strokes_coef: f64,
    reward: Option<String>, input: String, computation_required: bool, text_tolerance: bool }

impl Stage {
    fn with_content(mut self) -> Self {
        // Данные задания подгружаются из TOML. Процедурные условия генерируются движком.
        let name = match self.kind {
            Encounter::Tutorial => "tutorial", Encounter::Shyguy => "stesnyaga",
            Encounter::Counter => "counter", Encounter::Wise => "wise",
            Encounter::Egg => "egg", Encounter::Statue => "statue",
            Encounter::CardPlayer => "card_player", Encounter::Echo => "echo",
            Encounter::Chest => "chest", Encounter::Grot => "grot",
            Encounter::GrotQuest => "grot",
        };
        let name = if self.id.is_empty() { name.to_string() } else { self.id.clone() };
        self.id = name.clone();
        self.node_type = if matches!(self.kind, Encounter::Tutorial | Encounter::Grot) {"scroll"}
            else if matches!(self.kind, Encounter::CardPlayer) {"enemy.quiz"}
            else if matches!(self.kind, Encounter::Chest) {"chest.code"}
            else {"enemy.code"}.into();
        let path = format!("content/nodes/{name}.toml");
        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str::<NodeConfig>(&contents) {
                Ok(config) => {
                    self.id = config.meta.id;
                    self.label = config.meta.name;
                    self.lore = config.meta.lore;
                    self.node_type = config.meta.node_type;
                    // Для случайных условий используем сгенерированное ТЗ.
                    if !matches!(self.kind, Encounter::Counter | Encounter::Wise) {
                        self.task = config.meta.tt;
                    }
                    if let Some(game) = config.game {
                        self.input = game.input.unwrap_or_default();
                        if let Some(expected) = game.wait_output {
                            if !matches!(self.kind, Encounter::Counter | Encounter::Wise) {
                                self.expected = expected;
                            }
                        }
                    }
                    if let Some(rank) = config.ranking {
                        self.validator = rank.validator.unwrap_or_default();
                        self.computation_required = rank.computation_required.unwrap_or(false);
                        self.text_tolerance = rank.text_tolerance.unwrap_or(false);
                        if let Some(params) = rank.param {
                            if let Some(p) = params.pep8 { self.pep8_coef = p.coef; }
                            if let Some(p) = params.strokes { self.strokes_coef = p.coef; }
                        }
                    }
                    self.reward = config.reward.and_then(|r| r.item);
                }
                Err(err) => eprintln!("SHARKCODE: ошибка TOML {path}: {err}"),
            },
            Err(err) => eprintln!("SHARKCODE: отсутствует {path}: {err}"),
        }
        self
    }
}

#[derive(Deserialize)]
struct NodeConfig { #[serde(rename = "META")] meta: NodeMeta,
    #[serde(rename = "GAME")] game: Option<NodeGame>,
    #[serde(rename = "RANKING")] ranking: Option<NodeRanking>,
    #[serde(rename = "REWARD")] reward: Option<NodeReward> }
#[derive(Deserialize)]
struct NodeMeta { id: String, name: String, lore: String, tt: String,
    #[serde(rename = "type")] node_type: String }
#[derive(Deserialize)]
struct NodeGame { input: Option<String>, wait_output: Option<String> }
#[derive(Deserialize)]
struct NodeReward { item: Option<String> }
#[derive(Deserialize)]
struct NodeRanking { validator: Option<String>, param: Option<RankParams>,
    computation_required: Option<bool>, text_tolerance: Option<bool> }
#[derive(Deserialize)]
struct RankParams { pep8: Option<RankCoef>, strokes: Option<RankCoef> }
#[derive(Deserialize)]
struct RankCoef { coef: f64 }

impl Stage {
    fn task_id(&self) -> &'static str {
        match self.kind {
            Encounter::Shyguy => "shyguy", Encounter::Counter => "counter",
            Encounter::Wise => "wise", Encounter::Egg => "egg",
            Encounter::Statue => "statue", Encounter::Echo => "echo",
            Encounter::Chest => "chest", Encounter::GrotQuest => "none", _ => "none",
        }
    }
    fn tests(&self) -> Vec<Value> {
        if (self.id.starts_with("river_") || self.id.starts_with("for4_") || self.id.starts_with("tower_") || self.id.starts_with("field_") || self.id.starts_with("mills_") || self.id.starts_with("project_") || self.id.starts_with("if_") || self.id.starts_with("roof_")) && !self.id.ends_with("_entry") {
            if let Ok(cases) = serde_json::from_str::<Vec<Value>>(&self.input) {
                return cases;
            }
        }
        let input = if self.input.is_empty() { Vec::<String>::new() }
                    else { vec![self.input.clone()] };
        if matches!(self.kind, Encounter::Chest) { vec![json!({"inputs": input})] }
        else if self.id == "grot_reader" {
            vec![json!({"inputs":["True"],"expected":"True"}),
                 json!({"inputs":["False"],"expected":"False"})]
        } else { vec![json!({"inputs":input,"expected":self.expected.trim_end_matches('\n')})] }
    }
}

