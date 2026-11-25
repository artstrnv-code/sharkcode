fn format_report(report: &Value, passed: bool, rank: &str, notes: &str) -> String {
    let runs = report["runs"].as_array();
    let tests_total = runs.map_or(0, |r| r.len());
    let test_passed = report["evaluation"]["tests_passed"].as_u64().unwrap_or(0);
    let mut out = format!("{} · {} RANK\nПройдено тестов: {}/{}",
        if passed {"ПОБЕДА"} else {"НЕ ПРОЙДЕНО"}, rank, test_passed, tests_total);
    if let Some(runs) = runs {
        for (index, run) in runs.iter().enumerate() {
            let raw = run["stdout"].as_str().unwrap_or("");
            let actual = serde_json::to_string(raw).unwrap_or_default();
            let expected = run["expected_display"].as_str()
                .or_else(|| run["expected"].as_str());
            out.push_str(&format!("\n\nТЕСТ {}\nВыведено: {}", index+1, actual));
            if let Some(value) = expected {
                let label = if run["expectation_kind"].as_str() == Some("rule") {
                    "Условие проверки"
                } else { "Ожидалось" };
                out.push_str(&format!("\n{}: {}", label,
                    serde_json::to_string(value).unwrap_or_default()));
            }
            if let Some(returned) = run["returned"].as_str() {
                out.push_str(&format!("\n{}({}) вернуло: {}",
                    run["function"].as_str().unwrap_or("function"),
                    run["args"].get(0).map(Value::to_string).unwrap_or_default(), returned));
            }
            if let Some(error) = run["error"].as_str() {
                out.push_str(&format!("\nОшибка: {error}"));
            }
            if let Some(ok) = run["passed"].as_bool() {
                out.push_str(if ok {"\n✓ Пройден"} else {"\n✗ Не пройден"});
            }
        }
    }
    let ev = &report["evaluation"];
    if let Some(ok) = ev["validator_passed"].as_bool() {
        out.push_str(if ok {"\nСпециальная проверка: пройдена"}
            else {"\nСпециальная проверка: не пройдена"});
    }
    if ev["infrastructure_error"].as_bool() == Some(true) {
        out.push_str("\nСБОЙ ПРОВЕРЯЮЩЕГО: здоровье не расходуется");
    }
    for (key, label) in [("pep8_ok", "PEP8"),
                         ("no_overhead", "NO OVERHEAD"),
                         ("no_bad_practice", "NO BAD PRACTICE")] {
        if let Some(ok) = ev[key].as_bool() {
            out.push_str(&format!("\n{} {}", if ok {"+"} else {"−"}, label));
        }
    }
    if let Some(lines) = ev["meaningful_lines"].as_u64() {
        out.push_str(&format!("\nЗначимых строк: {lines}"));
    }
    if let Some(performance) = ev["performance"].as_object() {
        if let (Some(elapsed), Some(memory)) = (performance.get("execution_ns").and_then(Value::as_array),
            performance.get("peak_python_bytes").and_then(Value::as_array)) {
            if let (Some(ns), Some(bytes)) = (elapsed.first().and_then(Value::as_u64),
                memory.first().and_then(Value::as_u64)) {
                out.push_str(&format!("\nПрофиль первого теста: {:.3} мс, пик Python-памяти {} Б",
                    ns as f64 / 1_000_000.0, bytes));
            }
        }
    }
    if let Some(p) = ev["performance"].as_object() {
        let timing = (p.get("user_median_ns").and_then(Value::as_u64),
                      p.get("reference_median_ns").and_then(Value::as_u64));
        if let (Some(player),Some(reference)) = timing {
            out.push_str(&format!("\nСравнение: {:.3} мс / {:.3} мс (эталон)",
                player as f64 / 1_000_000.0, reference as f64 / 1_000_000.0));
        }
        if let (Some(player),Some(reference)) = (
            p.get("user_peak_bytes").and_then(Value::as_u64),
            p.get("reference_peak_bytes").and_then(Value::as_u64)) {
            out.push_str(&format!("\nПамять: {} Б / {} Б (эталон)", player, reference));
        }
    }
    if !notes.is_empty() { out.push_str(&format!("\n\nДиагностика:\n{}", notes)); }
    out
}

fn item_name(id:&str)->String {
    if let Ok(contents)=fs::read_to_string(format!("content/items/{id}.toml")) {
        if let Ok(parsed)=contents.parse::<toml::Table>() {
            if let Some(name)=parsed.get("name").and_then(toml::Value::as_str){return name.into();}
        }
    }
    id.into()
}
fn item_exists(id:&str)->bool{ fs::metadata(format!("content/items/{id}.toml")).is_ok() }
fn available_items()->String{
    fs::read_dir("content/items").ok().into_iter().flatten().flatten()
        .filter_map(|e|e.path().file_stem().map(|n|n.to_string_lossy().into_owned()))
        .collect::<Vec<_>>().join(", ")
}

fn fresh_seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as u64
}

// Generic world assembly. Regional generators own their encounter content.
fn build_stages(initial_seed: u64) -> Vec<Stage> {
    let mut result = valley::build_valley_stages(initial_seed);
    result.extend(river::build_river_stages(initial_seed));
    result.push(for4::build_river_to_for4());
    result.extend(for4::build_for4_stages(initial_seed));
    result.push(tower::build_river_to_tower());
    result.extend(tower::build_tower_stages(initial_seed));
    // Append new transitions after legacy stages to keep all existing save indices.
    result.push(field::from_for4());
    result.push(field::from_tower());
    result.extend(field::build(initial_seed));
    result.extend(mills::build(initial_seed));
    result.extend(projects::build(initial_seed));
    // Новые ноды в конце: индексы сохранений 0.2.50 остаются стабильными.
    result.push(ifmostroy::exit());
    result.extend(ifmostroy::build(initial_seed));
    // Добавляем новые ноды только в конец: старые индексы сохранений не меняются.
    result.push(roofs::exit());
    result.extend(roofs::build(initial_seed));
    result
}

fn data_dir() -> PathBuf { PathBuf::from("sharkcode_data") }

fn evaluate(request: &Value, id: u64) -> Result<Value, String> {
    let serialized = request.to_string();
    if serialized.len() > 1024 * 1024 {
        return Err("Sandbox Core: запрос превышает 1 МиБ".into());
    }
    let tmp = data_dir().join("session");
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let source = request["source"].as_str().unwrap_or_default();
    fs::write(tmp.join(format!("attempt_{id:06}.py")), source).map_err(|e| e.to_string())?;
    let python = sandbox::locate_python()?;
    let output = sandbox::execute(&python, "runtime/inspector.py", serialized.as_bytes())?;
    let report: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let _ = fs::write(tmp.join(format!("attempt_{id:06}.json")),
                      serde_json::to_vec_pretty(&report).unwrap_or_default());
    Ok(report)
}

