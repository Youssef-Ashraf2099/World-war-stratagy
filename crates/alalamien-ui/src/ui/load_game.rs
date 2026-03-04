use bevy::prelude::*;
use super::{MenuButton, MenuButtonAction};
use crate::systems::ui_manager::ScreenUI;

/// Spawns the load game screen UI
pub fn spawn_load_game(commands: &mut Commands) {
    // Load game container
    commands
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
        .with_children(|parent| {
            // Title
            parent.spawn(
                TextBundle::from_section(
                    "LOAD GAME",
                    TextStyle {
                        font_size: 56.0,
                        color: Color::srgb(0.0, 0.83, 1.0),
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
            );

            // Message
            parent.spawn(
                TextBundle::from_section(
                    "No saved games found.",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.6, 0.6, 0.6),
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
            );

            parent.spawn(
                TextBundle::from_section(
                    "Save game functionality will be available in Phase 1.",
                    TextStyle {
                        font_size: 18.0,
                        color: Color::srgb(0.5, 0.5, 0.5),
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
            );

            // Back button
            parent
                .spawn((
                    ButtonBundle {
                        style: create_button_style(),
                        background_color: BackgroundColor(Color::srgb(0.06, 0.21, 0.38)),
                        ..default()
                    },
                    MenuButton {
                        action: MenuButtonAction::BackToMenu,
                    },
                ))
                .with_children(|button| {
                    button.spawn(TextBundle::from_section(
                        "BACK TO MENU",
                        TextStyle {
                            font_size: 24.0,
                            color: Color::srgb(0.93, 0.93, 0.93),
                            ..default()
                        },
                    ));
                });
        });
}

fn create_button_style() -> Style {
    Style {
        width: Val::Px(220.0),
        height: Val::Px(60.0),
        margin: UiRect::all(Val::Px(12.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}
