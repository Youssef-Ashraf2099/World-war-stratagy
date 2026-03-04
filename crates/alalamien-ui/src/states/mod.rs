use bevy::prelude::*;

pub mod menu;

#[derive(Resource, Debug, Clone)]
pub struct MenuState {
    pub selected_button: usize,
    pub total_buttons: usize,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected_button: 0,
            total_buttons: 5, // New Game, Load Game, Settings, Credits, Quit
        }
    }
}

#[allow(dead_code)]
pub enum MenuAction {
    NewGame,
    LoadGame,
    Settings,
    Credits,
    Quit,
}
