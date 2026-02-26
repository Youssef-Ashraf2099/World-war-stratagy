//! Tick pipeline orchestration
//!
//! Executes subsystems in strict order each tick to ensure determinism.

use bevy_ecs::prelude::*;
use tracing::{info, debug};

use super::world::WorldState;
use crate::subsystems::*;

/// Tick execution pipeline
pub struct TickPipeline {
    phases: Vec<Box<dyn TickPhase>>,
}

/// A phase in the tick pipeline
pub trait TickPhase: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&mut self, world: &mut World);
}

impl TickPipeline {
    /// Create a new tick pipeline with default v0.1 phases
    pub fn new_v0_1() -> Self {
        let phases: Vec<Box<dyn TickPhase>> = vec![
            Box::new(demographic::DemographicPhase::new()),
            Box::new(economic::EconomicPhase::new()),
        ];

        Self { phases }
    }

    /// Execute all phases for one tick
    pub fn execute(&mut self, world_state: &mut WorldState) {
        let tick = world_state.current_tick();
        debug!(tick = tick, "Executing tick pipeline");

        for phase in &mut self.phases {
            debug!(phase = phase.name(), "Executing phase");
            phase.execute(&mut world_state.world);
        }

        world_state.advance_tick();
        
        if tick % 100 == 0 {
            info!(tick = tick, "Tick milestone reached");
        }
    }

    /// Execute multiple ticks
    pub fn execute_many(&mut self, world_state: &mut WorldState, count: u64) {
        for _ in 0..count {
            self.execute(world_state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Population, Resources, Province, ResourceType, OwnedBy};
    use glam::Vec2;

    #[test]
    fn test_pipeline_execution() {
        let mut world_state = WorldState::new(42);
        let mut pipeline = TickPipeline::new_v0_1();

        assert_eq!(world_state.current_tick(), 0);
        
        pipeline.execute(&mut world_state);
        assert_eq!(world_state.current_tick(), 1);
        
        pipeline.execute_many(&mut world_state, 10);
        assert_eq!(world_state.current_tick(), 11);
    }

    /// V0.1 Exit Criteria: Run 500 ticks without instability
    #[test]
    fn test_500_tick_stability() {
        let mut world_state = WorldState::new(42);
        let mut pipeline = TickPipeline::new_v0_1();

        // Create a nation and some provinces with initial state
        let nation = world_state.spawn_nation(
            "Test Nation".to_string(),
            [255, 0, 0],
            false,
        );
        let nation_id = world_state.world.get::<crate::core::types::Nation>(nation)
            .unwrap()
            .id;

        // Spawn test provinces with well-balanced initial conditions
        for i in 0..5 {
            let _province_entity = world_state.world.spawn((
                Province {
                    id: crate::core::types::ProvinceId::new(),
                    name: format!("Province {}", i),
                    position: Vec2::new(i as f32 * 10.0, 0.0),
                    dominant_resource: match i % 3 {
                        0 => ResourceType::Food,
                        1 => ResourceType::Iron,
                        _ => ResourceType::Oil,
                    },
                },
                Population {
                    total: 1_000_000,
                    growth_rate: 0.01,
                },
                Resources {
                    food: 1000.0,
                    iron: 500.0,
                    oil: 500.0,
                    rare_earths: 100.0,
                    water: 1000.0,
                    trade_ports: 1,
                },
                crate::core::types::Infrastructure { level: 5, max_level: 10 },
                OwnedBy { nation_id },
            )).id();
        }

        // Record initial state
        let initial_pop_total = get_total_population(&mut world_state);
            let _initial_resources = get_total_resources(&mut world_state);

        // Execute 500 ticks
        pipeline.execute_many(&mut world_state, 500);

        // Verify tick count
        assert_eq!(world_state.current_tick(), 500);

        // Check for NaNs and stability
        let mut query = world_state.world.query::<(&Population, &Resources)>();
        for (pop, res) in query.iter(&world_state.world) {
            // No NaNs check
            assert!(pop.total > 0, "Population became zero or negative");
            assert!(!res.food.is_nan(), "Food resource became NaN");
            assert!(!res.iron.is_nan(), "Iron resource became NaN");
            assert!(!res.oil.is_nan(), "Oil resource became NaN");
            assert!(!res.rare_earths.is_nan(), "Rare earths became NaN");
            assert!(!res.water.is_nan(), "Water became NaN");

            // No runaway exponential growth check
            assert!(pop.total < 1_000_000_000_000, 
                "Population grew exponentially (runaway growth): {}", pop.total);
            assert!(res.food < 1_000_000.0, 
                "Food grew exponentially (runaway growth): {}", res.food);
        }

        // Verify reasonable population change (should grow but not explode)
        let final_pop_total = get_total_population(&mut world_state);
        let pop_ratio = final_pop_total as f64 / initial_pop_total as f64;
        assert!(pop_ratio > 0.5, "Population declined catastrophically: ratio={}", pop_ratio);
        assert!(pop_ratio < 10.0, "Population exploded: ratio={}", pop_ratio);

        println!("✓ 500-tick stability test passed");
        println!("  Initial population: {}", initial_pop_total);
        println!("  Final population: {}", final_pop_total);
        println!("  Growth ratio: {:.2}x", pop_ratio);
    }

    /// Helper: Get total population across all provinces
    fn get_total_population(world_state: &mut WorldState) -> u64 {
        let mut query = world_state.world.query::<&Population>();
        query.iter(&world_state.world)
            .map(|pop| pop.total)
            .sum()
    }

    /// Helper: Get total resources across all provinces
    fn get_total_resources(world_state: &mut WorldState) -> (f64, f64, f64) {
        let mut query = world_state.world.query::<&Resources>();
        query.iter(&world_state.world)
            .fold((0.0, 0.0, 0.0), |(food, iron, oil), res| {
                (food + res.food, iron + res.iron, oil + res.oil)
            })
    }

    /// Test determinism: Same seed produces same results
    #[test]
    fn test_determinism_500_ticks() {
        let seed = 42;

        // Run 1
        let mut world1 = WorldState::new(seed);
        let mut pipeline1 = TickPipeline::new_v0_1();
        setup_test_world(&mut world1);
        let hash1_before = world1.state_hash();
        pipeline1.execute_many(&mut world1, 500);
        let hash1_after = world1.state_hash();

        // Run 2 with same seed
        let mut world2 = WorldState::new(seed);
        let mut pipeline2 = TickPipeline::new_v0_1();
        setup_test_world(&mut world2);
        let hash2_before = world2.state_hash();
        pipeline2.execute_many(&mut world2, 500);
        let hash2_after = world2.state_hash();

        // Initial states should match
        assert_eq!(hash1_before, hash2_before, "Initial states differ");
        
        // Final states should match (determinism)
        assert_eq!(hash1_after, hash2_after, "Simulation is not deterministic");
        
        println!("✓ Determinism verified over 500 ticks");
    }

    fn setup_test_world(world_state: &mut WorldState) {
        let nation = world_state.spawn_nation("Test".to_string(), [255, 0, 0], false);
        let nation_id = world_state.world.get::<crate::core::types::Nation>(nation).unwrap().id;
        
        world_state.spawn_province(
            "Test Province".to_string(),
            Vec2::new(0.0, 0.0),
            ResourceType::Food,
            nation_id,
        );
    }
}
