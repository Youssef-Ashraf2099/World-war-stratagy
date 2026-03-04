use bevy::prelude::*;

#[derive(Component)]
pub struct MenuButton {
    pub action: MenuButtonAction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MenuButtonAction {
    NewGame,
    LoadGame,
    Settings,
    Credits,
    Quit,
    BackToMenu,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct MenuIcon;

pub use self::menu::spawn_menu;
pub use self::credits::spawn_credits;
pub use self::load_game::spawn_load_game;
pub use self::background::{spawn_menu_background, animate_background};
pub use self::hud::spawn_hud;

pub mod menu;
pub mod credits;
pub mod load_game;
pub mod background;
pub mod hud;
