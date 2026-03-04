use bevy::prelude::*;
use super::{MenuButton, MenuButtonAction};
use crate::systems::ui_manager::ScreenUI;

pub fn spawn_menu(commands: &mut Commands) {    // Main menu container (centered)
    let menu_node = commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.1, 0.1, 0.18)),
            ..default()
        }, ScreenUI))
        .id();

    // Title
    let title = commands
        .spawn(
            TextBundle::from_section(
                "ALALAMIEN",
                TextStyle {
                    font_size: 64.0,
                    color: Color::srgb(0.0, 0.83, 1.0),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(20.0),
                    ..default()
                },
                ..default()
            }),
        )
        .id();

    let subtitle = commands
        .spawn(
            TextBundle::from_section(
                "World War Strategy",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(0.6, 0.6, 0.6),
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(40.0),
                    ..default()
                },
                ..default()
            }),
        )
        .id();

    // New Game Button
    let new_game_btn = commands
        .spawn((
            ButtonBundle {
                style: create_button_style(),
                background_color: BackgroundColor(Color::srgb(0.06, 0.21, 0.38)),
                ..default()
            },
            MenuButton {
                action: MenuButtonAction::NewGame,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "NEW GAME",
                TextStyle {
                    font_size: 28.0,
                    color: Color::srgb(0.93, 0.93, 0.93),
                    ..default()
                },
            ));
        })
        .id();

    // Load Game Button
    let load_game_btn = commands
        .spawn((
            ButtonBundle {
                style: create_button_style(),
                background_color: BackgroundColor(Color::srgb(0.06, 0.21, 0.38)),
                ..default()
            },
            MenuButton {
                action: MenuButtonAction::LoadGame,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "LOAD GAME",
                TextStyle {
                    font_size: 28.0,
                    color: Color::srgb(0.93, 0.93, 0.93),
                    ..default()
                },
            ));
        })
        .id();

    // Settings Button
    let settings_btn = commands
        .spawn((
            ButtonBundle {
                style: create_button_style(),
                background_color: BackgroundColor(Color::srgb(0.06, 0.21, 0.38)),
                ..default()
            },
            MenuButton {
                action: MenuButtonAction::Settings,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "SETTINGS",
                TextStyle {
                    font_size: 28.0,
                    color: Color::srgb(0.93, 0.93, 0.93),
                    ..default()
                },
            ));
        })
        .id();

    // Credits Button
    let credits_btn = commands
        .spawn((
            ButtonBundle {
                style: create_button_style(),
                background_color: BackgroundColor(Color::srgb(0.06, 0.21, 0.38)),
                ..default()
            },
            MenuButton {
                action: MenuButtonAction::Credits,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "CREDITS",
                TextStyle {
                    font_size: 28.0,
                    color: Color::srgb(0.93, 0.93, 0.93),
                    ..default()
                },
            ));
        })
        .id();

    // Quit Button
    let quit_btn = commands
        .spawn((
            ButtonBundle {
                style: create_button_style(),
                background_color: BackgroundColor(Color::srgb(0.06, 0.21, 0.38)),
                ..default()
            },
            MenuButton {
                action: MenuButtonAction::Quit,
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "QUIT",
                TextStyle {
                    font_size: 28.0,
                    color: Color::srgb(0.93, 0.93, 0.93),
                    ..default()
                },
            ));
        })
        .id();

    // Footer
    let footer = commands
        .spawn(
            TextBundle::from_section(
                "v0.8.0 | © 2026 Alalamien War Team",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.6, 0.6, 0.6),
                    ..default()
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                ..default()
            }),
        )
        .id();

    // Add all children to menu container
    commands.entity(menu_node).push_children(&[
        title,
        subtitle,
        new_game_btn,
        load_game_btn,
        settings_btn,
        credits_btn,
        quit_btn,
        footer,
    ]);
}

fn create_button_style() -> Style {
    Style {
        width: Val::Px(200.0),
        height: Val::Px(60.0),
        margin: UiRect::all(Val::Px(12.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}