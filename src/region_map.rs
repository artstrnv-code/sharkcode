//! Regional map rendering. Node positions and edges live outside main.rs.
use crate::*;

// Оформление карты и размещение нодов не зависят от количества регионов.
// Отдельные карты предотвращают склеивание секретной ветки с Долиной.
pub(super) fn spawn_location_map(parent: &mut ChildSpawnerCommands, location: Location,
    game: &Game, font: &Handle<Font>) {
    let river_items: Vec<(usize, f32, f32)> =
        std::iter::once(17).chain(river::RIVER_FIRST..=for4::river_to_for4(&game.stages))
            .enumerate().map(|(i, index)| {
                let mimic = game.stages[index].id.contains("_mimic_");
                (index, if mimic {190.0} else {19.0},
                 85.0 + 91.0 * (i.saturating_sub(usize::from(mimic))) as f32)
            }).collect();
    let for4_items: Vec<(usize,f32,f32)> = (for4::for4_first(&game.stages)..tower::river_to_tower(&game.stages))
        .enumerate().map(|(i,index)| {
            let mimic = game.stages[index].id.contains("_mimic_");
            (index, if mimic {190.0} else {19.0},
             85.0 + 91.0 * i.saturating_sub(usize::from(mimic)) as f32)
        }).collect();
    let tower_items: Vec<(usize,f32,f32)> = (tower::tower_first(&game.stages)..field::first(&game.stages)-2)
        .enumerate().map(|(i,index)| {
            let mimic = game.stages[index].id.contains("_mimic_");
            (index, if mimic {190.0} else {19.0},
             85.0 + 91.0 * i.saturating_sub(usize::from(mimic)) as f32)
        }).collect();
    let field_items: Vec<(usize,f32,f32)> = (field::first(&game.stages)..mills::first(&game.stages))
        .enumerate().map(|(i,index)| {
            let side=game.stages[index].id=="field_new_chest";
            (index,if side{190.0}else{19.0},85.0+91.0*(i.saturating_sub(usize::from(side))) as f32)
        }).collect();
    let mills_items: Vec<(usize,f32,f32)> = (mills::first(&game.stages)..game.stages.len())
        .filter(|&index| game.stages[index].id.starts_with("mills_"))
        .enumerate().map(|(i,index)| {
            let side=game.stages[index].id=="mills_chest";
            let last=game.stages[index].id.starts_with("mills_to_");
            (index,if side||last {190.0}else{19.0},85.0+91.0*(i.saturating_sub(usize::from(side))) as f32)
        }).collect();
    // Отдельный выход из Пашни и карта ИФМОСТРОЙ-14.
    let if_exit = game.stages.iter().position(|stage| stage.id == "field_to_if")
        .expect("field_to_if missing");
    let if_items: Vec<(usize, f32, f32)> =
        (ifmostroy::first(&game.stages)..roofs::first(&game.stages)-1)
        .enumerate().map(|(i,index)| {
            let id=game.stages[index].id.as_str();
            let side=id=="if_mimic" || id=="if_to_oop";
            (index, if side {190.0}else{19.0},
             85.0+91.0*(i.saturating_sub(usize::from(side))) as f32)
        }).collect();
    let roofs_items: Vec<(usize,f32,f32)> =
        (roofs::first(&game.stages)..game.stages.len())
        .enumerate().map(|(i,index)|(index,19.0,85.0 + 91.0 * i as f32)).collect();
    let mut field_with_exit = field_items.clone();
    field_with_exit.push((if_exit,190.0,85.0+91.0*(field_items.len().saturating_sub(2)) as f32));
    let mut for4_with_exit = for4_items.clone();
    for4_with_exit.push((field::first(&game.stages)-2,190.0,
        85.0+91.0*(for4_items.len().saturating_sub(1)) as f32));
    let mut tower_with_exit = tower_items.clone();
    tower_with_exit.push((field::first(&game.stages)-1,190.0,
        85.0+91.0*(tower_items.len().saturating_sub(1)) as f32));
    let mut river_with_two_exits = river_items.clone();
    river_with_two_exits.push((tower::river_to_tower(&game.stages),190.0,
        85.0 + 91.0 * (river_items.len().saturating_sub(1)) as f32));
    let mut valley_items = vec![(0,19.,85.),(1,19.,176.),(2,19.,267.),
        (3,19.,358.),(4,19.,449.),(5,19.,540.),(6,19.,631.),
        (7,190.,267.),(16,19.,722.),(8,190.,176.),(18,19.,813.)];
    for (i, stage) in game.stages.iter().enumerate() {
        if stage.id == "project_converter" { valley_items.push((i, 190., 540.)); }
        if stage.id == "project_void" { valley_items.push((i, 190., 631.)); }
    }
    let (tint, accent, description, items, canvas_height):
        (Color, Color, &str, &[(usize, f32, f32)], f32) = match location {
        Location::Valley => (
            Color::srgb(0.12,0.19,0.18), Color::srgb(0.44,0.69,0.59),
            "ТРОПА ПЕРЕМЕННЫХ  /  начало пути",
            &valley_items, 915.0),
        Location::Grot => (
            Color::srgb(0.14,0.13,0.21), Color::srgb(0.61,0.49,0.82),
            "СКРЫТЫЕ СВОДЫ  /  путь знаний",
            &[(9,19.,85.),(10,19.,176.),(11,19.,267.),(12,19.,358.),
              (13,19.,449.),(14,19.,540.),(15,19.,631.),(19,19.,722.)], 815.0),
        Location::River => (
            Color::srgb(0.09,0.18,0.23), Color::srgb(0.38,0.72,0.85),
            "РЕКА СВЯТОГО ЛЕСА  /  input и ветвления",
            &river_with_two_exits, 115.0 + 91.0 * river_items.len() as f32),
        Location::For4 => (
            Color::srgb(0.19,0.16,0.10), Color::srgb(0.80,0.68,0.40),
            "4 4  /  for и range",
            &for4_with_exit, 115.0 + 91.0 * for4_items.len() as f32),
        Location::Tower => (
            Color::srgb(0.12,0.12,0.19), Color::srgb(0.68,0.71,0.93),
            "БЕСКОНЕЧНАЯ БАШНЯ  /  while, break, continue",
            &tower_with_exit, 115.0 + 91.0 * tower_items.len() as f32),
        Location::Field => (
            Color::srgb(0.13,0.20,0.12), Color::srgb(0.63,0.82,0.42),
            "ВЕЛИКАЯ ПАШНЯ  /  списки и сортировка",
            &field_with_exit, 115.0 + 91.0 * field_items.len() as f32),
        Location::Ifmostroy => (
            Color::srgb(0.13,0.17,0.22), Color::srgb(0.56,0.78,0.88),
            "ИФМОСТРОЙ-14  /  функции, lambda и множества",
            &if_items, 115.0 + 91.0 * if_items.len() as f32),
        Location::Roofs => (
            Color::srgb(0.14,0.15,0.20), Color::srgb(0.76,0.75,0.90),
            "РАЗРУШЕННЫЕ КРЫШИ  /  библиотеки и random",
            &roofs_items, 115.0 + 91.0 * roofs_items.len() as f32),
        Location::Mills => (
            Color::srgb(0.18,0.15,0.12), Color::srgb(0.82,0.66,0.44),
            "МЕЛЬНИЦЫ СТАРИКОВ  /  кортежи и словари",
            &mills_items, 115.0 + 91.0 * mills_items.len() as f32),
    };
    parent.spawn((LocationMap(location), Node {
        display: if location == Location::Valley {Display::Flex}else{Display::None},
        position_type: PositionType::Relative, width: percent(100),
        height: px(canvas_height), flex_shrink: 0.0, ..default()
    }, BackgroundColor(tint)))
    .with_children(|map| {
        map.spawn((Text::new(description), text_font(font,13.0),
            TextColor(accent), Node {position_type:PositionType::Absolute,
                top:px(17),left:px(16), ..default()}));
        // Небольшие географические акценты и дорожные маркеры.
        for (x, y) in [(330.,55.),(318.,235.),(336.,414.),(322.,592.)] {
            map.spawn((Text::new("+"), text_font(font,22.0),
                TextColor(Color::srgba(0.64,0.78,0.74,0.42)),
                Node {position_type:PositionType::Absolute,left:px(x),top:px(y),..default()}));
        }
        if location == Location::River {
            for y in [90.,128.,166.,204.,242.,280.] {
                map.spawn((Node {position_type:PositionType::Absolute,left:px(290),
                    top:px(y), width:px(12),height:px(26), ..default()},
                    BackgroundColor(Color::srgb(0.14,0.39,0.48))));
            }
        }
        // Рёбра описывают реальные связи, а не случайный порядок в массиве.
        let edges: &[(usize, usize)] = match location {
            Location::Valley => &[(0,1),(1,2),(2,3),(3,4),(4,5),(5,6),
                (6,16),(16,18),(2,7),(1,8)],
            Location::Grot => &[(9,10),(10,11),(11,12),(12,13),(13,14),
                (14,15),(15,19)],
            Location::River | Location::For4 | Location::Tower | Location::Field | Location::Mills | Location::Ifmostroy | Location::Roofs => &[],
        };
        let dynamic_edges: Vec<(usize, usize)> = if location == Location::River {
            let mut edges: Vec<(usize,usize)> = std::iter::once(17)
                .chain(river::RIVER_FIRST..=for4::river_to_for4(&game.stages))
                .collect::<Vec<_>>().windows(2).map(|w| (w[0], w[1])).collect();
            edges.push((for4::river_to_for4(&game.stages)-1, tower::river_to_tower(&game.stages)));
            edges
        } else if location == Location::For4 {
            let mut edges = (for4::for4_first(&game.stages)..tower::river_to_tower(&game.stages))
                .collect::<Vec<_>>().windows(2).map(|w| (w[0],w[1])).collect::<Vec<_>>();
            edges.push((tower::river_to_tower(&game.stages)-1,field::first(&game.stages)-2));
            edges
        } else if location == Location::Tower {
            let mut edges = (tower::tower_first(&game.stages)..field::first(&game.stages)-2).collect::<Vec<_>>()
                .windows(2).map(|w| (w[0], w[1])).collect::<Vec<_>>();
            edges.push((field::first(&game.stages)-3,field::first(&game.stages)-1));
            edges
        } else if location == Location::Field {
            (field::first(&game.stages)..mills::first(&game.stages)).collect::<Vec<_>>()
                .windows(2).map(|w| (w[0],w[1])).collect()
        } else if location == Location::Ifmostroy {
            let start = ifmostroy::first(&game.stages);
            let all = (start..roofs::first(&game.stages)-1).collect::<Vec<_>>();
            let main = all.iter().copied().filter(|&i| !matches!(game.stages[i].id.as_str(),
                "if_mimic" | "if_to_oop")).collect::<Vec<_>>();
            let mut edges=main.windows(2).map(|w|(w[0],w[1])).collect::<Vec<_>>();
            if let Some(&mimic)=all.iter().find(|&&i|game.stages[i].id=="if_mimic") {
                edges.push((mimic-1,mimic));
            }
            edges
        } else if location == Location::Roofs {
            (roofs::first(&game.stages)..game.stages.len())
                .collect::<Vec<_>>().windows(2).map(|w|(w[0],w[1])).collect()
        } else if location == Location::Mills {
            let start=mills::first(&game.stages);
            let all=(start..game.stages.len()).filter(|&i| game.stages[i].id.starts_with("mills_"))
                .collect::<Vec<_>>();
            let main=all.iter().copied().filter(|&i| !matches!(game.stages[i].id.as_str(),
                "mills_chest"|"mills_to_prison"|"mills_to_matrix"|"mills_to_roofs")).collect::<Vec<_>>();
            let mut edges=main.windows(2).map(|w|(w[0],w[1])).collect::<Vec<_>>();
            if let Some(&boss)=main.iter().find(|&&i|game.stages[i].id=="mills_boss") {
                if let Some(&chest)=all.iter().find(|&&i|game.stages[i].id=="mills_chest") {edges.push((boss,chest));}
            }
            if let (Some(&clear), Some(&exit)) = (
                all.iter().find(|&&i| game.stages[i].id == "mills_clear"),
                all.iter().find(|&&i| game.stages[i].id == "mills_to_roofs")) {
                edges.push((clear, exit));
            }
            edges
        } else { Vec::new() };
        // Side chest nodes must never appear between the boss and clear path.
        let mut filtered_edges: Vec<(usize, usize)> = Vec::new();
        for (from, to) in dynamic_edges {
            if game.stages[to].id.contains("_mimic_") {
                filtered_edges.push((from, to));
            } else if game.stages[from].id.contains("_mimic_") {
                filtered_edges.push((from - 1, to));
            } else {
                filtered_edges.push((from, to));
            }
        }
        if location == Location::Field {
            // Ветвь к ИФМОСТРОЙ-14 не зависит от входа в Мельницы.
            if let Some(clear) = field_items.iter().find(|(i,_,_)| game.stages[*i].id=="field_clear") {
                filtered_edges.push((clear.0,if_exit));
            }
            if let Some(chest)=field_items.iter().find(|(i,_,_)|game.stages[*i].id=="field_new_chest").map(|x|x.0) {
                filtered_edges.retain(|&(a,b)|!(a==chest && b==chest+1));
                filtered_edges.push((chest-1,chest+1));
            }
        }
        for &(from,to) in edges.iter().chain(filtered_edges.iter()) {
            if game.stages[to].id.starts_with("project_")
                && !game.ever_visited_mills && !game.dev_mode { continue; }
            // Коллекции изучаются позже: преждевременно не раскрываем мимика.
            if game.stages[to].id == "river_mimic_conditions"
                && !game.codex.contains("Коллекции") && !game.dev_mode { continue; }
            let Some(&(_, x1, y1)) = items.iter().find(|(index,_,_)| *index == from) else {continue};
            let Some(&(_, x2, y2)) = items.iter().find(|(index,_,_)| *index == to) else {continue};
            if (x1-x2).abs() < 1.0 {
                map.spawn((Node {position_type:PositionType::Absolute,
                    left:px(x1+73.0),top:px(y1+48.0),width:px(3),
                    height:px((y2-y1-48.0).max(1.0)),..default()},BackgroundColor(accent)));
            } else {
                // Секретная ветка не должна выдавать своё существование линией.
                if from == 1 && to == 8 {
                    map.spawn((SecretRoute, Node {position_type:PositionType::Absolute,
                        display: if game.grot_unlocked || game.dev_mode {Display::Flex}else{Display::None},
                        left:px(x1+150.0),top:px(y1+25.0),width:px((x2-x1-150.0).max(2.0)),
                        height:px(3),..default()},BackgroundColor(accent)));
                } else {
                    map.spawn((Node {position_type:PositionType::Absolute,
                        left:px(x1+150.0),top:px(y1+25.0),width:px((x2-x1-150.0).max(2.0)),
                        height:px(3),..default()},BackgroundColor(accent)));
                }
            }
        }
        for &(index,x,y) in items {
            if game.stages[index].id == "river_mimic_conditions"
                && !game.codex.contains("Коллекции") && !game.dev_mode { continue; }
            let kind = game.stages[index].node_type.as_str();
            // ASCII-маркеры гарантированно доступны даже со старым UI-шрифтом.
            let sigil = if game.stages[index].id.starts_with("project_") {
                "[P]"
            } else { match kind {
                "scroll"=>"[T]", "region.clear"=>"[R]", "chest.code"=>"[$]",
                "enemy.quiz"=>"[?]", "entry"=>"[>]", "empty"=>"[~]", _=>"[!]",
            }};
            map.spawn((Button,Action::Select(index),MapButtonLabel(index),Node {
                display: if game.stages[index].id.starts_with("project_")
                    && !game.ever_visited_mills && !game.dev_mode {
                    Display::None
                } else { Display::Flex },
                position_type:PositionType::Absolute,left:px(x),top:px(y),
                width:px(150),min_height:px(48),padding:px(7).all(),
                border:px(1).all(),..default()},
                BorderColor::all(accent),
                BackgroundColor(Color::srgb(0.12,0.22,0.25))))
            .with_children(|button| {
                button.spawn((Text::new(format!("{sigil}  {}",game.stages[index].label)),
                    text_font(font,13.0),TextColor(Color::WHITE)));
            });
        }
    });
}

// Отдельная прокрутка двух областей Кодекса: список страниц и текст.
// Колесо действует только над той областью, где находится указатель.

// Карта региона и кнопки переходов обновляются независимо от содержимого нодов.
// Скрытый Грот полностью отсутствует в навигации обычного игрока.
pub(super) fn update_location_map(
    game: Res<Game>,
    mut maps: Query<(&LocationMap,&mut Node), Without<MapButtonLabel>>,
    mut map_buttons: Query<(&MapButtonLabel,&mut Node), (Without<LocationMap>, Without<SecretRoute>)>,
    mut secret_routes: Query<&mut Node, (With<SecretRoute>, Without<MapButtonLabel>, Without<LocationMap>)>,
    mut heading: Query<&mut Text,With<LocationTitle>>,
    mut scroll: Query<&mut ScrollPosition,With<MapScroll>>,
    mut previous: Local<Option<Location>>,
) {
    if !game.is_changed() { return; }
    if let Ok(mut label) = heading.single_mut() {
        label.0 = game.location.title().into();
    }
    for (location,mut node) in &mut maps {
        node.display = if location.0 == game.location {Display::Flex}else{Display::None};
    }
    // Секретный вход не появляется вообще до срабатывания условия у Стесняги.
    for (button,mut node) in &mut map_buttons {
        if button.0 == 8 {
            node.display = if game.grot_unlocked || game.dev_mode {
                Display::Flex
            } else {Display::None};
        } else if game.stages[button.0].id.starts_with("project_") {
            // The map is constructed once; reveal the optional encounters when
            // a later run starts, without requiring an application restart.
            node.display = if game.ever_visited_mills || game.dev_mode {
                Display::Flex
            } else {Display::None};
        }
    }
    for mut route in &mut secret_routes {
        route.display = if game.grot_unlocked || game.dev_mode {Display::Flex} else {Display::None};
    }
    if *previous != Some(game.location) {
        if let Ok(mut position) = scroll.single_mut() {position.y = 0.0;}
        *previous = Some(game.location);
    }
}

