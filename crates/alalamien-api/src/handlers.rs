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
    date_time: String,
    nation_count: usize,
    province_count: usize,
    game_clock: GameClockResponse,
}

#[derive(Debug, Serialize)]
pub struct GameClockResponse {
    start_year: i32,
    start_month: u32,
    start_day: u32,
    hours_per_tick: u32,
    speed: String,
    speed_ticks_per_step: u64,
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
        date_time: world.current_datetime_string(),
        nation_count: world.nation_count(),
        province_count: world.province_count(),
        game_clock: GameClockResponse {
            start_year: world.game_clock.start_year,
            start_month: world.game_clock.start_month,
            start_day: world.game_clock.start_day,
            hours_per_tick: world.game_clock.hours_per_tick,
            speed: world.game_clock.speed.as_str().to_string(),
            speed_ticks_per_step: world.speed_ticks_per_step(),
        },
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
    let ticks = req.ticks.unwrap_or_else(|| {
        let default_ticks = state
            .world
            .read()
            .map(|world| world.speed_ticks_per_step())
            .unwrap_or(1);
        if default_ticks == 0 { 1 } else { default_ticks }
    });

    let mut world = state.world.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut pipeline = state.pipeline.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Advancing simulation by {} ticks", ticks);
    pipeline.execute_many(&mut world, ticks);

    Ok(Json(json!({
        "current_tick": world.current_tick(),
        "date_time": world.current_datetime_string(),
        "speed": world.game_clock.speed.as_str(),
        "ticks_advanced": ticks
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateClockRequest {
    start_year: Option<i32>,
    start_month: Option<u32>,
    start_day: Option<u32>,
    hours_per_tick: Option<u32>,
    speed: Option<String>,
}

pub async fn update_clock(
    AxumState(state): AxumState<ApiState>,
    Json(req): Json<UpdateClockRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut world = state.world.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(hours_per_tick) = req.hours_per_tick {
        world.set_hours_per_tick(hours_per_tick);
    }

    if let Some(speed) = req.speed {
        let speed = parse_game_speed(&speed).ok_or(StatusCode::BAD_REQUEST)?;
        world.set_game_speed(speed);
    }

    if req.start_year.is_some() || req.start_month.is_some() || req.start_day.is_some() {
        let year = req.start_year.unwrap_or(world.game_clock.start_year);
        let month = req.start_month.unwrap_or(world.game_clock.start_month);
        let day = req.start_day.unwrap_or(world.game_clock.start_day);

        world
            .set_start_date(year, month, day)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    }

    Ok(Json(json!({
        "tick": world.current_tick(),
        "date_time": world.current_datetime_string(),
        "game_clock": {
            "start_year": world.game_clock.start_year,
            "start_month": world.game_clock.start_month,
            "start_day": world.game_clock.start_day,
            "hours_per_tick": world.game_clock.hours_per_tick,
            "speed": world.game_clock.speed.as_str(),
            "speed_ticks_per_step": world.speed_ticks_per_step()
        }
    })))
}

/// Get current game clock state
pub async fn get_clock(
    AxumState(state): AxumState<ApiState>,
) -> Result<Json<Value>, StatusCode> {
    let world = state.world.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "tick": world.current_tick(),
        "date_time": world.current_datetime_string(),
        "game_clock": {
            "start_year": world.game_clock.start_year,
            "start_month": world.game_clock.start_month,
            "start_day": world.game_clock.start_day,
            "hours_per_tick": world.game_clock.hours_per_tick,
            "speed": world.game_clock.speed.as_str(),
            "speed_ticks_per_step": world.speed_ticks_per_step()
        }
    })))
}

fn parse_game_speed(value: &str) -> Option<alalamien_engine::core::world::GameSpeed> {
    use alalamien_engine::core::world::GameSpeed;

    match value.to_ascii_lowercase().as_str() {
        "paused" | "pause" | "0" => Some(GameSpeed::Paused),
        "slow" | "1" => Some(GameSpeed::Slow),
        "normal" | "2" => Some(GameSpeed::Normal),
        "fast" | "3" => Some(GameSpeed::Fast),
        "very_fast" | "veryfast" | "4" => Some(GameSpeed::VeryFast),
        _ => None,
    }
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
