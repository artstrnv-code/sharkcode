fn buttons(mut game: ResMut<Game>,
    mut interactions: Query<(&Interaction, &Action, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut editor: ResMut<CodeBuffer>,
    mut project: ResMut<ProjectEditor>,
) {
    for (interaction, action, mut color) in &mut interactions {
        // Полноэкранный Кодекс является модальным: нельзя нажать скрытые кнопки игры.
        if game.codex_open && !matches!(action, Action::ToggleCodex | Action::CodexPage(_)) {
            continue;
        }
        match interaction {
            Interaction::Pressed => {
                color.0 = Color::srgb(0.32, 0.48, 0.46);
                match action {
                    Action::Select(index) => {
                        // Кнопки других экранов не должны быть активны через скрытый UI.
                        if game.stages.get(*index).is_some() {
                            game.select(*index);
                        }
                    },
                    Action::UseEntry => game.use_entry(&editor.source),
                    Action::ShowDebug => game.show_debug(),
                    Action::ToggleCodex => game.toggle_codex(),
                    Action::CodexPage(page) => game.open_codex_page(*page),
                    Action::NewRun => {
                        game.new_run();
                        project.reset();
                        editor.reset();
                        project.last_selected = game.selected;
                    },
                    Action::Run => {
                        if game.stages[game.selected].node_type == "enemy.project" {
                            let active = project.active;
                            project.files[active] = editor.source.clone();
                            game.attempt_project(&project.files);
                        } else { game.attempt(&editor.source); }
                    },
                    Action::QuizAnswer(answer) => game.answer_quiz(*answer),
                    Action::ProjectTab(index) => {
                        if game.stages[game.selected].node_type == "enemy.project" {
                            let active = project.active;
                            project.files[active] = editor.source.clone();
                            project.active = *index;
                            editor.source = project.files[*index].clone();
                            editor.cursor = editor.source.len();
                            editor.anchor = None;
                            editor.preferred_column = None;
                        }
                    },
                    Action::ToggleInventory => game.inventory_open = !game.inventory_open,
                    Action::CloseCompletion => game.completion = None,
                }
            }
            Interaction::Hovered => color.0 = Color::srgb(0.23, 0.37, 0.43),
            Interaction::None => color.0 = Color::srgb(0.17, 0.27, 0.33),
        }
    }
}

// Keep the visible editor and cursor synchronized when changing project tabs.
fn sync_project_editor(game: Res<Game>, mut buffer: ResMut<CodeBuffer>,
    mut project: ResMut<ProjectEditor>) {
    if project.last_selected != game.selected {
        // Переход в ДРУГОЙ нод всегда очищает поле, undo/redo и все файлы
        // предыдущего проектного испытания. Вкладки внутри одного нода
        // переключаются отдельно, без потери данных.
        project.reset();
        buffer.reset();
        if game.stages[game.selected].node_type == "enemy.project" {
            buffer.source = project.files[project.active].clone();
            buffer.cursor = buffer.source.len();
        } else if game.stages[game.selected].id == "tower_garaon" {
            // In the original boss challenge these two lines are prewritten.
            buffer.source = "x = True\nwhile x == True:\n    ".into();
            buffer.cursor = buffer.source.len();
        }
        project.last_selected = game.selected;
    }

}

fn update_quiz_options(game: Res<Game>,
    mut answers: Query<(&QuizButtonText, &mut Text)>,
    mut buttons: Query<(&QuizAnswerButton, &mut Node)>) {
    if !game.is_changed() {return;}
    let (options, _) = valley::quiz_info(&game.stages[game.selected].id);
    for (index, mut label) in &mut answers {
        label.0 = options.get(index.0).cloned().unwrap_or_default();
    }
    // Двухвариантные тесты не должны получать выдуманные ответы.
    for (index, mut node) in &mut buttons {
        node.display = if index.0 < options.len() { Display::Flex } else { Display::None };
    }
}

// Единый рендерер панелей: не создаёт конфликтующих &mut Node запросов.
fn update_overlays(
    game: Res<Game>, input: Res<DevInput>,
    mut panels: Query<(&mut Node, Option<&EditorPanel>, Option<&RunPanel>,
        Option<&ConsolePanel>, Option<&InventoryPanel>, Option<&CompletionPanel>, Option<&ProjectTabs>, Option<&EntryPanel>, Option<&DebugPanel>),
        Or<(With<EditorPanel>, With<RunPanel>, With<ConsolePanel>,
            With<InventoryPanel>, With<CompletionPanel>, With<ProjectTabs>, With<EntryPanel>, With<DebugPanel>)>>,
    mut labels: Query<(&mut Text, Option<&ConsoleLabel>, Option<&InventoryText>,
        Option<&CompletionText>),
        Or<(With<ConsoleLabel>, With<InventoryText>, With<CompletionText>)>>,
) {
    if !game.is_changed() && !input.is_changed(){return;}
    let code_node = !game.finished_encounter(game.selected)
        && (matches!(game.stages[game.selected].node_type.as_str(), "enemy.code"|"chest.code"|"enemy.project")
            || game.entry_uses_code());
    for (mut panel, editor, run, console, inventory, complete, tabs, entry, debug) in &mut panels {
        panel.display=if (editor.is_some()||run.is_some()) && code_node && !game.codex_open
            || entry.is_some() && game.stages[game.selected].node_type == "entry"
                && !game.entry_uses_code() && !game.codex_open
            || tabs.is_some() && game.stages[game.selected].node_type == "enemy.project"
                && !game.finished_encounter(game.selected) && !game.codex_open
            || debug.is_some() && code_node && game.inventory.get("debug_gun").copied().unwrap_or(0) > 0 && !game.codex_open
            || console.is_some() && game.console_open && !game.codex_open
            || inventory.is_some() && game.inventory_open && !game.codex_open
            || complete.is_some() && game.completion.is_some() && !game.codex_open
            {Display::Flex}else{Display::None};
    }
    for (mut label, console, inventory, complete) in &mut labels {
        if console.is_some(){label.0=format!("CONSOLE {} > {}\n{}",if game.dev_mode{"DEV"}else{"OFF"},input.line,game.console_output);}
        else if inventory.is_some(){label.0=if game.inventory.is_empty(){"Инвентарь пуст".into()}
            else{game.inventory.iter().map(|(id,n)|format!("{} ×{}",item_name(id),n)).collect::<Vec<_>>().join("\n")};}
        else if complete.is_some(){label.0=game.completion.clone().unwrap_or_default();}
    }
}

// Посимвольное раскрытие технического задания и отчёта.
// PageDown мгновенно завершает анимацию, не мешая вводу пробелов в редакторе.
fn typewriter(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>, mut reveal: ResMut<TextReveal>,
    // Both queries mutate Text, so Bevy needs proof they are disjoint (B0001).
    mut stage: Query<&mut Text, (With<StageText>, Without<MessageText>)>,
    mut message: Query<&mut Text, (With<MessageText>, Without<StageText>)>,
) {
    reveal.elapsed += time.delta_secs();
    let fast_forward=keys.just_pressed(KeyCode::PageDown) && !game.console_open;
    let amount=if fast_forward {usize::MAX}
        else if reveal.elapsed>=0.02 {let n=(reveal.elapsed/0.02) as usize;reveal.elapsed=0.0;n}
        else {0};
    if amount==0{return;}
    let n=reveal.stage.chars().count();
    reveal.stage_visible=reveal.stage_visible.saturating_add(amount).min(n);
    if let Ok(mut label)=stage.single_mut(){label.0=reveal.stage.chars().take(reveal.stage_visible).collect();}
    let n=reveal.message.chars().count();
    reveal.message_visible=reveal.message_visible.saturating_add(amount).min(n);
    if let Ok(mut label)=message.single_mut(){label.0=reveal.message.chars().take(reveal.message_visible).collect();}
}

fn update_labels(game: Res<Game>, mut stage: Query<&mut Text, (With<StageText>, Without<MessageText>, Without<StatusText>)>,
    mut msg: Query<&mut Text, (With<MessageText>, Without<StageText>, Without<StatusText>)>,
    mut status: Query<&mut Text, (With<StatusText>, Without<MessageText>, Without<StageText>)>,
    map_buttons: Query<(&MapButtonLabel, &Children), With<Button>>,
    mut quiz_panel: Query<&mut Node, (With<QuizPanel>, Without<Button>, Without<CodexPanel>)>,
    mut reveal: ResMut<TextReveal>,
    mut codex_panels: Query<&mut Node, (With<CodexPanel>, Without<Button>, Without<QuizPanel>)>,
    mut codex_texts: Query<&mut Text, (With<CodexText>, Without<StageText>, Without<MessageText>, Without<StatusText>)>,
    mut codex_entries: Query<(&CodexEntry, &mut Node), (With<Button>, Without<CodexPanel>, Without<QuizPanel>)>,
    mut texts: Query<&mut Text, (Without<StageText>, Without<MessageText>, Without<StatusText>, Without<CodexText>, Without<QuizButtonText>)>) {
    if !game.is_changed() { return; }
    if let Ok(mut panel) = quiz_panel.single_mut() {
        panel.display = if game.stages[game.selected].node_type == "enemy.quiz"
            && !game.finished_encounter(game.selected) {
            Display::Flex
        } else { Display::None };
    }
    if let Ok(mut panel) = codex_panels.single_mut() {
        panel.display = if game.codex_open { Display::Flex } else { Display::None };
    }
    if let Ok(mut text) = codex_texts.single_mut() {
        let next = game.codex_text();
        if text.0 != next { text.0 = next; }
    }
    for (entry, mut node) in &mut codex_entries {
        let obtained = match entry.0 {
            CodexPage::Base => game.codex.contains("База"),
            CodexPage::BestAssignment => game.codex.contains("Лучшее присваивание"),
            CodexPage::Libraries1 => game.codex.contains("Библиотеки (часть 1)"),
            CodexPage::Libraries2 => game.codex.contains("Библиотеки (часть 2)"),
            CodexPage::RiverBranching => game.codex.contains("Река святого леса: Ветвления"),
            CodexPage::RiverComparison => game.codex.contains("Сравнение"),
            CodexPage::RiverLogic => game.codex.contains("Операторы логики"),
            CodexPage::ForBase => game.codex.contains("4 4: Основы циклов for"),
            CodexPage::ForAdvanced => game.codex.contains("Продвинутый: Переменные"),
            CodexPage::TowerWhile => game.codex.contains("Бесконечная башня: Циклы while"),
            CodexPage::TowerLore => game.codex.contains("Лор: Первозмий и аспекты"),
            CodexPage::Walrus => game.codex.contains("Моржовое присваивание"),
            CodexPage::Notes0 => game.codex.contains("Заметки №0"),
            CodexPage::Notes1 => game.codex.contains("Заметки №1"),
            CodexPage::MatchCase => game.codex.contains("match/case"),
            CodexPage::FieldLists1 => game.codex.contains("Великая пашня: Списки (часть 1)"),
            CodexPage::FieldLists2 => game.codex.contains("Великая пашня: Списки (часть 2)"),
            CodexPage::FieldAdvanced => game.codex.contains("Великая пашня: Злоба"),
            CodexPage::FieldAdvanced2 => game.codex.contains("Великая пашня: Фишки массивов"),
            CodexPage::MillsTuples => game.codex.contains("Мельницы стариков: Кортежи"),
            CodexPage::MillsDicts => game.codex.contains("Мельницы стариков: Словари"),
            CodexPage::IfFunctions => game.codex.contains("ИФМОСТРОЙ-14: Функция"),
            CodexPage::IfLambda => game.codex.contains("ИФМОСТРОЙ-14: Лямбды"),
            CodexPage::IfSets => game.codex.contains("ИФМОСТРОЙ-14: Множества"),
            CodexPage::RoofRandom => game.codex.contains("Разрушенные крыши: Библиотеки. random"),
            CodexPage::KnightsIntro => game.knights_unlocked,
            CodexPage::KnightsNames => game.knights_unlocked,
            CodexPage::KnightsComments => game.knights_unlocked,
            CodexPage::KnightsSpaces => game.knights_unlocked,
            CodexPage::KnightsNewline => game.knights_unlocked,
            CodexPage::KnightsConditions => game.knights_unlocked,
        };
        node.display = if obtained { Display::Flex } else { Display::None };
    }
    let s = &game.stages[game.selected];
    let stage_full = if matches!(s.node_type.as_str(), "entry" | "empty" | "region.clear" | "scroll") {
        format!("{}\n\n{}", s.label, s.lore)
    } else {
        if game.finished_encounter(game.selected) {
            format!("{}\n\nИспытание завершено. Результат сохранён; повторное решение недоступно до нового забега.", s.label)
        } else {
            format!("{}\n\n{}\n\nТЗ: {}", s.label, s.lore, s.task)
        }
    };
    if reveal.stage != stage_full {
        reveal.stage = stage_full;
        reveal.stage_visible = 0;
        if let Ok(mut text)=stage.single_mut(){text.0.clear();}
    }
    if reveal.message != game.message {
        reveal.message = game.message.clone();
        reveal.message_visible = 0;
        if let Ok(mut text)=msg.single_mut(){text.0.clear();}
    }
    for (node, children) in &map_buttons {
        for child in children.iter() {
            if let Ok(mut label) = texts.get_mut(child) {
                let status = if game.cleared.contains(&node.0) { "[OK]" }
                    else if !game.can_enter(node.0) { "[-]" } else { "[+]" };
                let sigil = match game.stages[node.0].node_type.as_str() {
                    "scroll" => "[T]", "region.clear" => "[R]", "chest.code" => "[$]",
                    "enemy.quiz" => "[?]", "entry" => "[>]", "empty" => "[~]", _ => "[!]",
                };
                label.0 = format!("{status} {sigil} {}", game.stages[node.0].label);
            }
        }
    }
    if let Ok(mut text) = status.single_mut() {
        text.0 = format!("HP: {}/5 · Побеждено врагов: {}/6\nПолные забеги: {} · Скрижалей: {}\nF12: консоль · DEV: {}", game.hp,
            game.cleared.iter().filter(|i| (1usize..=6).contains(*i)).count(),game.runs,
            game.codex.len(),if game.dev_mode {"ON"} else {"OFF"});
    }
}
