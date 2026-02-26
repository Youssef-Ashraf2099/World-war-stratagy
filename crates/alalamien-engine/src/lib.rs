//! # Alalamien War - Simulation Engine
//!
//! A deterministic geopolitical simulation engine built with bevy_ecs.
//!
//! ## Architecture
//!
//! - **core**: Foundation types and state management
//! - **subsystems**: Game systems (economy, warfare, diplomacy)
//! - **instrumentation**: Debugging and monitoring
//! - **utils**: Shared utilities

pub mod core;
pub mod subsystems;
pub mod instrumentation;
pub mod utils;
pub mod game;

pub use core::{
    types::{Nation, Province, ResourceType, Resources},
    world::WorldState,
    tick::TickPipeline,
};
pub use game::NationData;

/// Engine version following semantic versioning
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    /// Random seed for deterministic simulation
    pub seed: u64,
    /// Target ticks per second
    pub tps: u32,
    /// Enable debug logging
    pub debug_mode: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            tps: 60,
            debug_mode: true,
        }
    }
}
