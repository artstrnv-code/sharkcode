fn keyboard_editor(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut buffer: ResMut<CodeBuffer>, mut dev_input: ResMut<DevInput>,
    mut game_mut: ResMut<Game>,
) {
    let ctrl=keys.pressed(KeyCode::ControlLeft)||keys.pressed(KeyCode::ControlRight);
    let shift=keys.pressed(KeyCode::ShiftLeft)||keys.pressed(KeyCode::ShiftRight);
    let alt=keys.pressed(KeyCode::AltLeft)||keys.pressed(KeyCode::AltRight);
    let console=game_mut.console_open;
    let editable=matches!(game_mut.stages[game_mut.selected].node_type.as_str(),"enemy.code"|"chest.code"|"enemy.project")
        && !game_mut.codex_open && game_mut.completion.is_none()
        && !game_mut.finished_encounter(game_mut.selected);
    let mut changed = false;
    for event in events.read(){
        if !event.state.is_pressed(){continue;}
        if event.key_code==KeyCode::F12 && !game_mut.codex_open {
            game_mut.console_open=!game_mut.console_open;continue;
        }
        // Закрытие полноэкранного Кодекса доступно даже без редактора.
        if game_mut.codex_open {
            if event.key_code==KeyCode::Escape { game_mut.codex_open=false; }
            continue;
        }
        if console {
            match &event.logical_key {
                Key::Enter => {let cmd=dev_input.line.clone();dev_input.line.clear();game_mut.console_command(&cmd);}
                Key::Backspace => {dev_input.line.pop();}
                Key::Character(value) if !ctrl && !alt => {
                    let val=event.text.as_deref().unwrap_or(value.as_str());
                    if !val.chars().any(char::is_control){dev_input.line.push_str(val);}
                }
                Key::Space => dev_input.line.push(' '),
                _=>{}
            }
            continue;
        }
        if !editable {continue;}
        if ctrl {
            if let Key::Character(ch)=&event.logical_key {
                match ch.to_lowercase().as_str(){
                    "a"=>{buffer.anchor=Some(0);buffer.cursor=buffer.source.len();changed=true;}
                    "c"=>{if let Some((a,b))=buffer.selection(){if let Ok(mut cb)=arboard::Clipboard::new(){let _=cb.set_text(buffer.source[a..b].to_string());}}}
                    "x"=>{if let Some((a,b))=buffer.selection(){if let Ok(mut cb)=arboard::Clipboard::new(){let _=cb.set_text(buffer.source[a..b].to_string());}buffer.save_history();buffer.erase_selection();changed=true;}}
                    // Вставка кода намеренно запрещена в игровом редакторе.
                    "v"=>{},
                    "z"=>{buffer.undo(shift);changed=true;}
                    "y"=>{buffer.undo(true);changed=true;}
                    _=>{}
                }
            }
            match event.logical_key {
                Key::Home=>{buffer.move_to(0,shift);changed=true;}
                Key::End=>{let len=buffer.source.len();buffer.move_to(len,shift);changed=true;}
                _=>{}
            }
            continue;
        }
        if alt{continue;}
        match &event.logical_key{
            Key::Backspace=>{buffer.backspace();changed=true;}
            Key::Delete=>{buffer.delete();changed=true;}
            Key::ArrowLeft=>{buffer.move_left(shift);changed=true;}
            Key::ArrowRight=>{buffer.move_right(shift);changed=true;}
            Key::ArrowUp=>{buffer.move_vertical(true,shift);changed=true;}
            Key::ArrowDown=>{buffer.move_vertical(false,shift);changed=true;}
            Key::PageUp=>{for _ in 0..8 {buffer.move_vertical(true,shift);}changed=true;}
            Key::PageDown=>{for _ in 0..8 {buffer.move_vertical(false,shift);}changed=true;}
            Key::Home=>{let pos=buffer.line_start();buffer.move_to(pos,shift);changed=true;}
            Key::End=>{let pos=buffer.line_end();buffer.move_to(pos,shift);changed=true;}
            Key::Enter=>{
                let start = buffer.line_start();
                let at_end = buffer.cursor == buffer.line_end();
                let line = buffer.source[start..buffer.cursor].trim();
                let sword = game_mut.inventory.get("justice_sword").copied().unwrap_or(0) > 0;
                let spear = game_mut.inventory.get("justice_spear").copied().unwrap_or(0) > 0;
                let auto_colon = at_end && !line.ends_with(':') && !line.contains('#')
                    && ((sword && line.starts_with("for ") && line.contains(" in "))
                        || (spear && line.starts_with("while ") && line.len() > 6));
                buffer.insert(if auto_colon {":\n"} else {"\n"});
                changed=true;
            }
            Key::Tab=>{buffer.insert("    ");changed=true;}
            Key::Space=>{buffer.insert(" ");changed=true;}
            Key::Character(val)=>{let value=event.text.as_deref().unwrap_or(val.as_str());
                if !value.chars().any(char::is_control) {
                    let double_shield = game_mut.inventory.get("double_shield").copied().unwrap_or(0) > 0;
                    if double_shield && value == "(" { buffer.insert_open_paren(); }
                    else if double_shield && value == ")" { buffer.insert_close_paren(); }
                    else { buffer.insert(value); }
                    changed=true;
                }}
            _=>{}
        }
    }
    // Перерисовка текста и автопрокрутка выполняются одной системой ниже.
    let _ = changed;
}

fn mouse_editor(
    game: Res<Game>, mut buffer: ResMut<CodeBuffer>,
    mouse: Res<ButtonInput<MouseButton>>,
    editor: Query<(&Interaction,&RelativeCursorPosition,&ComputedNode,&ScrollPosition),With<EditorPanel>>,
) {
    if game.console_open || game.codex_open || game.completion.is_some(){return;}
    if game.finished_encounter(game.selected)
        || !matches!(game.stages[game.selected].node_type.as_str(),"enemy.code"|"chest.code"|"enemy.project") { return; }
    if mouse.just_released(MouseButton::Left){buffer.dragging=false;}
    if let Ok((interaction,pos,size,scroll))=editor.single(){
        if *interaction==Interaction::Pressed && mouse.pressed(MouseButton::Left) {
            if let Some(v)=pos.normalized {
                // RelativeCursorPosition: центр=(0,0), границы -0.5..0.5.
                let x=(v.x+0.5)*size.size().x-10.0;
                let y=(v.y+0.5)*size.size().y-10.0+scroll.y;
                let row=(y.max(0.0)/22.0).floor() as usize;
                let numbered = game.inventory.get("neo_plow").copied().unwrap_or(0) > 0;
                let prefix = if numbered { (buffer.source.lines().count().max(1).to_string().len().max(2) + 3) as f32 * 10.0 } else { 0.0 };
                let col=((x-prefix).max(0.0)/10.0).round() as usize;
                let target=buffer.position_at(row,col);
                if mouse.just_pressed(MouseButton::Left){
                    buffer.dragging=true;
                    buffer.move_to(target,false);
                    buffer.anchor=Some(target);
                } else if buffer.dragging {buffer.move_to(target,true);}

            }
        }
    }
}

#[derive(Component)] struct MapButtonLabel(usize);
#[derive(Component)] struct SecretRoute;

fn codex_scroll(
    game: Res<Game>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut pages: Query<(&mut ScrollPosition, &RelativeCursorPosition, &ComputedNode), With<CodexScroll>>,
    mut entries: Query<(&mut ScrollPosition, &RelativeCursorPosition, &ComputedNode), (With<CodexListScroll>, Without<CodexScroll>)>,
) {
    for event in wheel_events.read() {
        if !game.codex_open { continue; }
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y * 26.0,
            MouseScrollUnit::Pixel => event.y,
        };
        if let Ok((mut position, pointer, size)) = entries.single_mut() {
            if pointer.normalized.is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5) {
                let max = (size.content_size().y - size.size().y).max(0.0) * size.inverse_scale_factor;
                position.y = (position.y - delta).clamp(0.0, max);
                continue;
            }
        }
        if let Ok((mut position, pointer, size)) = pages.single_mut() {
            if pointer.normalized.is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5) {
                let max = (size.content_size().y - size.size().y).max(0.0) * size.inverse_scale_factor;
                position.y = (position.y - delta).clamp(0.0, max);
            }
        }
    }
}

// Новая скрижаль начинается с первой строки. Нельзя сбрасывать scroll
// каждый кадр: это раньше делало длинный текст практически непрокручиваемым.
fn reset_codex_scroll(
    game: Res<Game>,
    mut scroll: Query<&mut ScrollPosition, With<CodexScroll>>,
    mut previous: Local<Option<CodexPage>>,
) {
    if *previous != Some(game.codex_page) {
        if let Ok(mut position) = scroll.single_mut() { position.y = 0.0; }
        *previous = Some(game.codex_page);
    }
}

// Прокрутка редактора только при наведении на редактор.
fn editor_scroll(
    game: Res<Game>, mut events: MessageReader<MouseWheel>,
    mut editor: Query<(&mut ScrollPosition, &RelativeCursorPosition, &ComputedNode), With<EditorPanel>>,
) {
    for event in events.read() {
        if game.codex_open || game.console_open { continue; }
        if let Ok((mut position, pointer, size)) = editor.single_mut() {
            if pointer.normalized.is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5) {
                let delta = match event.unit {
                    MouseScrollUnit::Line => event.y * 26.0,
                    MouseScrollUnit::Pixel => event.y,
                };
                let max = (size.content_size().y - size.size().y).max(0.0) * size.inverse_scale_factor;
                position.y = (position.y - delta).clamp(0.0, max);
            }
        }
    }
}

// Колесо мыши над результатами прокручивает только отчёт.
fn report_scroll(
    game: Res<Game>, mut events: MessageReader<MouseWheel>,
    mut report: Query<(&mut ScrollPosition, &RelativeCursorPosition, &ComputedNode), With<ReportScroll>>,
) {
    for event in events.read() {
        if game.codex_open || game.console_open { continue; }
        if let Ok((mut position, pointer, size)) = report.single_mut() {
            if pointer.normalized.is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5) {
                let delta = match event.unit {
                    MouseScrollUnit::Line => event.y * 26.0,
                    MouseScrollUnit::Pixel => event.y,
                };
                let max = (size.content_size().y - size.size().y).max(0.0)
                    * size.inverse_scale_factor;
                position.y = (position.y - delta).clamp(0.0, max);
            }
        }
    }
}

// Прокрутка карты не должна перехватывать колесо над редактором.
fn map_scroll(
    game: Res<Game>, mut events: MessageReader<MouseWheel>,
    editor: Query<&RelativeCursorPosition, With<EditorPanel>>,
    mut map: Query<(&mut ScrollPosition, &RelativeCursorPosition, &ComputedNode), With<MapScroll>>,
) {
    for event in events.read() {
        if game.codex_open { continue; }
        let pointer_over_editor = editor.single().ok().and_then(|p|p.normalized)
            .is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5);
        if pointer_over_editor { continue; }
        if let Ok((mut position, pointer, size)) = map.single_mut() {
            if pointer.normalized.is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5) {
                let delta = match event.unit {
                    MouseScrollUnit::Line => event.y * 26.0,
                    MouseScrollUnit::Pixel => event.y,
                };
                let max = (size.content_size().y - size.size().y).max(0.0) * size.inverse_scale_factor;
                position.y = (position.y - delta).clamp(0.0, max);
            }
        }
    }
}

// Редактор отображает каретку внутри текстового потока. По изменению
// реального курсора подводим вертикальную прокрутку к активной строке.
fn update_editor_view(
    buffer: Res<CodeBuffer>, game: Res<Game>, mut commands: Commands,
    mut texts: Query<(Entity, &mut Text, &TextFont), With<CodeEditor>>,
    mut panels: Query<(&mut ScrollPosition, &ComputedNode), With<EditorPanel>>,
    mut last: Local<Option<(usize, usize, usize, u32, u32)>>,
) {
    let marker = (buffer.cursor, buffer.source.len(), game.selected,
        game.inventory.get("neo_plow").copied().unwrap_or(0),
        game.inventory.get("gold_idol").copied().unwrap_or(0));
    let changed = buffer.is_changed() || last.as_ref() != Some(&marker);
    if !changed { return; }
    if let Ok((entity, mut text, font)) = texts.single_mut() {
        let visible = display_code(&buffer,
            game.inventory.get("neo_plow").copied().unwrap_or(0) > 0);
        // Спаны принадлежат только редактору. Пересоздаются при вводе,
        // а не каждый кадр; исходный код от подсветки не изменяется.
        commands.entity(entity).despawn_related::<Children>();
        if game.inventory.get("gold_idol").copied().unwrap_or(0) > 0 {
            text.0.clear();
            let font=font.clone();
            commands.entity(entity).with_children(|parent| {
                for (part,keyword) in keyword_segments(&visible) {
                    let color=if keyword {Color::srgb(0.97,0.72,0.34)} else {Color::WHITE};
                    parent.spawn((TextSpan::new(part),font.clone(),TextColor(color)));
                }
            });
        } else {text.0=visible;}
    }
    if let Ok((mut scroll, size)) = panels.single_mut() {
        // Сбрасываем прокрутку при переходе в другой нод.
        if last.as_ref().is_some_and(|(_, _, previous, _, _)| *previous != game.selected) {
            scroll.y = 0.0;
        } else {
            let (row, _) = buffer.cursor_position();
            let row_y = row as f32 * 22.0;
            let visible = (size.size().y - 20.0).max(22.0);
            if row_y < scroll.y { scroll.y = row_y; }
            if row_y + 22.0 > scroll.y + visible {
                scroll.y = row_y + 22.0 - visible;
            }
        }
    }
    *last = Some(marker);
}

