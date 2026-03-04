use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle};

use crate::AppState;
use crate::api::SimState;
use crate::map::nation_colors::{color_for_nation, color_u8_for_nation};
use crate::map::nation_seed_data::seed_for;
use crate::map::picking::{
    nation_aabb, click_select_system, hover_system,
    HoveredNation, NationMesh, SelectedNation,
};
use crate::map::shapefile_loader::{load_countries_shp, WorldGeoData};
use crate::map::tessellator::tessellate_group;

/// Marker component for all map entities — enables clean despawn on state exit.
#[derive(Component)]
pub struct MapEntity;

/// Loading progress resource — set by the loader, polled by the Loading update.
#[derive(Resource, Default)]
pub struct MapLoadState {
    pub done: bool,
    pub nations_loaded: usize,
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<WorldGeoData>()
            .init_resource::<MapLoadState>()
            .init_resource::<HoveredNation>()
            .init_resource::<SelectedNation>()
            // Loading state — parse shapefiles once
            .add_systems(OnEnter(AppState::Loading), load_geodata_sync)
            // Game state — spawn map and seed engine nations
            .add_systems(OnEnter(AppState::Game), (spawn_map, seed_engine_from_geodata))
            .add_systems(OnExit(AppState::Game), despawn_map)
            .add_systems(
                Update,
                (hover_system, click_select_system).run_if(in_state(AppState::Game)),
            );
    }
}

// ---------------------------------------------------------------------------
// Loading state systems
// ---------------------------------------------------------------------------

/// Runs once when Loading state is entered.
/// Synchronously reads the Natural Earth shapefile and stores results in
/// [`WorldGeoData`].  The loading UI update loop will then transition to Game.
pub fn load_geodata_sync(
    mut geo_data: ResMut<WorldGeoData>,
    mut load_state: ResMut<MapLoadState>,
) {
    if load_state.done {
        return; // already loaded (re-entry guard)
    }

    // The shapefile path is relative to the workspace root (cargo run CWD).
    let shp_path = std::path::Path::new("assets/data/ne_110m_admin_0_countries.shp");

    if !shp_path.exists() {
        let err = format!("Shapefile not found at '{}'", shp_path.display());
        tracing::error!("{}", err);
        load_state.error = Some(err);
        load_state.done = true; // mark done so we transition anyway
        return;
    }

    match load_countries_shp(shp_path) {
        Ok(data) => {
            load_state.nations_loaded = data.nations.len();
            *geo_data = data;
            load_state.done = true;
            tracing::info!("Geodata loaded: {} nations", load_state.nations_loaded);
        }
        Err(e) => {
            tracing::error!("Failed to load geodata: {e}");
            load_state.error = Some(e);
            load_state.done = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Game state systems
// ---------------------------------------------------------------------------

/// Spawns the ocean background quad + one mesh entity per nation polygon group.
pub fn spawn_map(
    mut commands: Commands,
    geo_data: Res<WorldGeoData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // ------------------------------------------------------------------
    // Ocean background — a large dark-blue quad filling the world space
    // ------------------------------------------------------------------
    let ocean_mesh = Mesh::from(Rectangle::new(
        crate::map::projection::MAP_WIDTH + 64.0,
        crate::map::projection::MAP_HEIGHT + 64.0,
    ));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(ocean_mesh).into(),
            material: materials.add(ColorMaterial::from(Color::srgb(0.10, 0.16, 0.26))),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        MapEntity,
        Name::new("OceanBackground"),
    ));

    // ------------------------------------------------------------------
    // Nation polygons
    // ------------------------------------------------------------------
    if geo_data.nations.is_empty() {
        tracing::warn!("WorldGeoData is empty — no nations to render");
        return;
    }

    let mut spawned = 0usize;

    for (nation_idx, nation) in geo_data.nations.iter().enumerate() {
        let seed = seed_for(&nation.iso_a3);
        let base_color = color_for_nation(&nation.iso_a3);
        let aabb = nation_aabb(&nation.groups);

        for group in &nation.groups {
            let Some(mesh) = tessellate_group(group) else {
                continue;
            };

            let mesh_handle = meshes.add(mesh);
            let mat_handle = materials.add(ColorMaterial::from(base_color));

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: mesh_handle.into(),
                    material: mat_handle,
                    // Slight z > 0 so nations render above ocean
                    transform: Transform::from_xyz(0.0, 0.0, 1.0),
                    ..default()
                },
                NationMesh {
                    nation_index: nation_idx,
                    iso_a3: nation.iso_a3.clone(),
                    name: nation.name.clone(),
                    base_color,
                    aabb,
                    population: seed.population,
                },
                MapEntity,
                Name::new(format!("Nation_{}", nation.iso_a3)),
            ));

            spawned += 1;
        }
    }

    tracing::info!("Spawned {} nation polygon meshes", spawned);
}

/// Despawn all map entities when leaving the Game state.
fn despawn_map(
    mut commands: Commands,
    map_entities: Query<Entity, With<MapEntity>>,
) {
    for entity in &map_entities {
        commands.entity(entity).despawn_recursive();
    }
}

/// Seed the embedded engine's `WorldState` with one nation per shapefile entry.
/// Called once on `OnEnter(AppState::Game)` — after `load_geodata_sync` has
/// already populated `WorldGeoData`.  The engine then runs the full simulation
/// pipeline against these real nations every game-clock tick.
fn seed_engine_from_geodata(
    geo_data: Res<WorldGeoData>,
    sim: Res<SimState>,
) {
    let Ok(mut world) = sim.0.world.write() else {
        tracing::error!("seed_engine_from_geodata: could not lock WorldState");
        return;
    };

    // Guard: if nations are already seeded (e.g. re-entering Game state)
    // just return to avoid duplicates.
    if world.nation_count() > 0 {
        tracing::info!("Engine already has {} nations — skipping re-seed", world.nation_count());
        return;
    }

    use alalamien_engine::core::types::{Legitimacy, GDP, EconomicStress};

    let mut count = 0usize;
    for nation in &geo_data.nations {
        let color = color_u8_for_nation(&nation.iso_a3);
        let entity = world.spawn_nation(nation.name.clone(), color, false);
        let seed = seed_for(&nation.iso_a3);
        world.world.entity_mut(entity).insert(GDP {
            value: seed.gdp,
            growth_rate: seed.growth_rate,
        });
        world.world.entity_mut(entity).insert(Legitimacy::new(seed.legitimacy));
        world.world.entity_mut(entity).insert(EconomicStress {
            gdp: seed.gdp,
            current_deficit: 0.0,
            accumulated_deficit: 0.0,
        });
        count += 1;
    }

    tracing::info!("Seeded engine with {} real nations (with historical stats)", count);
}
