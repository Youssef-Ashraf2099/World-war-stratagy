//! Embedded API server for desktop app

use alalamien_api::{ApiState, run_server};
use tracing::info;
use std::path::Path;

pub async fn start_embedded_server() -> anyhow::Result<()> {
    info!("Initializing embedded API server");

    // Create API state
    let api_state = ApiState::new();

    // Try to load from geodata first
    let geodata_paths = [
        Path::new("src/game/scenarios/nations.json"),
        Path::new("../../src/game/scenarios/nations.json"),
        Path::new("../../../src/game/scenarios/nations.json"),
    ];

    let mut loaded_geodata = false;
    for path in &geodata_paths {
        if path.exists() {
            match api_state.init_from_geodata(path) {
                Ok(_) => {
                    info!("Loaded world from geodata: {}", path.display());
                    loaded_geodata = true;
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to load geodata from {}: {}", path.display(), e);
                }
            }
        }
    }

    // Fallback to test scenario if geodata not found
    if !loaded_geodata {
        info!("Geodata not found, using test scenario");
        api_state.init_test_scenario();
    }

    // Start server on localhost:3000
    run_server(api_state, 3000).await?;

    Ok(())
}
