//! SHARKCODE — общее ядро игры, редактор и UI (логика регионов вынесена).
//! Карта/игровые переходы принадлежат Rust, учебный код исполняет CPython.
//! Sandbox Core ограничивает ресурсы, но ещё НЕ изолирует файлы и сеть.

use bevy::prelude::*;
use regions::Location;
mod valley;
mod regions;
mod region_map;
mod river;
mod for4;
mod tower;
mod field;
mod mills;
mod projects;
mod ifmostroy;
mod roofs;
mod sandbox;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use serde_json::{json, Value};
use serde::Deserialize;
use bevy::ui::RelativeCursorPosition;
use std::{collections::{HashSet, HashMap}, fs, path::PathBuf,
          time::{SystemTime, UNIX_EPOCH}};

const TUTORIAL: &str = include_str!("../content/tutorial_base.txt");
const ASSIGNMENT: &str = include_str!("../content/tablet_best_assignment.txt");

// SHARKCODE 0.2.52: разделение исходников без изменения игрового поведения.
// include! сохраняет прежнюю область видимости. Это промежуточный
// безрисковый этап: последующая модульная изоляция потребует проверки API.
include!("game/stages.rs");
include!("game/state.rs");
include!("game/codex.rs");
include!("game/save.rs");
include!("game/services.rs");
include!("editor/model.rs");
include!("editor/systems.rs");
include!("ui/layout.rs");
include!("ui/systems.rs");

fn main() {
    // Packaged releases may be launched from a shortcut with arbitrary working
    // directory. Resolve game resources relative to the executable instead.
    if !cfg!(debug_assertions) {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(directory) = exe.parent() {
                if let Err(err) = std::env::set_current_dir(directory) {
                    eprintln!("SHARKCODE: cannot select game directory: {err}");
                    return;
                }
            }
        }
    }
    let _ = fs::remove_dir_all(data_dir().join("session"));
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { title: "SHARKCODE — Долина переменн".into(),
                resolution: (1220, 800).into(), ..default() }), ..default()
        }))
        .insert_resource(Game::new())
        .init_resource::<CodeBuffer>()
        .init_resource::<ProjectEditor>()
        .init_resource::<DevInput>()
        .init_resource::<TextReveal>()
        .add_systems(Startup, setup)
        .add_systems(Update, (keyboard_editor, mouse_editor, buttons, sync_project_editor, update_labels, update_quiz_options, update_overlays, region_map::update_location_map, typewriter, reset_codex_scroll, codex_scroll, editor_scroll, report_scroll, map_scroll, update_editor_view).chain())
        .run();
}

