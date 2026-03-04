use bevy::prelude::*;
use crate::AppState;
use crate::map::map_plugin::MapLoadState;

/// Polls geodata loading progress.
/// When `MapLoadState.done` is true (set by `load_geodata_sync` in OnEnter(Loading)),
/// transitions to `AppState::Game`.
pub fn loading_input_system(
    load_state: Res<MapLoadState>,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Allow ESC to abort back to menu
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(AppState::Menu);
        return;
    }

    if load_state.done {
        info!("Geodata ready ({} nations) — entering Game", load_state.nations_loaded);
        next_state.set(AppState::Game);
    }
}
