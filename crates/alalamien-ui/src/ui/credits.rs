use bevy::prelude::*;
use super::{MenuButton, MenuButtonAction};
use crate::systems::ui_manager::ScreenUI;

/// Spawns the credits screen UI
pub fn spawn_credits(commands: &mut Commands) {
    // Credits container
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
                    "CREDITS",
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

            // Game Title
            parent.spawn(
                TextBundle::from_section(
                    "ALALAMIEN: World War Strategy",
                    TextStyle {
                        font_size: 32.0,
                        color: Color::srgb(0.93, 0.93, 0.93),
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

            // Developer Credit
            parent.spawn(
                TextBundle::from_section(
                    "Developed by",
                    TextStyle {
                        font_size: 20.0,
                        color: Color::srgb(0.6, 0.6, 0.6),
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect {
                        top: Val::Px(20.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    ..default()
                }),
            );

            parent.spawn(
                TextBundle::from_section(
                    "Youssef Ashraf",
                    TextStyle {
                        font_size: 36.0,
                        color: Color::srgb(0.91, 0.27, 0.38),
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

            // Version info
            parent.spawn(
                TextBundle::from_section(
                    "Version 0.8.0 - Phase 0.5",
                    TextStyle {
                        font_size: 16.0,
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
