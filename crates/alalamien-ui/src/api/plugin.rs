use std::sync::Arc;
use bevy::prelude::*;
use alalamien_api::ApiState;

// ---------------------------------------------------------------------------
// Bevy resource — shared world simulation state
// ---------------------------------------------------------------------------

/// Wraps the engine's `ApiState` so Bevy systems can read world data directly
/// without any HTTP round-trips.  All fields inside are `Arc<RwLock<…>>` so
/// acquiring a read lock is always safe from any Bevy system.
#[derive(Resource, Clone)]
pub struct SimState(pub Arc<ApiState>);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Initialises the embedded simulation:
///  1. Creates `ApiState` (wraps `WorldState` + `TickPipeline`)
///  2. Seeds it with the test scenario
///  3. Spawns the axum REST server on port 3030 in a background thread
///     (useful for debugging; the UI reads state directly — no HTTP needed)
///  4. Registers `SimState` as a Bevy resource
///  5. Wires the tick-advance system
pub struct ApiPlugin;

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        // ── 1. Create shared simulation state ──────────────────────────────
        // Nations are seeded later by seed_engine_from_geodata (OnEnter Game)
        // so we start with a clean world — no test scenario.
        let api_state = Arc::new(ApiState::new());

        // ── 2. Embed the axum REST server in a background OS thread ────────
        //      Uses its own single-threaded Tokio runtime so it never
        //      conflicts with Bevy's executor.
        let server_state = Arc::clone(&api_state);
        std::thread::Builder::new()
            .name("api-server".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("tokio rt");
                rt.block_on(async move {
                    if let Err(e) = alalamien_api::run_server(
                        (*server_state).clone(),
                        3030,
                    ).await {
                        eprintln!("[api-server] error: {e}");
                    }
                });
            })
            .expect("failed to spawn api-server thread");

        // ── 3. Register resource + systems ─────────────────────────────────
        app.insert_resource(SimState(api_state))
            .add_systems(
                Update,
                advance_simulation_tick.run_if(in_state(crate::AppState::Game)),
            );
    }
}

// ---------------------------------------------------------------------------
// System: advance the engine pipeline each time the UI clock fires a tick
// ---------------------------------------------------------------------------

pub fn advance_simulation_tick(
    mut evr: EventReader<crate::systems::game_clock::GameTickFired>,
    sim: Res<SimState>,
) {
    let count = evr.read().count();
    if count == 0 { return; }

    let Ok(mut world)    = sim.0.world.write()    else { return };
    let Ok(mut pipeline) = sim.0.pipeline.write() else { return };
    pipeline.execute_many(&mut world, count as u64);
}
