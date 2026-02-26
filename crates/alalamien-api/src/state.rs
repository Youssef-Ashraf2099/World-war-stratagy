//! API server state management

use alalamien_engine::{
    core::{world::WorldState, tick::TickPipeline},
    instrumentation::MetricsRegistry,
};
use std::sync::{Arc, RwLock};

/// Shared API state
#[derive(Clone)]
pub struct ApiState {
    pub world: Arc<RwLock<WorldState>>,
    pub pipeline: Arc<RwLock<TickPipeline>>,
    pub metrics: MetricsRegistry,
}

impl ApiState {
    /// Create new API state with default world
    pub fn new() -> Self {
        Self::with_seed(42)
    }

    /// Create new API state with specific seed
    pub fn with_seed(seed: u64) -> Self {
        let world = WorldState::new(seed);
        let pipeline = TickPipeline::new_v0_1();
        
        Self {
            world: Arc::new(RwLock::new(world)),
            pipeline: Arc::new(RwLock::new(pipeline)),
            metrics: MetricsRegistry::new(),
        }
    }

    /// Initialize world from Natural Earth geodata
    pub fn init_from_geodata(&self, nations_json_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let world = WorldState::from_geodata(42, nations_json_path)?;
        *self.world.write().unwrap() = world;
        Ok(())
    }

    /// Initialize world with test scenario (fallback for testing without geodata)
    pub fn init_test_scenario(&self) {
        let mut world = self.world.write().unwrap();
        
        // Create test nations
        let nation1 = world.spawn_nation(
            "Empire of Testing".to_string(),
            [255, 0, 0],
            true,
        );

        let nation2 = world.spawn_nation(
            "Republic of Debug".to_string(),
            [0, 0, 255],
            false,
        );

        // Get nation IDs for province ownership
        let nation1_id = world.world.get::<alalamien_engine::core::types::Nation>(nation1)
            .unwrap()
            .id;

        let nation2_id = world.world.get::<alalamien_engine::core::types::Nation>(nation2)
            .unwrap()
            .id;

        // Create test provinces
        world.spawn_province(
            "Capital Province".to_string(),
            glam::Vec2::new(0.0, 0.0),
            alalamien_engine::core::types::ResourceType::Food,
            nation1_id,
        );

        world.spawn_province(
            "Industrial Province".to_string(),
            glam::Vec2::new(10.0, 0.0),
            alalamien_engine::core::types::ResourceType::Iron,
            nation1_id,
        );

        world.spawn_province(
            "Border Province".to_string(),
            glam::Vec2::new(5.0, 5.0),
            alalamien_engine::core::types::ResourceType::Oil,
            nation2_id,
        );
    }
}

impl Default for ApiState {
    fn default() -> Self {
        Self::new()
    }
}
