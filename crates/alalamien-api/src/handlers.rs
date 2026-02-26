//! HTTP request handlers

use axum::{
    extract::{Path, State as AxumState},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;

use crate::state::ApiState;

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Debug, Serialize)]
pub struct WorldStateResponse {
    tick: u64,
    seed: u64,
    nation_count: usize,
    province_count: usize,
}

#[derive(Debug, Serialize)]
pub struct NationResponse {
    id: String,
    name: String,
    color: [u8; 3],
    legitimacy: f64,
    gdp: f64,
    is_player: bool,
}

#[derive(Debug, Serialize)]
pub struct ProvinceResponse {
    id: String,
    name: String,
    position: [f32; 2],
    dominant_resource: String,
    population: u64,
    owner_id: String,
    resources: ResourcesResponse,
}

#[derive(Debug, Serialize)]
pub struct ResourcesResponse {
    food: f64,
    iron: f64,
    oil: f64,
    rare_earths: f64,
    water: f64,
    trade_ports: u32,
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: alalamien_engine::VERSION.to_string(),
    })
}

/// Get world state summary
pub async fn get_world_state(
    AxumState(state): AxumState<ApiState>,
) -> Result<Json<WorldStateResponse>, StatusCode> {
    let mut world = state.world.write().map_err(|e| {
        error!("Failed to acquire world lock: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(WorldStateResponse {
        tick: world.current_tick(),
        seed: world.seed,
        nation_count: world.nation_count(),
        province_count: world.province_count(),
    }))
}

/// Advance simulation by N ticks
#[derive(Debug, Deserialize)]
pub struct AdvanceTickRequest {
    ticks: Option<u64>,
}

pub async fn advance_tick(
    AxumState(state): AxumState<ApiState>,
    Json(req): Json<AdvanceTickRequest>,
) -> Result<Json<Value>, StatusCode> {
    let ticks = req.ticks.unwrap_or(1);

    let mut world = state.world.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut pipeline = state.pipeline.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Advancing simulation by {} ticks", ticks);
    pipeline.execute_many(&mut world, ticks);

    Ok(Json(json!({
        "current_tick": world.current_tick(),
        "ticks_advanced": ticks
    })))
}

/// Get all nations
pub async fn get_nations(
    AxumState(state): AxumState<ApiState>,
) -> Result<Json<Vec<NationResponse>>, StatusCode> {
    use alalamien_engine::core::types::*;

    let mut world = state.world.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut nations = Vec::new();
    let mut query = world.world.query::<(
        &Nation,
        &Legitimacy,
        &GDP,
        Option<&PlayerControlled>,
    )>();

    for (nation, legitimacy, gdp, player) in query.iter(&world.world) {
        nations.push(NationResponse {
            id: nation.id.0.to_string(),
            name: nation.name.clone(),
            color: nation.color,
            legitimacy: legitimacy.value,
            gdp: gdp.value,
            is_player: player.is_some(),
        });
    }

    Ok(Json(nations))
}

/// Get nation by ID
pub async fn get_nation_by_id(
    AxumState(state): AxumState<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<NationResponse>, StatusCode> {
    use alalamien_engine::core::types::*;

    let nation_uuid = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut world = state.world.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut query = world.world.query::<(
        &Nation,
        &Legitimacy,
        &GDP,
        Option<&PlayerControlled>,
    )>();

    for (nation, legitimacy, gdp, player) in query.iter(&world.world) {
        if nation.id.0 == nation_uuid {
            return Ok(Json(NationResponse {
                id: nation.id.0.to_string(),
                name: nation.name.clone(),
                color: nation.color,
                legitimacy: legitimacy.value,
                gdp: gdp.value,
                is_player: player.is_some(),
            }));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

/// Get all provinces
pub async fn get_provinces(
    AxumState(state): AxumState<ApiState>,
) -> Result<Json<Vec<ProvinceResponse>>, StatusCode> {
    use alalamien_engine::core::types::*;
    let mut world = state.world.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut provinces = Vec::new();
    let mut query = world.world.query::<(
        &Province,
        &Population,
        &Resources,
        &OwnedBy,
    )>();

    for (province, population, resources, owner) in query.iter(&world.world) {
        provinces.push(ProvinceResponse {
            id: province.id.0.to_string(),
            name: province.name.clone(),
            position: [province.position.x, province.position.y],
            dominant_resource: format!("{:?}", province.dominant_resource),
            population: population.total,
            owner_id: owner.nation_id.0.to_string(),
            resources: ResourcesResponse {
                food: resources.food,
                iron: resources.iron,
                oil: resources.oil,
                rare_earths: resources.rare_earths,
                water: resources.water,
                trade_ports: resources.trade_ports,
            },
        });
    }

    Ok(Json(provinces))
}

/// Get province by ID
pub async fn get_province_by_id(
    AxumState(state): AxumState<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ProvinceResponse>, StatusCode> {
    use alalamien_engine::core::types::*;
    let province_uuid = Uuid::parse_str(&id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut world = state.world.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut query = world.world.query::<(
        &Province,
        &Population,
        &Resources,
        &OwnedBy,
    )>();

    for (province, population, resources, owner) in query.iter(&world.world) {
        if province.id.0 == province_uuid {
            return Ok(Json(ProvinceResponse {
                id: province.id.0.to_string(),
                name: province.name.clone(),
                position: [province.position.x, province.position.y],
                dominant_resource: format!("{:?}", province.dominant_resource),
                population: population.total,
                owner_id: owner.nation_id.0.to_string(),
                resources: ResourcesResponse {
                    food: resources.food,
                    iron: resources.iron,
                    oil: resources.oil,
                    rare_earths: resources.rare_earths,
                    water: resources.water,
                    trade_ports: resources.trade_ports,
                },
            }));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

/// Get metrics snapshot
pub async fn get_metrics(
    AxumState(state): AxumState<ApiState>,
) -> Json<Value> {
    let snapshot = state.metrics.snapshot();
    Json(serde_json::to_value(snapshot).unwrap())
}
