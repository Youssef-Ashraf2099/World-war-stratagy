use bevy::prelude::*;

mod states;
mod systems;
mod ui;
mod components;
mod resources;
mod audio;
mod icon;
mod map;

use states::MenuState;
use systems::{
    menu_input::menu_input_system,
    animation::animation_system,
    button::button_hover_system,
    navigation::navigation_system,
    loading::loading_input_system,
    ui_manager::*,
    camera::{camera_pan, camera_zoom, camera_fit_world, camera_wasd},
    game_clock::{advance_clock, clock_keyboard_controls, update_date_label, GameClock},
};
use ui::hud::{update_nation_panel, update_pause_indicator, esc_to_menu};
use audio::load_menu_audio;
use icon::set_window_icon;
use map::MapPlugin;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    LoadGame,
    Credits,
    Loading,
    Game,
    Settings,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Alalamien: World War Strategy".to_string(),
                        resolution: (1280.0, 768.0).into(),
                        window_theme: Some(bevy::window::WindowTheme::Dark),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    file_path: "../../assets".to_string(),
                    ..default()
                }),
        )
        // Map/geodata plugin (registers WorldGeoData, MapLoadState, picking resources + systems)
        .add_plugins(MapPlugin)
        // State machine
        .init_state::<AppState>()
        // Persistent resources
        .init_resource::<MenuState>()
        .init_resource::<GameClock>()
        // Startup systems — icon runs in PostStartup so the window already exists
        .add_systems(Startup, (setup, load_menu_audio))
        .add_systems(PostStartup, set_window_icon)
        // ----------------------------------------------------------------
        // MENU state
        // ----------------------------------------------------------------
        .add_systems(OnEnter(AppState::Menu), setup_menu_ui)
        .add_systems(OnExit(AppState::Menu), cleanup_ui)
        .add_systems(
            Update,
            (
                menu_input_system,
                button_hover_system,
                navigation_system,
                animation_system,
                ui::animate_background,
            )
                .run_if(in_state(AppState::Menu)),
        )
        // ----------------------------------------------------------------
        // LOAD GAME screen
        // ----------------------------------------------------------------
        .add_systems(OnEnter(AppState::LoadGame), setup_load_game_ui)
        .add_systems(OnExit(AppState::LoadGame), cleanup_ui)
        .add_systems(
            Update,
            (button_hover_system, navigation_system).run_if(in_state(AppState::LoadGame)),
        )
        // ----------------------------------------------------------------
        // CREDITS screen
        // ----------------------------------------------------------------
        .add_systems(OnEnter(AppState::Credits), setup_credits_ui)
        .add_systems(OnExit(AppState::Credits), cleanup_ui)
        .add_systems(
            Update,
            (button_hover_system, navigation_system).run_if(in_state(AppState::Credits)),
        )
        // ----------------------------------------------------------------
        // LOADING state — shapefile parsing + transition to Game
        // ----------------------------------------------------------------
        .add_systems(OnEnter(AppState::Loading), setup_loading_ui)
        .add_systems(OnExit(AppState::Loading), cleanup_ui)
        .add_systems(
            Update,
            (loading_input_system, animate_loading_bar).run_if(in_state(AppState::Loading)),
        )
        // ----------------------------------------------------------------
        // GAME state — map + HUD + camera + clock
        // ----------------------------------------------------------------
        .add_systems(OnEnter(AppState::Game), setup_game_ui)
        .add_systems(OnExit(AppState::Game), cleanup_ui)
        .add_systems(
            Update,
            (
                // Camera
                camera_pan,
                camera_wasd,
                camera_zoom,
                camera_fit_world,
                // Clock
                advance_clock,
                clock_keyboard_controls,
                // HUD updates
                update_date_label,
                update_nation_panel,
                update_pause_indicator,
                // Navigation
                esc_to_menu,
            )
                .run_if(in_state(AppState::Game)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    // Camera — persistent across all states; used by map rendering + UI camera
    commands.spawn(Camera2dBundle::default());
}

