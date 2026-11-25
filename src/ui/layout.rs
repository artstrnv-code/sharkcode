// Берём системные шрифты Windows: они содержат кириллицу.
// Не копируем шрифты в дистрибутив и не полагаемся на ASCII-only шрифт Bevy.
fn load_windows_font(fonts: &mut Assets<Font>, candidates: &[&str]) -> Option<Handle<Font>> {
    let windir = std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    for name in candidates {
        let path = windir.join("Fonts").join(name);
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(font) = Font::try_from_bytes(bytes) {
                return Some(fonts.add(font));
            }
        }
    }
    None
}

fn text_font(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont { font: font.clone(), font_size: size, ..default() }
}

fn compact_codex_button(parent: &mut ChildSpawnerCommands, ui_font: &Handle<Font>) {
    parent.spawn((Button, Action::ToggleCodex,
        Node { width: px(164), min_height: px(34), flex_shrink: 0.0,
            padding: px(6).all(), ..default() },
        BackgroundColor(Color::srgb(0.17, 0.27, 0.33))))
        .with_children(|p| {
            p.spawn((Text::new("Кодекс знаний"), text_font(ui_font, 14.0),
                TextColor(Color::srgb(0.94, 0.93, 0.86))));
        });
}

fn codex_entry(parent: &mut ChildSpawnerCommands, page: CodexPage,
    label: &str, ui_font: &Handle<Font>) {
    parent.spawn((Button, Action::CodexPage(page), CodexEntry(page),
        Node { width: px(210), min_height: px(34), padding: px(6).all(),
            margin: px(3).all(), ..default() },
        BackgroundColor(Color::srgb(0.17, 0.27, 0.33))))
        .with_children(|p| {
            p.spawn((Text::new(label), text_font(ui_font, 14.0),
                TextColor(Color::srgb(0.94, 0.93, 0.86))));
        });
}

fn styled_button(parent: &mut ChildSpawnerCommands, name: String, action: Action,
    ui_font: &Handle<Font>) {
    parent.spawn((Button, action,
        Node { width: percent(100), min_height: px(39), padding: px(8).all(),
            ..default() },
        BackgroundColor(Color::srgb(0.17, 0.27, 0.33)),
    )).with_children(|p| { p.spawn((Text::new(name), text_font(ui_font, 16.0),
        TextColor(Color::srgb(0.94, 0.93, 0.86)))); });
}

fn setup(mut commands: Commands, game: Res<Game>, mut fonts: ResMut<Assets<Font>>) {
    // Приоритет — системные шрифты Windows с кириллицей.
    // При отсутствии нужных шрифтов используем резервный шрифт Bevy,
    // но предупреждаем в консоли, что кириллица может не отображаться.
    let ui_font = load_windows_font(&mut fonts, &["segoeui.ttf", "arial.ttf", "tahoma.ttf"])
        .unwrap_or_else(|| {
            eprintln!("SHARKCODE: кириллический UI-шрифт Windows не найден");
            TextFont::default().font
        });
    let code_font = load_windows_font(&mut fonts, &["consola.ttf", "cour.ttf", "lucon.ttf"])
        .unwrap_or_else(|| {
            eprintln!("SHARKCODE: моноширинный шрифт Windows не найден; использован UI-шрифт");
            ui_font.clone()
        });
    commands.spawn(Camera2d);
    commands.spawn((Node { width: percent(100), height: percent(100),
        flex_direction: FlexDirection::Row, padding: px(16).all(), column_gap: px(20),
        overflow: Overflow::clip(), ..default() },
        BackgroundColor(Color::srgb(0.06, 0.10, 0.14))))
        .with_children(|root| {
            root.spawn((Node { width: px(405), height: percent(100),
                min_width: px(355), flex_shrink: 0.0,
                flex_direction: FlexDirection::Column, row_gap: px(8),
                overflow: Overflow::clip(), ..default() },))
            .with_children(|left| {
                left.spawn((Text::new("SHARKCODE"), text_font(&ui_font, 26.0),
                    TextColor(Color::WHITE),Node {flex_shrink:0.0,..default()}));
                left.spawn((LocationTitle, Text::new("Долина переменн"),
                    text_font(&ui_font, 19.0),TextColor(Color::srgb(0.64,0.83,0.80)),
                    Node {flex_shrink:0.0,..default()}));
                left.spawn((MapScroll,ScrollPosition::default(),RelativeCursorPosition::default(),Node {
                    width:percent(100),flex_grow:1.0,min_height:px(0),
                    overflow:Overflow::scroll_y(),..default()},
                    BackgroundColor(Color::srgb(0.085,0.14,0.18))))
                .with_children(|viewport| {
                    for region in [Location::Valley,Location::Grot,Location::River,Location::For4,Location::Tower,Location::Field,Location::Mills,Location::Ifmostroy,Location::Roofs] {
                        region_map::spawn_location_map(viewport,region,&game,&ui_font);
                    }
                });
                left.spawn(Node {width:percent(100), flex_shrink:0.0,
                    flex_direction:FlexDirection::Column,row_gap:px(6),..default()})
                .with_children(|actions| {
                    styled_button(actions,"Новый забег".into(),Action::NewRun,&ui_font);
                    styled_button(actions,"Инвентарь".into(),Action::ToggleInventory,&ui_font);
                });
                left.spawn((Text::new(""),StatusText,text_font(&ui_font,14.0),
                    Node {flex_shrink:0.0,..default()},
                    TextColor(Color::srgb(0.69,0.80,0.78))));
            });
            root.spawn((Node { flex_grow: 1.0, min_width: px(0), height: percent(100),
                flex_direction: FlexDirection::Column, row_gap: px(10),
                overflow: Overflow::clip(), ..default() },))
            .with_children(|right| {
                right.spawn((ConsolePanel, Node { display: Display::None, width: percent(100),
                    min_height: px(65), padding: px(9).all(), ..default() }, BackgroundColor(Color::srgb(0.11,0.21,0.19))))
                .with_children(|panel| {panel.spawn((ConsoleLabel,Text::new(""),text_font(&code_font,14.0),TextColor(Color::WHITE)));});
                right.spawn((CompletionPanel, Node { display: Display::None, width: percent(100),
                    padding: px(15).all(), flex_direction:FlexDirection::Column, ..default() },
                    BackgroundColor(Color::srgb(0.055,0.08,0.10))))
                .with_children(|panel| {
                    panel.spawn((CompletionText,Text::new(""),text_font(&code_font,18.0),TextColor(Color::srgb(0.68,0.92,0.76))));
                    styled_button(panel,"Продолжить".into(),Action::CloseCompletion,&ui_font);
                });
                // Задание в собственном ограниченном по высоте скролле: длинный текст не
                // вытесняет редактор, кнопки и диагностику за нижний край окна.
                right.spawn((StagePanel, ScrollPosition::default(), Node {
                    width: percent(100), height: px(180), flex_shrink: 0.0,
                    overflow: Overflow::scroll_y(), ..default() },))
                    .with_children(|text| {
                        text.spawn((Text::new(""), StageText, text_font(&ui_font, 19.0),
                            TextColor(Color::srgb(0.95, 0.92, 0.85))));
                    });
                // Ответы показываются только на ноде врага-теста.
                right.spawn((QuizPanel, Node { display: Display::None,
                    width: percent(100), flex_direction: FlexDirection::Column,
                    row_gap: px(3), ..default() }))
                .with_children(|quiz| {
                    for index in 0..4 {
                        quiz.spawn((Button, Action::QuizAnswer(index), QuizAnswerButton(index),
                            Node { width: percent(100), min_height: px(30),
                                padding: px(6).all(), margin: px(2).all(), ..default() },
                            BackgroundColor(Color::srgb(0.17, 0.27, 0.33))))
                        .with_children(|button| {
                            button.spawn((QuizButtonText(index), Text::new(""),
                                text_font(&ui_font, 14.0), TextColor(Color::WHITE)));
                        });
                    }
                });
                right.spawn((ProjectTabs, Node { display: Display::None,
                    flex_direction: FlexDirection::Row, width: percent(100), column_gap: px(5),
                    ..default() },))
                .with_children(|tabs| {
                    for (index, name) in ["pyproject.toml", "README.md", "__init__.py", "core.py"].iter().enumerate() {
                        tabs.spawn((Button, Action::ProjectTab(index), Node {
                            padding: px(7).all(), ..default()
                        }, BackgroundColor(Color::srgb(0.17, 0.27, 0.33))))
                        .with_children(|button| { button.spawn((Text::new(*name), text_font(&code_font, 13.0),
                            TextColor(Color::WHITE))); });
                    }
                });
                right.spawn((EditorPanel, Button, RelativeCursorPosition::default(), ScrollPosition::default(), Node { width: percent(100), height: px(205), flex_shrink: 0.0,
                    border: px(1).all(), padding: px(10).all(),
                    overflow: Overflow::scroll_y(), ..default() },
                    BorderColor::all(Color::srgb(0.3, 0.4, 0.5)),
                    BackgroundColor(Color::srgb(0.10, 0.16, 0.20))))
                .with_children(|edit| {
                    // Текст и каретка являются разными UI-сущностями.
                    edit.spawn((CodeEditor, Text::new("│"),
                        text_font(&code_font, 18.0),
                        // The text keeps its full height so scroll never clips
                        // lines merely because an edit exceeds the viewport.
                        Node { width: percent(100), flex_shrink: 0.0, ..default() },
                        TextColor(Color::WHITE)));
                });
                // Кодекс доступен и на скрижалях, и на врагах-тестах.
                // Только кнопка запуска кода зависит от типа текущего нода.
                right.spawn((Node { width: percent(100), flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center, column_gap: px(12), ..default() },))
                    .with_children(|buttons| {
                        buttons.spawn((RunPanel, Node { flex_grow: 1.0, min_width: px(0), ..default() },))
                            .with_children(|run| {
                                styled_button(run, "Исполнить код".into(), Action::Run, &ui_font);
                            });
                        buttons.spawn((EntryPanel, Node {display:Display::None,flex_grow:1.0,..default()}))
                            .with_children(|entry| {
                                styled_button(entry, "Войти".into(), Action::UseEntry, &ui_font);
                            });
                        buttons.spawn((DebugPanel, Node { display: Display::None, width: px(128), ..default() }))
                            .with_children(|debug| styled_button(debug, "Run Debug".into(), Action::ShowDebug, &ui_font));
                        compact_codex_button(buttons, &ui_font);
                    });
                right.spawn((InventoryPanel, Node { display: Display::None, width: percent(100),
                    padding: px(10).all(), ..default() }, BackgroundColor(Color::srgb(0.12,0.19,0.23))))
                .with_children(|inv| {inv.spawn((InventoryText,Text::new(""),text_font(&ui_font,15.0),TextColor(Color::WHITE)));});
                // Отчёт прокручивается отдельно и никогда не меняет высоту редактора.
                right.spawn((ReportScroll, ScrollPosition::default(), RelativeCursorPosition::default(),
                    Node { width: percent(100), height: px(160), min_height: px(90),
                        flex_grow: 1.0, flex_shrink: 1.0,
                        overflow: Overflow::scroll_y(), ..default() }))
                    .with_children(|report| {
                        report.spawn((Text::new(""), MessageText, text_font(&ui_font, 14.0),
                            Node { width: percent(100), flex_shrink: 0.0, ..default() },
                            TextColor(Color::srgb(0.81, 0.86, 0.88))));
                    });
            });
        });
    // Модальный полноэкранный Кодекс — отдельный корневой UI-слой.
    // Он отрисовывается поверх всей игры и не влияет на flex-компоновку колонок.
    commands.spawn((CodexPanel, bevy::ui::GlobalZIndex(100), Node {
        display: Display::None, position_type: PositionType::Absolute,
        top: px(0), left: px(0), width: percent(100), height: percent(100),
        flex_direction: FlexDirection::Column, padding: px(24).all(), row_gap: px(16),
        overflow: Overflow::clip(), ..default()
    }, BackgroundColor(Color::srgb(0.055, 0.09, 0.13))))
    .with_children(|codex| {
        codex.spawn((Node { width: percent(100), flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center,
            flex_shrink: 0.0, ..default() },))
        .with_children(|header| {
            header.spawn((Text::new("КОДЕКС ЗНАНИЙ"), text_font(&ui_font, 28.0),
                TextColor(Color::WHITE)));
            header.spawn((Button, Action::ToggleCodex, Node {
                width: px(175), min_height: px(42), padding: px(9).all(),
                ..default() }, BackgroundColor(Color::srgb(0.17, 0.27, 0.33))))
            .with_children(|btn| {
                btn.spawn((Text::new("Закрыть  [Esc]"), text_font(&ui_font, 16.0),
                    TextColor(Color::WHITE)));
            });
        });
        codex.spawn((Node { width: percent(100), flex_grow: 1.0, min_height: px(0),
            flex_direction: FlexDirection::Row, column_gap: px(20), ..default() },))
        .with_children(|body| {
            body.spawn((CodexListScroll, ScrollPosition::default(), RelativeCursorPosition::default(), Node { width: px(250), flex_shrink: 0.0,
                min_height: px(0), overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column, row_gap: px(8), ..default() },))
            .with_children(|entries| {
                codex_entry(entries, CodexPage::Base, "База", &ui_font);
                codex_entry(entries, CodexPage::BestAssignment, "Лучшее присваивание", &ui_font);
                codex_entry(entries, CodexPage::Libraries1, "Библиотеки (часть 1)", &ui_font);
                codex_entry(entries, CodexPage::Libraries2, "Библиотеки (часть 2)", &ui_font);
                codex_entry(entries, CodexPage::RiverBranching, "Река святого леса: Ветвления", &ui_font);
                codex_entry(entries, CodexPage::RiverComparison, "Сравнение", &ui_font);
                codex_entry(entries, CodexPage::RiverLogic, "Операторы логики", &ui_font);
                codex_entry(entries, CodexPage::ForBase, "4 4: Основы циклов for", &ui_font);
                codex_entry(entries, CodexPage::ForAdvanced, "Продвинутый: Переменные", &ui_font);
                codex_entry(entries, CodexPage::TowerWhile, "Бесконечная башня: Циклы while", &ui_font);
                codex_entry(entries, CodexPage::TowerLore, "Лор: Первозмий и аспекты", &ui_font);
                codex_entry(entries, CodexPage::Walrus, "Моржовое присваивание", &ui_font);
                codex_entry(entries, CodexPage::Notes0, "Заметки №0", &ui_font);
                codex_entry(entries, CodexPage::Notes1, "Заметки №1", &ui_font);
                codex_entry(entries, CodexPage::MatchCase, "match/case", &ui_font);
                codex_entry(entries, CodexPage::FieldLists1, "Великая пашня: Списки (часть 1)", &ui_font);
                codex_entry(entries, CodexPage::FieldLists2, "Великая пашня: Списки (часть 2)", &ui_font);
                codex_entry(entries, CodexPage::FieldAdvanced, "Великая пашня: Злоба", &ui_font);
                codex_entry(entries, CodexPage::FieldAdvanced2, "Великая пашня: Фишки массивов", &ui_font);
                codex_entry(entries, CodexPage::MillsTuples, "Мельницы стариков: Кортежи", &ui_font);
                codex_entry(entries, CodexPage::MillsDicts, "Мельницы стариков: Словари", &ui_font);
                codex_entry(entries, CodexPage::IfFunctions, "ИФМОСТРОЙ-14: Функция", &ui_font);
                codex_entry(entries, CodexPage::IfLambda, "ИФМОСТРОЙ-14: Лямбды", &ui_font);
                codex_entry(entries, CodexPage::IfSets, "ИФМОСТРОЙ-14: Множества", &ui_font);
                codex_entry(entries, CodexPage::RoofRandom, "Разрушенные крыши: Библиотеки. random", &ui_font);
                codex_entry(entries, CodexPage::KnightsIntro, "PEP8 — письмо", &ui_font);
                codex_entry(entries, CodexPage::KnightsNames, "Рыцари: Переменные", &ui_font);
                codex_entry(entries, CodexPage::KnightsComments, "Рыцари: Комментарии и длины", &ui_font);
                codex_entry(entries, CodexPage::KnightsSpaces, "Рыцари: Пробелы", &ui_font);
                codex_entry(entries, CodexPage::KnightsNewline, "Рыцари: Перевод строки", &ui_font);
                codex_entry(entries, CodexPage::KnightsConditions, "Рыцари: Условия", &ui_font);
            });
            body.spawn((CodexScroll, ScrollPosition::default(), RelativeCursorPosition::default(), Node {
                flex_grow: 1.0, flex_basis: px(0), min_width: px(0), min_height: px(0),
                overflow: Overflow::scroll_y(), flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch, padding: px(16).all(), ..default() },
                BackgroundColor(Color::srgb(0.09, 0.15, 0.19))))
            .with_children(|page| {
                page.spawn((CodexText, Text::new(""), text_font(&ui_font, 17.0),
                    TextColor(Color::srgb(0.85, 0.89, 0.92)),
                    // Wrapping text must have a constrained width but retain
                    // its complete computed height inside ScrollPosition.
                    Node { width: percent(100), flex_shrink: 0.0, ..default() }));
            });
        });
    });
}

