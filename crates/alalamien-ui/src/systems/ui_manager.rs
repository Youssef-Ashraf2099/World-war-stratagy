use bevy::prelude::*;
use crate::ui;

/// Marker component for UI elements that should be cleaned up on state transitions
#[derive(Component)]
pub struct ScreenUI;

/// System to cleanup UI when leaving a state
/// Follows Single Responsibility Principle: Only handles UI cleanup
pub fn cleanup_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<ScreenUI>>,
) {
    for entity in &ui_query {
        commands.entity(entity).despawn_recursive();
    }
}

/// System to setup Menu UI when entering Menu state
pub fn setup_menu_ui(mut commands: Commands) {
    ui::spawn_menu_background(commands.reborrow());
    ui::spawn_menu(&mut commands);
}

/// System to setup Credits UI when entering Credits state
pub fn setup_credits_ui(mut commands: Commands) {
    ui::spawn_credits(&mut commands);
}

/// System to setup Load Game UI when entering LoadGame state
pub fn setup_load_game_ui(mut commands: Commands) {
    ui::spawn_load_game(&mut commands);
}

/// System to setup Loading screen UI — shown while shapefiles are parsed.
pub fn setup_loading_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(20.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgb(0.06, 0.06, 0.12)),
                ..default()
            },
            ScreenUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle::from_section(
                "ALALAMIEN: WORLD WAR STRATEGY",
                TextStyle {
                    font_size: 28.0,
                    color: Color::srgb(0.0, 0.83, 1.0),
                    ..default()
                },
            ));

            // Loading message
            parent.spawn(TextBundle::from_section(
                "Loading world geodata...",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(0.80, 0.80, 0.85),
                    ..default()
                },
            ));

            // Progress bar outer frame
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(400.0),
                        height: Val::Px(8.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.15, 0.15, 0.22)),
                    border_color: BorderColor(Color::srgba(0.0, 0.75, 1.0, 0.4)),
                    ..default()
                })
                .with_children(|bar| {
                    // Animated fill (will pulse via the animate_loading_bar system)
                    bar.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Percent(60.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.0, 0.75, 1.0)),
                            ..default()
                        },
                        LoadingBar,
                    ));
                });

            // ESC hint
            parent.spawn(TextBundle::from_section(
                "ESC — return to menu",
                TextStyle {
                    font_size: 12.0,
                    color: Color::srgb(0.45, 0.45, 0.55),
                    ..default()
                },
            ));
        });
}

/// Marker for the loading progress bar inner fill.
#[derive(Component)]
pub struct LoadingBar;

/// Animate the loading bar width with a simple sine pulse.
pub fn animate_loading_bar(
    time: Res<Time>,
    mut bar_q: Query<&mut Style, With<LoadingBar>>,
) {
    let pct = (((time.elapsed_seconds() * 0.8).sin() + 1.0) * 0.5 * 80.0 + 10.0) as f32;
    for mut style in &mut bar_q {
        style.width = Val::Percent(pct);
    }
}

/// Setup the in-game screen UI (HUD panels + camera reset).
pub fn setup_game_ui(commands: Commands) {
    ui::spawn_hud(commands);
}

