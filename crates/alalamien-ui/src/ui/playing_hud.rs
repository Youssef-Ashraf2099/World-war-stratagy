use bevy::prelude::*;
use crate::systems::ui_manager::ScreenUI;
use crate::systems::game_clock::DateLabel;
use crate::api::SimState;
use crate::resources::PlayerNation;
use crate::map::picking::{NationMesh, SelectedNation};

// ---------------------------------------------------------------------------
// Colour palette (matches selection HUD)
// ---------------------------------------------------------------------------
const HUD_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.92);
const HUD_BORDER: Color = Color::srgba(0.0, 0.75, 1.0, 0.25);
const PANEL_BG: Color = Color::srgba(0.05, 0.08, 0.15, 0.97);
const TEXT_PRIMARY: Color = Color::srgb(0.90, 0.90, 0.95);
const TEXT_DIM: Color = Color::srgb(0.55, 0.55, 0.65);
const TEXT_ACCENT: Color = Color::srgb(0.0, 0.83, 1.0);
const TEXT_GREEN: Color = Color::srgb(0.30, 0.90, 0.45);
const TEXT_RED: Color = Color::srgb(0.95, 0.35, 0.35);
const TEXT_YELLOW: Color = Color::srgb(1.0, 0.85, 0.2);

// ---------------------------------------------------------------------------
// Marker components — top resource bar
// ---------------------------------------------------------------------------

#[derive(Component)] pub struct PlayingRoot;
#[derive(Component)] pub struct TopBarNationName;
#[derive(Component)] pub struct TopBarGdp;
#[derive(Component)] pub struct TopBarPop;
#[derive(Component)] pub struct TopBarLeg;
#[derive(Component)] pub struct TopBarStab;

// ---------------------------------------------------------------------------
// Marker components — left player panel
// ---------------------------------------------------------------------------

#[derive(Component)] pub struct LeftPanelGdp;
#[derive(Component)] pub struct LeftPanelPop;
#[derive(Component)] pub struct LeftPanelLeg;
#[derive(Component)] pub struct LeftPanelStab;
#[derive(Component)] pub struct LeftPanelGrowth;

// ---------------------------------------------------------------------------
// Marker components — right inspector panel
// ---------------------------------------------------------------------------

#[derive(Component)] pub struct InspectorPanel;
#[derive(Component)] pub struct InspectorHeader;
#[derive(Component)] pub struct InspectorName;
#[derive(Component)] pub struct InspectorIso;
#[derive(Component)] pub struct InspectorGdp;
#[derive(Component)] pub struct InspectorPop;
#[derive(Component)] pub struct InspectorLeg;
#[derive(Component)] pub struct InspectorStab;

// ---------------------------------------------------------------------------
// Marker components — bottom bar
// ---------------------------------------------------------------------------

#[derive(Component)] pub struct PlayingPauseIndicator;
#[derive(Component)] pub struct PlayingEngineStatus;

// ---------------------------------------------------------------------------
// HUD layout constants
// ---------------------------------------------------------------------------

const LEFT_W: f32 = 210.0;
const RIGHT_W: f32 = 200.0;
const TOP_H: f32 = 48.0;
const BOT_H: f32 = 44.0;

// ---------------------------------------------------------------------------
// Spawn
// ---------------------------------------------------------------------------

/// Spawn the full playing-mode HUD.
/// All nodes carry [`ScreenUI`] so `cleanup_ui` removes them on state exit.
pub fn spawn_playing_hud(mut commands: Commands) {
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
            PlayingRoot,
            Name::new("PlayingRoot"),
        ))
        .with_children(|root| {
            // ==================================================================
            // TOP RESOURCE BAR
            // ==================================================================
            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(TOP_H),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::horizontal(Val::Px(16.0)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        ..default()
                    },
                    background_color: BackgroundColor(HUD_BG),
                    border_color: BorderColor(HUD_BORDER),
                    ..default()
                },
                Name::new("TopResourceBar"),
            ))
            .with_children(|bar| {
                // --- Nation name (left) ---
                bar.spawn((
                    TextBundle::from_sections(vec![
                        TextSection::new("\u{1f3db} ", TextStyle { font_size: 14.0, color: TEXT_DIM, ..default() }),
                        TextSection::new("—", TextStyle { font_size: 14.0, color: TEXT_ACCENT, ..default() }),
                    ]),
                    TopBarNationName,
                    Name::new("TopNationName"),
                ));

                // --- Centre resource chips ---
                bar.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(28.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("ResourceChips"),
                ))
                .with_children(|chips| {
                    // GDP chip
                    chips.spawn((
                        TextBundle::from_sections(vec![
                            TextSection::new("\u{1f4b0} GDP  ", TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() }),
                            TextSection::new("—", TextStyle { font_size: 13.0, color: TEXT_GREEN, ..default() }),
                        ]),
                        TopBarGdp,
                    ));
                    // Population chip
                    chips.spawn((
                        TextBundle::from_sections(vec![
                            TextSection::new("\u{1f465} Pop  ", TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() }),
                            TextSection::new("—", TextStyle { font_size: 13.0, color: TEXT_PRIMARY, ..default() }),
                        ]),
                        TopBarPop,
                    ));
                    // Legitimacy chip
                    chips.spawn((
                        TextBundle::from_sections(vec![
                            TextSection::new("\u{26a1} Leg  ", TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() }),
                            TextSection::new("—", TextStyle { font_size: 13.0, color: TEXT_ACCENT, ..default() }),
                        ]),
                        TopBarLeg,
                    ));
                    // Stability chip
                    chips.spawn((
                        TextBundle::from_sections(vec![
                            TextSection::new("\u{1f4ca} Stab  ", TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() }),
                            TextSection::new("—", TextStyle { font_size: 13.0, color: TEXT_PRIMARY, ..default() }),
                        ]),
                        TopBarStab,
                    ));
                });

                // --- Date (right) ---
                bar.spawn((
                    TextBundle::from_section(
                        "Jan 1939 | Turn 0",
                        TextStyle { font_size: 13.0, color: TEXT_PRIMARY, ..default() },
                    ),
                    DateLabel,
                    Name::new("PlayingDate"),
                ));
            });

            // ==================================================================
            // MIDDLE ROW — left panel | map area | right inspector
            // ==================================================================
            root.spawn((
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    ..default()
                },
                Name::new("MiddleRow"),
            ))
            .with_children(|row| {
                // ---- LEFT: player nation panel ----
                spawn_left_panel(row);

                // ---- CENTRE: map placeholder (renders behind UI) ----
                row.spawn((
                    NodeBundle {
                        style: Style {
                            flex_grow: 1.0,
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::ColumnReverse,
                            align_items: AlignItems::FlexEnd,
                            justify_content: JustifyContent::FlexStart,
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        ..default()
                    },
                    Name::new("MapArea"),
                ))
                .with_children(|_area| {
                    // nothing needed — map renders in world space behind UI
                });

                // ---- RIGHT: nation inspector ----
                spawn_right_inspector(row);
            });

            // ==================================================================
            // BOTTOM BAR
            // ==================================================================
            root.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(BOT_H),
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
                // Pause / speed indicator
                bar.spawn((
                    TextBundle::from_section(
                        "\u{23f8} PAUSED",
                        TextStyle { font_size: 13.0, color: TEXT_ACCENT, ..default() },
                    ),
                    PlayingPauseIndicator,
                ));

                // Speed hint
                bar.spawn(TextBundle::from_section(
                    "|  1\u{00d7}  2\u{00d7}  4\u{00d7}",
                    TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() },
                ));

                // Engine status
                bar.spawn((
                    TextBundle::from_section(
                        "\u{2713} Engine tick 0",
                        TextStyle { font_size: 12.0, color: TEXT_DIM, ..default() },
                    ),
                    PlayingEngineStatus,
                ));

                // ESC hint
                bar.spawn(TextBundle::from_section(
                    "ESC \u{2014} menu",
                    TextStyle { font_size: 11.0, color: TEXT_DIM, ..default() },
                ));
            });
        });
}

// ---------------------------------------------------------------------------
// Sub-builders
// ---------------------------------------------------------------------------

fn spawn_left_panel(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(LEFT_W),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(7.0),
                border: UiRect::right(Val::Px(1.0)),
                ..default()
            },
            background_color: BackgroundColor(PANEL_BG),
            border_color: BorderColor(HUD_BORDER),
            ..default()
        },
        Name::new("LeftPanel"),
    ))
    .with_children(|panel| {
        // Header
        panel.spawn(TextBundle::from_section(
            "YOUR NATION",
            TextStyle { font_size: 10.0, color: TEXT_DIM, ..default() },
        ));

        // Divider
        divider(panel);

        // GDP row
        labeled_stat(panel, "\u{1f4b0} GDP", "—", TEXT_GREEN, LeftPanelGdp);
        // Population row
        labeled_stat(panel, "\u{1f465} Population", "—", TEXT_PRIMARY, LeftPanelPop);
        // Legitimacy row
        labeled_stat(panel, "\u{26a1} Legitimacy", "—", TEXT_ACCENT, LeftPanelLeg);
        // Stability row
        labeled_stat(panel, "\u{1f4ca} Stability", "—", TEXT_YELLOW, LeftPanelStab);
        // Growth rate row
        labeled_stat(panel, "\u{1f4c8} Growth", "—", TEXT_GREEN, LeftPanelGrowth);

        // Divider
        divider(panel);

        // Controls hint
        panel.spawn(TextBundle::from_section(
            "CONTROLS\nWASD / Arrows \u{2014} pan\nScroll \u{2014} zoom\nRMB drag \u{2014} pan\nF \u{2014} fit world\n\nClick any nation\nto inspect it",
            TextStyle { font_size: 10.0, color: TEXT_DIM, ..default() },
        ));
    });
}

fn spawn_right_inspector(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(RIGHT_W),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(7.0),
                border: UiRect::left(Val::Px(1.0)),
                display: Display::None, // hidden until a nation is clicked
                ..default()
            },
            background_color: BackgroundColor(PANEL_BG),
            border_color: BorderColor(HUD_BORDER),
            ..default()
        },
        InspectorPanel,
        Name::new("InspectorPanel"),
    ))
    .with_children(|panel| {
        // Header
        panel.spawn((
            TextBundle::from_section(
                "NATION INTEL",
                TextStyle { font_size: 10.0, color: TEXT_DIM, ..default() },
            ),
            InspectorHeader,
        ));

        divider(panel);

        // Nation name
        panel.spawn((
            TextBundle::from_section(
                "—",
                TextStyle { font_size: 16.0, color: TEXT_PRIMARY, ..default() },
            ),
            InspectorName,
        ));

        // ISO code
        panel.spawn((
            TextBundle::from_section(
                "",
                TextStyle { font_size: 11.0, color: TEXT_ACCENT, ..default() },
            ),
            InspectorIso,
        ));

        divider(panel);

        labeled_stat(panel, "\u{1f4b0} GDP", "—", TEXT_GREEN, InspectorGdp);
        labeled_stat(panel, "\u{1f465} Pop", "—", TEXT_PRIMARY, InspectorPop);
        labeled_stat(panel, "\u{26a1} Leg", "—", TEXT_ACCENT, InspectorLeg);
        labeled_stat(panel, "\u{1f4ca} Stab", "—", TEXT_YELLOW, InspectorStab);
    });
}

fn divider(parent: &mut ChildBuilder) {
    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            margin: UiRect::vertical(Val::Px(2.0)),
            ..default()
        },
        background_color: BackgroundColor(HUD_BORDER),
        ..default()
    });
}

fn labeled_stat<M: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    initial: &str,
    value_color: Color,
    marker: M,
) {
    parent.spawn((
        TextBundle::from_sections(vec![
            TextSection::new(
                format!("{label}:  "),
                TextStyle { font_size: 11.0, color: TEXT_DIM, ..default() },
            ),
            TextSection::new(
                initial,
                TextStyle { font_size: 12.0, color: value_color, ..default() },
            ),
        ]),
        marker,
    ));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn stability_text(leg: f64) -> String {
    if leg < 0.0        { "—".to_string() }
    else if leg >= 70.0 { "Stable".to_string() }
    else if leg >= 40.0 { "Unstable".to_string() }
    else                { "Critical".to_string() }
}

/// Set section[1] value on every text in the given query.
macro_rules! set_val {
    ($q:expr, $v:expr) => {
        for mut t in $q.iter_mut() {
            if t.sections.len() > 1 { t.sections[1].value = $v.to_string(); }
        }
    };
}

// ---------------------------------------------------------------------------
// Shared helper — compute player nation stats from the engine
// ---------------------------------------------------------------------------

struct PlayerStats {
    gdp_str:    String,
    pop_str:    String,
    leg_str:    String,
    stab_str:   String,
    growth_str: String,
    name:       String,
}

fn compute_player_stats(
    player_nation: &PlayerNation,
    sim: &SimState,
    nation_meshes: &Query<&NationMesh>,
) -> Option<PlayerStats> {
    use alalamien_engine::core::types::{Nation, Legitimacy, GDP};

    let player_name = player_nation.name.as_ref()?;
    let player_iso  = player_nation.iso_a3.as_ref()?;

    let Ok(mut world_guard) = sim.0.world.write() else { return None };
    let player_lower = player_name.to_lowercase();

    let mut gdp_val: f64    = 0.0;
    let mut growth_val: f64 = 0.0;
    let mut leg_val: f64    = -1.0;
    let mut found = false;

    let mut wq = world_guard.world.query::<(&Nation, &Legitimacy, &GDP)>();
    for (nation, leg, gdp) in wq.iter(&world_guard.world) {
        if nation.name.to_lowercase() == player_lower {
            gdp_val    = gdp.value;
            growth_val = gdp.growth_rate;
            leg_val    = leg.value;
            found      = true;
            break;
        }
    }
    drop(world_guard);

    let pop = nation_meshes
        .iter()
        .find(|m| m.iso_a3 == *player_iso)
        .map(|m| m.population)
        .unwrap_or(0);

    let gdp_str = if !found { "—".to_string() }
        else if gdp_val >= 1_000_000_000.0 { format!("{:.1}B", gdp_val / 1_000_000_000.0) }
        else if gdp_val >= 1_000_000.0     { format!("{:.1}M", gdp_val / 1_000_000.0) }
        else                               { format!("{:.0}", gdp_val) };
    let pop_str    = if pop >= 1_000_000 { format!("{:.1}M", pop as f64 / 1_000_000.0) }
        else if pop > 0 { pop.to_string() } else { "—".to_string() };
    let leg_str    = if leg_val >= 0.0 { format!("{:.1}%", leg_val) } else { "—".to_string() };
    let growth_str = if found { format!("{:+.1}%", growth_val * 100.0) } else { "—".to_string() };
    let stab_str   = stability_text(leg_val);

    Some(PlayerStats {
        gdp_str, pop_str, leg_str, stab_str, growth_str,
        name: player_name.clone(),
    })
}

// ---------------------------------------------------------------------------
// System 1 — update top bar (5 text queries in one ParamSet = no B0001)
// ---------------------------------------------------------------------------

pub fn update_playing_stats(
    player_nation: Res<PlayerNation>,
    sim: Res<SimState>,
    nation_meshes: Query<&NationMesh>,
    mut top: ParamSet<(
        Query<&mut Text, With<TopBarNationName>>,
        Query<&mut Text, With<TopBarGdp>>,
        Query<&mut Text, With<TopBarPop>>,
        Query<&mut Text, With<TopBarLeg>>,
        Query<&mut Text, With<TopBarStab>>,
    )>,
) {
    let Some(s) = compute_player_stats(&player_nation, &sim, &nation_meshes) else { return };

    for mut t in top.p0().iter_mut() {
        if t.sections.len() > 1 { t.sections[1].value.clone_from(&s.name); }
    }
    set_val!(top.p1(), s.gdp_str);
    set_val!(top.p2(), s.pop_str);
    set_val!(top.p3(), s.leg_str);
    set_val!(top.p4(), s.stab_str);
}

// ---------------------------------------------------------------------------
// System 2 — update left panel (5 text queries in one ParamSet = no B0001)
// Different system → no conflict with System 1's ParamSet.
// ---------------------------------------------------------------------------

pub fn update_left_panel(
    player_nation: Res<PlayerNation>,
    sim: Res<SimState>,
    nation_meshes: Query<&NationMesh>,
    mut left: ParamSet<(
        Query<&mut Text, With<LeftPanelGdp>>,
        Query<&mut Text, With<LeftPanelPop>>,
        Query<&mut Text, With<LeftPanelLeg>>,
        Query<&mut Text, With<LeftPanelStab>>,
        Query<&mut Text, With<LeftPanelGrowth>>,
    )>,
) {
    let Some(s) = compute_player_stats(&player_nation, &sim, &nation_meshes) else { return };

    set_val!(left.p0(), s.gdp_str);
    set_val!(left.p1(), s.pop_str);
    set_val!(left.p2(), s.leg_str);
    set_val!(left.p3(), s.stab_str);
    set_val!(left.p4(), s.growth_str);
}

// ---------------------------------------------------------------------------
// Update system — nation inspector (right panel)
// ---------------------------------------------------------------------------

pub fn update_nation_inspector(
    selected: Res<SelectedNation>,
    player_nation: Res<PlayerNation>,
    sim: Res<SimState>,
    nation_meshes: Query<&NationMesh>,
    mut panel_q: Query<&mut Style, With<InspectorPanel>>,
    // All six text labels grouped in a ParamSet
    mut texts: ParamSet<(
        Query<&mut Text, With<InspectorName>>,  // p0
        Query<&mut Text, With<InspectorIso>>,   // p1
        Query<&mut Text, With<InspectorGdp>>,   // p2
        Query<&mut Text, With<InspectorPop>>,   // p3
        Query<&mut Text, With<InspectorLeg>>,   // p4
        Query<&mut Text, With<InspectorStab>>,  // p5
    )>,
) {
    if !selected.is_changed() { return; }

    use alalamien_engine::core::types::{Nation, Legitimacy, GDP};

    let show = selected.iso_a3.is_some()
        && selected.iso_a3 != player_nation.iso_a3;

    for mut style in &mut panel_q {
        style.display = if show { Display::Flex } else { Display::None };
    }

    if !show { return; }

    let iso  = selected.iso_a3.as_deref().unwrap_or("");
    let name = selected.name.as_deref().unwrap_or("—");

    // ---- Engine stats ----
    let Ok(mut world_guard) = sim.0.world.write() else { return };
    let name_lower = name.to_lowercase();
    let mut gdp_val: f64 = 0.0;
    let mut leg_val: f64 = -1.0;
    let mut found = false;

    let mut wq = world_guard.world.query::<(&Nation, &Legitimacy, &GDP)>();
    for (n, l, g) in wq.iter(&world_guard.world) {
        if n.name.to_lowercase() == name_lower {
            gdp_val = g.value; leg_val = l.value; found = true; break;
        }
    }
    drop(world_guard);

    let pop = nation_meshes
        .iter()
        .find(|m| m.iso_a3 == iso)
        .map(|m| m.population)
        .unwrap_or(0);

    let gdp_str  = if !found { "—".to_string() }
        else if gdp_val >= 1_000_000_000.0 { format!("{:.1}B", gdp_val / 1_000_000_000.0) }
        else if gdp_val >= 1_000_000.0     { format!("{:.1}M", gdp_val / 1_000_000.0) }
        else                               { format!("{:.0}", gdp_val) };
    let pop_str  = if pop >= 1_000_000 { format!("{:.1}M", pop as f64 / 1_000_000.0) }
        else if pop > 0 { pop.to_string() } else { "—".to_string() };
    let leg_str  = if leg_val >= 0.0 { format!("{:.1}%", leg_val) } else { "—".to_string() };
    let stab_str = stability_text(leg_val);

    // Write name / iso to section[0], stats to section[1]
    for mut t in texts.p0().iter_mut() {
        if !t.sections.is_empty() { t.sections[0].value = name.to_string(); }
    }
    for mut t in texts.p1().iter_mut() {
        if !t.sections.is_empty() { t.sections[0].value = iso.to_string(); }
    }
    set_val!(texts.p2(), gdp_str);
    set_val!(texts.p3(), pop_str);
    set_val!(texts.p4(), leg_str);
    set_val!(texts.p5(), stab_str);
}

// ---------------------------------------------------------------------------
// Update system — bottom bar pause indicator (mirrors selection HUD version)
// ---------------------------------------------------------------------------

pub fn update_playing_pause(
    clock: Res<crate::systems::game_clock::GameClock>,
    mut q: Query<&mut Text, With<PlayingPauseIndicator>>,
) {
    if !clock.is_changed() { return; }
    let label = if clock.paused {
        format!("\u{23f8} PAUSED  {}", clock.speed.label())
    } else {
        format!("\u{25b6} PLAYING  {}", clock.speed.label())
    };
    for mut text in &mut q {
        if let Some(s) = text.sections.first_mut() { s.value = label.clone(); }
    }
}

/// Mirror the engine tick count into the bottom bar.
pub fn update_playing_engine_status(
    sim: Res<SimState>,
    mut q: Query<&mut Text, With<PlayingEngineStatus>>,
) {
    let tick = sim.0.world.read().map(|w| w.current_tick()).unwrap_or(0);
    for mut text in &mut q {
        if let Some(s) = text.sections.first_mut() {
            let new = format!("\u{2713} Engine tick {tick}");
            if s.value != new { s.value = new; }
        }
    }
}
