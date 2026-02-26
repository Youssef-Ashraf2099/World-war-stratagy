//! API Server Entry Point

use alalamien_api::{run_server, ApiState};
use tracing::info;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    info!("Starting Alalamien War API Server v{}", alalamien_engine::VERSION);

    // Create API state
    let api_state = ApiState::new();

    // Try to load from geodata
    let geodata_paths = [
        "src/game/scenarios/nations.json",
        "../../src/game/scenarios/nations.json",
        "../../../src/game/scenarios/nations.json",
    ];

    let mut loaded_geodata = false;
    for path in &geodata_paths {
        if Path::new(path).exists() {
            match api_state.init_from_geodata(Path::new(path)) {
                Ok(_) => {
                    info!("Loaded world from geodata: {}", path);
                    loaded_geodata = true;
                    break;
                }
                Err(e) => {
                    eprintln!("Failed to load geodata from {}: {}", path, e);
                }
            }
        }
    }

    // Fallback to test scenario
    if !loaded_geodata {
        info!("Geodata not found, using test scenario");
        api_state.init_test_scenario();
    }

    // Start server
    run_server(api_state, 3000).await?;

    Ok(())
}
