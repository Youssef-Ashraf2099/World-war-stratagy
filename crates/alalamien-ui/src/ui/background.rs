use bevy::prelude::*;
use crate::systems::ui_manager::ScreenUI;

/// Background effect system for visual enhancement
#[derive(Component)]
pub struct BackgroundEffect;

pub fn spawn_menu_background(mut commands: Commands) {
    // Main background with gradient effect
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.1, 0.1, 0.18)),
                ..default()
            },
            BackgroundEffect,
            ScreenUI,
        ))
        .with_children(|parent| {
            // Top accent gradient
            parent.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(30.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.83, 1.0, 0.05)),
                ..default()
            });

            // Bottom accent gradient
            parent.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(30.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.91, 0.27, 0.38, 0.05)),
                ..default()
            });

            // Left accent line
            parent.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(2.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.83, 1.0, 0.2)),
                ..default()
            });

            // Right accent line
            parent.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(2.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.91, 0.27, 0.38, 0.2)),
                ..default()
            });
        });
}

/// System to animate background elements for subtle visual effect
pub fn animate_background(
    mut query: Query<&mut BackgroundColor, With<BackgroundEffect>>,
    time: Res<Time>,
) {
    // Subtle color pulsing effect based on time
    let _elapsed = time.elapsed_seconds();
    
    // Keep background static for now - can add pulsing animation later
    for mut bg_color in &mut query {
        // Background remains at base color
        *bg_color = BackgroundColor(Color::srgb(0.1, 0.1, 0.18));
    }
}
