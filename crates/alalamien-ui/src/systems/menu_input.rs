use bevy::prelude::*;
use crate::states::MenuState;

pub fn menu_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut exit: EventWriter<AppExit>,
) {
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        menu_state.selected_button = (menu_state.selected_button + 1) % menu_state.total_buttons;
    }

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        menu_state.selected_button = if menu_state.selected_button == 0 {
            menu_state.total_buttons - 1
        } else {
            menu_state.selected_button - 1
        };
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        // Handle enter key - would trigger selected button action
        // Implementation depends on which button is selected
    }
}
