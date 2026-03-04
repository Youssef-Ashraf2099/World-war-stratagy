use bevy::prelude::*;
use crate::ui::{MenuButton, MenuButtonAction};
use crate::audio::MenuAudio;

/// System responsible for handling navigation between screens
/// Follows Single Responsibility Principle: Only manages screen transitions
pub fn navigation_system(
    mut interaction_query: Query<
        (&Interaction, &MenuButton),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<crate::AppState>>,
    mut commands: Commands,
    menu_audio: Res<MenuAudio>,
    mut exit: EventWriter<AppExit>,
) {
    for (interaction, menu_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Play click sound
            commands.spawn(AudioBundle {
                source: menu_audio.click_sound.clone(),
                ..default()
            });

            handle_navigation(menu_button.action, &mut next_state, &mut exit);
        }
    }
}

/// Handles the logic of navigation based on button action
/// Separated for testability and clarity (SOLID: Single Responsibility)
fn handle_navigation(
    action: MenuButtonAction,
    next_state: &mut ResMut<NextState<crate::AppState>>,
    exit: &mut EventWriter<AppExit>,
) {
    use crate::AppState;

    match action {
        MenuButtonAction::NewGame => {
            info!("New Game pressed - transitioning to Loading");
            next_state.set(AppState::Loading);
        }
        MenuButtonAction::LoadGame => {
            info!("Load Game pressed - transitioning to LoadGame screen");
            next_state.set(AppState::LoadGame);
        }
        MenuButtonAction::Settings => {
            info!("Settings pressed - transitioning to Settings");
            next_state.set(AppState::Settings);
        }
        MenuButtonAction::Credits => {
            info!("Credits pressed - transitioning to Credits");
            next_state.set(AppState::Credits);
        }
        MenuButtonAction::BackToMenu => {
            info!("Back to Menu pressed - returning to Menu");
            next_state.set(AppState::Menu);
        }
        MenuButtonAction::Quit => {
            info!("Quit pressed - exiting application");
            exit.send(AppExit::Success);
        }
    }
}
