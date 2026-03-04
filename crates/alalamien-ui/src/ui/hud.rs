use bevy::prelude::*;
use crate::systems::ui_manager::ScreenUI;
use crate::systems::game_clock::{DateLabel, GameClock};
use crate::map::picking::{NationMesh, SelectedNation};
use crate::api::SimState;
use crate::resources::PlayerNation;

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

/// GDP stat label in the left nation panel.
#[derive(Component)]
pub struct NationGdpLabel;

/// Legitimacy stat label in the left nation panel.
#[derive(Component)]
pub struct NationLegitimacyLabel;

/// Population stat label in the left nation panel.
#[derive(Component)]
pub struct NationPopLabel;

/// Stability stat label in the left nation panel.
#[derive(Component)]
pub struct NationStabilityLabel;

/// Bottom-bar API connection status text.
#[derive(Component)]
pub struct ApiStatusLabel;

/// Top-bar title label (shows player nation name once one is chosen).
#[derive(Component)]
pub struct PlayerNationLabel;

/// Left panel header (shows "SELECT A NATION" / "YOUR NATION").
#[derive(Component)]
pub struct PanelHeaderLabel;

/// The "PLAY AS" button node in the left panel.
#[derive(Component)]
pub struct PlayAsButton;

/// Text inside the "PLAY AS" button.
#[derive(Component)]
pub struct PlayAsButtonText;

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
                // --- Left side: game title / player nation ---
                bar.spawn((
                    TextBundle::from_section(
                        "ALALAMIEN: WORLD WAR STRATEGY",
                        TextStyle {
                            font_size: 14.0,
                            color: TEXT_ACCENT,
                            ..default()
                        },
                    ),
                    PlayerNationLabel,
                    Name::new("TopBarTitle"),
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
                    panel.spawn((
                        TextBundle::from_section(
                            "SELECT A NATION",
                            TextStyle {
                                font_size: 11.0,
                                color: TEXT_DIM,
                                ..default()
                            },
                        ),
                        PanelHeaderLabel,
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

                    // Live stat labels — updated each tick by update_nation_panel
                    panel.spawn((
                        TextBundle::from_section(
                            "GDP: —",
                            TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() },
                        ),
                        NationGdpLabel,
                    ));
                    panel.spawn((
                        TextBundle::from_section(
                            "Legitimacy: —",
                            TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() },
                        ),
                        NationLegitimacyLabel,
                    ));
                    panel.spawn((
                        TextBundle::from_section(
                            "Population: —",
                            TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() },
                        ),
                        NationPopLabel,
                    ));
                    panel.spawn((
                        TextBundle::from_section(
                            "Stability: —",
                            TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() },
                        ),
                        NationStabilityLabel,
                    ));

                    // Second divider
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

                    // PLAY AS button
                    panel.spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgba(0.0, 0.3, 0.5, 0.25)),
                            border_color: BorderColor(Color::srgba(0.15, 0.80, 1.0, 0.35)),
                            ..default()
                        },
                        PlayAsButton,
                        Name::new("PlayAsButton"),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            TextBundle::from_section(
                                "Click a nation first",
                                TextStyle { font_size: 11.0, color: TEXT_DIM, ..default() },
                            ),
                            PlayAsButtonText,
                        ));
                    });

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

                // Centre: API connection status
                bar.spawn((
                    TextBundle::from_section(
                        "API: connecting…",
                        TextStyle {
                            font_size: 12.0,
                            color: TEXT_DIM,
                            ..default()
                        },
                    ),
                    ApiStatusLabel,
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
/// Reads GDP and Legitimacy directly from the embedded engine WorldState.
/// Reads Population from the `NationMesh` Bevy component.
pub fn update_nation_panel(
    selected: Res<SelectedNation>,
    sim: Res<SimState>,
    nation_meshes: Query<&NationMesh>,
    mut name_q: Query<
        &mut Text,
        (With<NationNameLabel>, Without<NationIsoLabel>, Without<NationGdpLabel>, Without<NationLegitimacyLabel>, Without<NationPopLabel>, Without<NationStabilityLabel>),
    >,
    mut iso_q: Query<
        &mut Text,
        (With<NationIsoLabel>, Without<NationNameLabel>, Without<NationGdpLabel>, Without<NationLegitimacyLabel>, Without<NationPopLabel>, Without<NationStabilityLabel>),
    >,
    mut gdp_q: Query<
        &mut Text,
        (With<NationGdpLabel>, Without<NationNameLabel>, Without<NationIsoLabel>, Without<NationLegitimacyLabel>, Without<NationPopLabel>, Without<NationStabilityLabel>),
    >,
    mut leg_q: Query<
        &mut Text,
        (With<NationLegitimacyLabel>, Without<NationNameLabel>, Without<NationIsoLabel>, Without<NationGdpLabel>, Without<NationPopLabel>, Without<NationStabilityLabel>),
    >,
    mut pop_q: Query<
        &mut Text,
        (With<NationPopLabel>, Without<NationNameLabel>, Without<NationIsoLabel>, Without<NationGdpLabel>, Without<NationLegitimacyLabel>, Without<NationStabilityLabel>),
    >,
    mut stab_q: Query<
        &mut Text,
        (With<NationStabilityLabel>, Without<NationNameLabel>, Without<NationIsoLabel>, Without<NationGdpLabel>, Without<NationLegitimacyLabel>, Without<NationPopLabel>),
    >,
) {
    if !selected.is_changed() { return; }

    let nation_name = selected.name.clone().unwrap_or_else(|| "—".to_string());
    let iso = selected.iso_a3.clone().unwrap_or_default();

    for mut text in &mut name_q {
        if let Some(s) = text.sections.first_mut() { s.value = nation_name.clone(); }
    }
    for mut text in &mut iso_q {
        if let Some(s) = text.sections.first_mut() { s.value = iso.clone(); }
    }

    // Read live stats from embedded engine (direct memory read — no HTTP)
    let (gdp_text, leg_text, leg_value) = read_nation_stats(&sim, &nation_name);

    for mut text in &mut gdp_q {
        if let Some(s) = text.sections.first_mut() { s.value = gdp_text.clone(); }
    }
    for mut text in &mut leg_q {
        if let Some(s) = text.sections.first_mut() { s.value = leg_text.clone(); }
    }

    // Population — read from NationMesh (seeded from nation_seed_data at spawn time)
    let pop_text = if iso.is_empty() {
        "Population: —".to_string()
    } else {
        let pop = nation_meshes
            .iter()
            .find(|m| m.iso_a3 == iso)
            .map(|m| m.population)
            .unwrap_or(0);
        if pop == 0 {
            "Population: —".to_string()
        } else if pop >= 1_000_000 {
            format!("Population: {:.1}M", pop as f64 / 1_000_000.0)
        } else {
            format!("Population: {pop}")
        }
    };
    for mut text in &mut pop_q {
        if let Some(s) = text.sections.first_mut() { s.value = pop_text.clone(); }
    }

    // Stability — derive from legitimacy value
    let stab_text = if iso.is_empty() {
        "Stability: —".to_string()
    } else if leg_value < 0.0 {
        "Stability: —".to_string()
    } else if leg_value >= 70.0 {
        "Stability: Stable".to_string()
    } else if leg_value >= 40.0 {
        "Stability: Unstable".to_string()
    } else {
        "Stability: Critical".to_string()
    };
    for mut text in &mut stab_q {
        if let Some(s) = text.sections.first_mut() { s.value = stab_text.clone(); }
    }
}

/// Read a nation's GDP and legitimacy from the engine WorldState by name.
/// Returns (gdp_string, legitimacy_string, raw_legitimacy_value).
/// Raw value is -1.0 when no nation was found.
fn read_nation_stats(sim: &SimState, target_name: &str) -> (String, String, f64) {
    use alalamien_engine::core::types::{Nation, Legitimacy, GDP};

    let Ok(mut world_guard) = sim.0.world.write() else {
        return ("GDP: —".to_string(), "Legitimacy: —".to_string(), -1.0);
    };
    let target_lower = target_name.to_lowercase();

    let mut query = world_guard.world.query::<(&Nation, &Legitimacy, &GDP)>();
    for (nation, legitimacy, gdp) in query.iter(&world_guard.world) {
        if nation.name.to_lowercase() == target_lower {
            let gdp_str = if gdp.value >= 1_000_000_000.0 {
                format!("GDP: {:.1}B", gdp.value / 1_000_000_000.0)
            } else if gdp.value >= 1_000_000.0 {
                format!("GDP: {:.1}M", gdp.value / 1_000_000.0)
            } else {
                format!("GDP: {:.0}", gdp.value)
            };
            let leg_str = format!("Legitimacy: {:.1}%", legitimacy.value);
            return (gdp_str, leg_str, legitimacy.value);
        }
    }

    ("GDP: —".to_string(), "Legitimacy: —".to_string(), -1.0)
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

/// Update the bottom-bar API status label to show engine tick count.
/// Reads directly from the embedded WorldState — no HTTP.
pub fn update_api_status(
    sim: Res<SimState>,
    mut query: Query<&mut Text, With<ApiStatusLabel>>,
) {
    let tick = sim.0.world.read()
        .map(|w| w.current_tick())
        .unwrap_or(0);

    // Update every frame only when visible text would change
    // (tick only changes when the clock advances, so we compare the value)
    for mut text in &mut query {
        if let Some(s) = text.sections.first_mut() {
            let new_label = format!("✓ Engine tick {tick}");
            if s.value != new_label { s.value = new_label; }
        }
    }
}
/// Visual hover feedback on the PLAY AS button.
pub fn play_as_button_hover(
    mut button_q: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<PlayAsButton>),
    >,
) {
    for (interaction, mut bg, mut border) in &mut button_q {
        match interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.0, 0.55, 0.75, 0.70));
                *border = BorderColor(Color::srgb(0.15, 0.90, 1.0));
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.0, 0.40, 0.60, 0.45));
                *border = BorderColor(Color::srgba(0.15, 0.85, 1.0, 0.80));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.0, 0.30, 0.50, 0.25));
                *border = BorderColor(Color::srgba(0.15, 0.80, 1.0, 0.35));
            }
        }
    }
}

/// When PlayerNation first becomes Some, swap out the selection HUD for the
/// full playing HUD. Uses a `Local<bool>` so it only fires once.
pub fn swap_to_playing_hud(
    player_nation: Res<PlayerNation>,
    mut was_playing: Local<bool>,
    ui_entities: Query<Entity, With<ScreenUI>>,
    mut commands: Commands,
) {
    let now_playing = player_nation.name.is_some();
    if now_playing && !*was_playing {
        *was_playing = true;
        // Despawn the selection HUD
        for entity in &ui_entities {
            commands.entity(entity).despawn_recursive();
        }
        // Spawn the game HUD
        crate::ui::playing_hud::spawn_playing_hud(commands);
    }
}

// ---------------------------------------------------------------------------
// Player nation systems
// ---------------------------------------------------------------------------

/// Handle clicks on the PLAY AS button.
/// Copies the currently selected nation into [`PlayerNation`].
pub fn play_as_button_system(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<PlayAsButton>)>,
    selected: Res<SelectedNation>,
    mut player_nation: ResMut<PlayerNation>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            if selected.iso_a3.is_some() {
                player_nation.iso_a3 = selected.iso_a3.clone();
                player_nation.name = selected.name.clone();
            }
        }
    }
}

/// Update the PLAY AS button text whenever the selected nation changes.
pub fn update_play_as_button(
    selected: Res<SelectedNation>,
    player_nation: Res<PlayerNation>,
    mut text_q: Query<&mut Text, With<PlayAsButtonText>>,
) {
    if !selected.is_changed() && !player_nation.is_changed() {
        return;
    }
    let label = match &selected.name {
        Some(name) => {
            if player_nation.name.as_deref() == Some(name.as_str()) {
                format!("{} Playing as {name}", '\u{2713}')
            } else {
                format!("{} PLAY AS {name}", '\u{25ba}')
            }
        }
        None => "Click a nation first".to_string(),
    };
    let color = if selected.iso_a3.is_some() {
        Color::srgb(0.15, 0.90, 1.0)
    } else {
        Color::srgb(0.40, 0.40, 0.50)
    };
    for mut text in &mut text_q {
        if let Some(s) = text.sections.first_mut() {
            s.value = label.clone();
            s.style.color = color;
        }
    }
}

/// Update the top-bar title and panel header whenever the player nation changes.
pub fn update_player_nation_label(
    player_nation: Res<PlayerNation>,
    mut title_q: Query<&mut Text, (With<PlayerNationLabel>, Without<PanelHeaderLabel>)>,
    mut header_q: Query<&mut Text, (With<PanelHeaderLabel>, Without<PlayerNationLabel>)>,
) {
    if !player_nation.is_changed() {
        return;
    }
    let title = match &player_nation.name {
        Some(name) => format!("🏛 {name}  |  ALALAMIEN"),
        None => "ALALAMIEN: WORLD WAR STRATEGY".to_string(),
    };
    let header = match &player_nation.name {
        Some(_) => "YOUR NATION".to_string(),
        None => "SELECT A NATION".to_string(),
    };
    for mut text in &mut title_q {
        if let Some(s) = text.sections.first_mut() {
            s.value = title.clone();
        }
    }
    for mut text in &mut header_q {
        if let Some(s) = text.sections.first_mut() {
            s.value = header.clone();
        }
    }
}