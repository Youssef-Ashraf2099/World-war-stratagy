use bevy::prelude::*;
use crate::systems::ui_manager::ScreenUI;
use crate::systems::game_clock::{DateLabel, GameClock};
use crate::map::picking::SelectedNation;

// ---------------------------------------------------------------------------
// Marker components
// ---------------------------------------------------------------------------

/// Marker for the left-panel nation detail container.
#[derive(Component)]
pub struct NationDetailPanel;

/// Marker for the nation name text in the left panel.
#[derive(Component)]
pub struct NationNameLabel;

/// Marker for the ISO code text in the left panel.
#[derive(Component)]
pub struct NationIsoLabel;

/// Marker for the "paused / playing" status indicator in the bottom bar.
#[derive(Component)]
pub struct PauseIndicator;

// ---------------------------------------------------------------------------
// Colours
// ---------------------------------------------------------------------------
const HUD_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.92);
const HUD_BORDER: Color = Color::srgba(0.0, 0.75, 1.0, 0.25);
const TEXT_PRIMARY: Color = Color::srgb(0.90, 0.90, 0.95);
const TEXT_DIM: Color = Color::srgb(0.55, 0.55, 0.65);
const TEXT_ACCENT: Color = Color::srgb(0.0, 0.83, 1.0);

// ---------------------------------------------------------------------------
// HUD spawn
// ---------------------------------------------------------------------------

/// Spawns the four-panel in-game HUD.  All elements carry [`ScreenUI`] so
/// they are automatically cleaned up by `cleanup_ui` on state exit.
pub fn spawn_hud(mut commands: Commands) {
    // ======================================================================
    // ROOT: full-screen transparent container, flex column
    // ======================================================================
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            ScreenUI,
            Name::new("HudRoot"),
        ))
        .with_children(|root| {
            // ==============================================================
            // TOP BAR
            // ==============================================================
            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(44.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(HUD_BG),
                    border_color: BorderColor(HUD_BORDER),
                    ..default()
                },
                Name::new("TopBar"),
            ))
            .with_children(|bar| {
                // --- Left side: game title / nation placeholder ---
                bar.spawn(TextBundle::from_section(
                    "ALALAMIEN: WORLD WAR STRATEGY",
                    TextStyle {
                        font_size: 14.0,
                        color: TEXT_ACCENT,
                        ..default()
                    },
                ));

                // --- Centre: resource stats row (placeholder) ---
                bar.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(24.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("ResourceBar"),
                ))
                .with_children(|res| {
                    for label in &["👥 —", "💰 —", "⚔ —", "🌾 —"] {
                        res.spawn(TextBundle::from_section(
                            *label,
                            TextStyle {
                                font_size: 12.0,
                                color: TEXT_DIM,
                                ..default()
                            },
                        ));
                    }
                });

                // --- Right side: date label ---
                bar.spawn((
                    TextBundle::from_section(
                        "Jan 1939 | Turn 0",
                        TextStyle {
                            font_size: 13.0,
                            color: TEXT_PRIMARY,
                            ..default()
                        },
                    ),
                    DateLabel,
                    Name::new("DateLabel"),
                ));
            });

            // ==============================================================
            // MIDDLE ROW: left panel + map area (map renders behind UI)
            // ==============================================================
            root.spawn((
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    ..default()
                },
                Name::new("MiddleRow"),
            ))
            .with_children(|row| {
                // --- Left nation detail panel ---
                row.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(240.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(8.0),
                            border: UiRect::right(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(HUD_BG),
                        border_color: BorderColor(HUD_BORDER),
                        ..default()
                    },
                    NationDetailPanel,
                    Name::new("LeftPanel"),
                ))
                .with_children(|panel| {
                    panel.spawn(TextBundle::from_section(
                        "SELECT A NATION",
                        TextStyle {
                            font_size: 11.0,
                            color: TEXT_DIM,
                            ..default()
                        },
                    ));

                    // Nation name
                    panel.spawn((
                        TextBundle::from_section(
                            "—",
                            TextStyle {
                                font_size: 18.0,
                                color: TEXT_PRIMARY,
                                ..default()
                            },
                        ),
                        NationNameLabel,
                    ));

                    // ISO code
                    panel.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font_size: 12.0,
                                color: TEXT_ACCENT,
                                ..default()
                            },
                        ),
                        NationIsoLabel,
                    ));

                    // Divider
                    panel.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(4.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(HUD_BORDER),
                        ..default()
                    });

                    // Placeholder stats
                    for stat in &["Population: —", "GDP: —", "Stability: —", "Legitimacy: —"] {
                        panel.spawn(TextBundle::from_section(
                            *stat,
                            TextStyle {
                                font_size: 12.0,
                                color: TEXT_DIM,
                                ..default()
                            },
                        ));
                    }

                    // Controls hint
                    panel.spawn(TextBundle::from_section(
                        "\nCONTROLS\nWASD / Arrows — pan\nScroll — zoom to cursor\nRMB drag — pan\nF — fit world\n\nSPEED\nSpace — pause/play\n1 / 2 / 3 — speed",
                        TextStyle {
                            font_size: 10.0,
                            color: TEXT_DIM,
                            ..default()
                        },
                    ));
                });
            });

            // ==============================================================
            // BOTTOM BAR
            // ==============================================================
            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(48.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(16.0)),
                        border: UiRect::top(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(HUD_BG),
                    border_color: BorderColor(HUD_BORDER),
                    ..default()
                },
                Name::new("BottomBar"),
            ))
            .with_children(|bar| {
                // Left: pause indicator + speed
                bar.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("ClockControls"),
                ))
                .with_children(|ctrl| {
                    // Pause / play label (updated by system)
                    ctrl.spawn((
                        TextBundle::from_section(
                            "⏸ PAUSED",
                            TextStyle {
                                font_size: 13.0,
                                color: TEXT_ACCENT,
                                ..default()
                            },
                        ),
                        PauseIndicator,
                    ));
                    ctrl.spawn(TextBundle::from_section(
                        "|  1×  2×  4×",
                        TextStyle {
                            font_size: 12.0,
                            color: TEXT_DIM,
                            ..default()
                        },
                    ));
                });

                // Centre: event log placeholder
                bar.spawn(TextBundle::from_section(
                    "No events",
                    TextStyle {
                        font_size: 12.0,
                        color: TEXT_DIM,
                        ..default()
                    },
                ));

                // Right: ESC hint
                bar.spawn(TextBundle::from_section(
                    "ESC — menu",
                    TextStyle {
                        font_size: 11.0,
                        color: TEXT_DIM,
                        ..default()
                    },
                ));
            });
        });
}

// ---------------------------------------------------------------------------
// HUD Update systems
// ---------------------------------------------------------------------------

/// Sync the left-panel nation labels to the currently selected nation.
pub fn update_nation_panel(
    selected: Res<SelectedNation>,
    mut name_q: Query<&mut Text, (With<NationNameLabel>, Without<NationIsoLabel>)>,
    mut iso_q: Query<&mut Text, (With<NationIsoLabel>, Without<NationNameLabel>)>,
) {
    if !selected.is_changed() {
        return;
    }
    for mut text in &mut name_q {
        if let Some(section) = text.sections.first_mut() {
            section.value = selected.name.clone().unwrap_or_else(|| "—".to_string());
        }
    }
    for mut text in &mut iso_q {
        if let Some(section) = text.sections.first_mut() {
            section.value = selected.iso_a3.clone().unwrap_or_default();
        }
    }
}

/// Sync the bottom-bar pause indicator to the current clock state.
pub fn update_pause_indicator(
    clock: Res<GameClock>,
    mut query: Query<&mut Text, With<PauseIndicator>>,
) {
    if !clock.is_changed() {
        return;
    }
    let label = if clock.paused {
        format!("⏸ PAUSED  {}", clock.speed.label())
    } else {
        format!("▶ PLAYING  {}", clock.speed.label())
    };
    for mut text in &mut query {
        if let Some(section) = text.sections.first_mut() {
            section.value = label.clone();
        }
    }
}

/// ESC in Game state returns to Menu.
pub fn esc_to_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<crate::AppState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(crate::AppState::Menu);
    }
}
