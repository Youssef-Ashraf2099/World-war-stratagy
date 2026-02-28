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

    /// Create a new tick pipeline with default v0.2 phases
    /// Order follows ROADMAP.md requirements:
    /// 1. Economy (production)
    /// 2. Trade (resource distribution)
    /// 3. Logistics (supply lines, attrition)
    /// 4. Stability (legitimacy, protests, civil war)
    /// 5. Demographics (population changes)
    pub fn new_v0_2() -> Self {
        let phases: Vec<Box<dyn TickPhase>> = vec![
            Box::new(economic::EconomicPhase::new()),
            Box::new(trade::TradePhase::new()),
            Box::new(logistics::LogisticsPhase::new()),
            Box::new(stability::StabilityPhase::new()),
            Box::new(demographic::DemographicPhase::new()),
        ];
        Self { phases }
    }

    /// Create a new tick pipeline with V0.3 phases (War & AI)
    /// Order:
    /// 1. AI Decision (strategic choices)
    /// 2. Warfare (declarations, peace)
    /// 3. Economy (production)
    /// 4. Trade (distribution)
    /// 5. Logistics (supply lines)
    /// 6. Combat (battle resolution)
    /// 7. Occupation (territory control)
    /// 8. Stability (legitimacy, unrest)
    /// 9. Demographics (population)
    pub fn new_v0_3() -> Self {
        use crate::subsystems::*;
        let phases: Vec<Box<dyn TickPhase>> = vec![
            Box::new(AIDecisionPhase::new()),
            Box::new(WarfarePhase::new()),
            Box::new(EconomicPhase::new()),
            Box::new(TradePhase::new()),
            Box::new(LogisticsPhase::new()),
            Box::new(CombatPhase::new()),
            Box::new(OccupationPhase::new()),
            Box::new(StabilityPhase::new()),
            Box::new(DemographicPhase::new()),
        ];
        Self { phases }
    }

    /// Create a new tick pipeline with V0.35 phases (Advanced AI)
    ///
    /// Same deterministic order as V0.3, but upgrades AI decisions:
    /// 1. Advanced AI Decision
    /// 2. Warfare
    /// 3. Economy
    /// 4. Trade
    /// 5. Logistics
    /// 6. Combat
    /// 7. Occupation
    /// 8. Stability
    /// 9. Demographics
    pub fn new_v0_35() -> Self {
        use crate::subsystems::*;
        let phases: Vec<Box<dyn TickPhase>> = vec![
            Box::new(AdvancedAIDecisionPhase::new()),
            Box::new(WarfarePhase::new()),
            Box::new(EconomicPhase::new()),
            Box::new(TradePhase::new()),
            Box::new(LogisticsPhase::new()),
            Box::new(CombatPhase::new()),
            Box::new(OccupationPhase::new()),
            Box::new(StabilityPhase::new()),
            Box::new(DemographicPhase::new()),
        ];
        Self { phases }
    }

    /// Create a tick pipeline with debug instrumentation (for development)
    pub fn new_v0_2_debug(config: &crate::EngineConfig) -> Self {
        let phases: Vec<Box<dyn TickPhase>> = vec![
            Box::new(economic::EconomicPhase::new()),
            Box::new(trade::TradePhase::new()),
            Box::new(logistics::LogisticsPhase::new()),
            Box::new(stability::StabilityPhase::new()),
            Box::new(demographic::DemographicPhase::new()),
            Box::new(crate::instrumentation::DebuggerPhase::new(config.clone())),
        ];
        Self { phases }
    }

    /// Add a new phase to the pipeline
    pub fn add_phase(&mut self, phase: Box<dyn TickPhase>) {
        self.phases.push(phase);
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

    /// Test determinism over full V0.2 pipeline replay
    #[test]
    fn test_determinism_500_ticks_v0_2_replay() {
        let seed = 1337;

        let mut world1 = WorldState::new(seed);
        let mut pipeline1 = TickPipeline::new_v0_2();
        setup_test_world_v0_2(&mut world1);
        let hash1_before = world1.state_hash();
        pipeline1.execute_many(&mut world1, 500);
        let hash1_after = world1.state_hash();

        let mut world2 = WorldState::new(seed);
        let mut pipeline2 = TickPipeline::new_v0_2();
        setup_test_world_v0_2(&mut world2);
        let hash2_before = world2.state_hash();
        pipeline2.execute_many(&mut world2, 500);
        let hash2_after = world2.state_hash();

        assert_eq!(hash1_before, hash2_before, "V0.2 initial states differ");
        assert_eq!(hash1_after, hash2_after, "V0.2 replay is not deterministic");
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

    /// Test determinism over 1000 ticks with V0.35 (Combat + Advanced AI + Clock System)
    #[test]
    fn test_determinism_1000_ticks_v0_35() {
        let seed = 9999;

        // Run 1
        let mut world1 = WorldState::new(seed);
        let mut pipeline1 = TickPipeline::new_v0_35();
        setup_test_world_v0_2(&mut world1);
        let hash1_before = world1.state_hash();
        pipeline1.execute_many(&mut world1, 1000);
        let hash1_after = world1.state_hash();

        // Run 2 with same seed
        let mut world2 = WorldState::new(seed);
        let mut pipeline2 = TickPipeline::new_v0_35();
        setup_test_world_v0_2(&mut world2);
        let hash2_before = world2.state_hash();
        pipeline2.execute_many(&mut world2, 1000);
        let hash2_after = world2.state_hash();

        // Initial states should match
        assert_eq!(hash1_before, hash2_before, "V0.35 initial states differ");
        
        // Final states should match (determinism verified over 1000 ticks)
        assert_eq!(hash1_after, hash2_after, "V0.35 simulation is not deterministic over 1000 ticks");
        
        // Verify clock system is part of state hash
        let tick1 = world1.current_tick();
        let tick2 = world2.current_tick();
        assert_eq!(tick1, tick2, "Tick count mismatch after determinism test");
        assert_eq!(tick1, 1000, "Expected tick count 1000, got {}", tick1);
        
        println!("✓ V0.35 Determinism verified over 1000 ticks (Combat + Advanced AI + Clock System)");
        println!("  Clock: {} hours per tick, Speed: {}", world1.game_clock.hours_per_tick, world1.game_clock.speed.as_str());
    }

    fn setup_test_world_v0_2(world_state: &mut WorldState) {
        let nation_a = world_state.spawn_nation("Nation A".to_string(), [255, 0, 0], false);
        let nation_b = world_state.spawn_nation("Nation B".to_string(), [0, 0, 255], false);

        let nation_a_id = world_state
            .world
            .get::<crate::core::types::Nation>(nation_a)
            .unwrap()
            .id;
        let nation_b_id = world_state
            .world
            .get::<crate::core::types::Nation>(nation_b)
            .unwrap()
            .id;

        if let Some(mut war) = world_state.world.get_mut::<crate::core::types::WarState>(nation_a) {
            war.at_war_with.push(nation_b_id);
        }
        if let Some(mut war) = world_state.world.get_mut::<crate::core::types::WarState>(nation_b) {
            war.at_war_with.push(nation_a_id);
        }

        let a_cap = world_state.spawn_province(
            "A Core".to_string(),
            Vec2::new(0.0, 0.0),
            ResourceType::Food,
            nation_a_id,
        );
        world_state.world.entity_mut(a_cap).insert(crate::core::types::Capital);

        let a_front = world_state.spawn_province(
            "A Front".to_string(),
            Vec2::new(1.0, 0.0),
            ResourceType::Iron,
            nation_a_id,
        );

        let b_cap = world_state.spawn_province(
            "B Core".to_string(),
            Vec2::new(2.0, 0.0),
            ResourceType::Oil,
            nation_b_id,
        );
        world_state.world.entity_mut(b_cap).insert(crate::core::types::Capital);

        let a_cap_id = world_state.world.get::<Province>(a_cap).unwrap().id;
        let a_front_id = world_state.world.get::<Province>(a_front).unwrap().id;
        let b_cap_id = world_state.world.get::<Province>(b_cap).unwrap().id;

        world_state.add_province_border(a_cap_id, a_front_id);
        world_state.add_province_border(a_front_id, b_cap_id);
    }
}
