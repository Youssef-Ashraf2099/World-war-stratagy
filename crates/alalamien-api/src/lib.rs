//! # Alalamien War - API Server
//!
//! REST API for exposing simulation state to frontend

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod handlers;
mod state;

pub use state::ApiState;

/// Start the API server
pub async fn run_server(api_state: ApiState, port: u16) -> anyhow::Result<()> {
    let app = create_router(api_state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await?;
    
    info!("API server listening on http://127.0.0.1:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the axum router with all routes
fn create_router(api_state: ApiState) -> Router {
    // CORS layer to allow frontend access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/world/state", get(handlers::get_world_state))
        .route("/world/tick", post(handlers::advance_tick))
        .route("/world/clock", post(handlers::update_clock))
        .route("/nations", get(handlers::get_nations))
        .route("/nations/:id", get(handlers::get_nation_by_id))
        .route("/provinces", get(handlers::get_provinces))
        .route("/provinces/:id", get(handlers::get_province_by_id))
        .route("/metrics", get(handlers::get_metrics))
        .with_state(api_state)
        .layer(cors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let api_state = ApiState::new();
        let _router = create_router(api_state);
        // Router created successfully
    }
}
