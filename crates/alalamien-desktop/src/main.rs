//! Alalamien War Desktop Application
//!
//! Tauri-based desktop wrapper that runs the API server
//! and embeds the frontend.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::info;

mod embedded_server;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    info!("Starting Alalamien War Desktop v{}", alalamien_engine::VERSION);

    // Start embedded API server in background
    tokio::spawn(async {
        if let Err(e) = embedded_server::start_embedded_server().await {
            eprintln!("Failed to start embedded server: {}", e);
        }
    });

    // Build and run Tauri app
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            info!("Tauri app setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
