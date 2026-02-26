//! Demographic subsystem
//!
//! Handles population growth, death rates, and migration.

use bevy_ecs::prelude::*;
use tracing::trace;

use crate::core::types::{Population, Resources};
use crate::core::tick::TickPhase;

/// Demographic simulation phase
pub struct DemographicPhase {
    config: DemographicConfig,
}

#[derive(Debug, Clone)]
pub struct DemographicConfig {
    /// Base growth rate per tick
    pub base_growth_rate: f64,
    /// Food required per 1000 people
    pub food_per_capita: f64,
}

impl Default for DemographicConfig {
    fn default() -> Self {
        Self {
            base_growth_rate: 0.0001, // 0.01% per tick
            food_per_capita: 0.1,
        }
    }
}

impl DemographicPhase {
    pub fn new() -> Self {
        Self {
            config: DemographicConfig::default(),
        }
    }

    pub fn with_config(config: DemographicConfig) -> Self {
        Self { config }
    }
}

impl TickPhase for DemographicPhase {
    fn name(&self) -> &str {
        "Demographic"
    }

    fn execute(&mut self, world: &mut World) {
        // Query all provinces with population and resources
        let mut query = world.query::<(&mut Population, &Resources)>();

        for (mut population, resources) in query.iter_mut(world) {
            let food_surplus = calculate_food_surplus(
                resources.food,
                population.total,
                self.config.food_per_capita,
            );

            // Population growth based on food availability
            let growth_modifier = if food_surplus > 0.0 {
                1.0 + (food_surplus * 0.001).min(0.5)
            } else {
                1.0 + (food_surplus * 0.002).max(-0.1) // Negative growth if starving
            };

            let growth = (population.total as f64 * self.config.base_growth_rate * growth_modifier) as i64;
            population.total = (population.total as i64 + growth).max(0) as u64;

            trace!(
                pop = population.total,
                growth = growth,
                food_surplus = food_surplus,
                "Population updated"
            );
        }
    }
}

/// Calculate food surplus/deficit
fn calculate_food_surplus(available_food: f64, population: u64, food_per_capita: f64) -> f64 {
    let required_food = (population as f64 / 1000.0) * food_per_capita;
    available_food - required_food
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Province, ProvinceId, ResourceType};
    use glam::Vec2;

    #[test]
    fn test_food_surplus_calculation() {
        let surplus = calculate_food_surplus(100.0, 1_000_000, 0.1);
        assert_eq!(surplus, 0.0); // Exactly balanced

        let surplus = calculate_food_surplus(150.0, 1_000_000, 0.1);
        assert!(surplus > 0.0); // Food surplus

        let surplus = calculate_food_surplus(50.0, 1_000_000, 0.1);
        assert!(surplus < 0.0); // Food deficit
    }

    #[test]
    fn test_population_growth() {
        let mut world = World::default();
        
        // Spawn a province with resources
        let mut resources = Resources::default();
        resources.food = 150.0; // Surplus
        
        world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "TestProvince".to_string(),
                position: Vec2::ZERO,
                dominant_resource: ResourceType::Food,
            },
            Population {
                total: 1_000_000,
                growth_rate: 0.01,
            },
            resources,
        ));

        let mut phase = DemographicPhase::new();
        
        // Get initial population
        let initial_pop = world.query::<&Population>()
            .iter(&world)
            .next()
            .unwrap()
            .total;

        // Execute phase
        phase.execute(&mut world);

        // Population should grow
        let final_pop = world.query::<&Population>()
            .iter(&world)
            .next()
            .unwrap()
            .total;

        assert!(final_pop > initial_pop);
    }
}
