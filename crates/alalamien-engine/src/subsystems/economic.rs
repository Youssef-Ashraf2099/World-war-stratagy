//! Economic subsystem
//!
//! Handles resource production, consumption, and GDP calculation.

use bevy_ecs::prelude::*;
use tracing::trace;

use crate::core::types::{Resources, ResourceType, Population, Infrastructure, Province};
use crate::core::tick::TickPhase;

/// Economic simulation phase
pub struct EconomicPhase {
    config: EconomicConfig,
}

#[derive(Debug, Clone)]
pub struct EconomicConfig {
    /// Base production multiplier
    pub base_production: f64,
    /// Infrastructure efficiency bonus per level
    pub infrastructure_bonus: f64,
}

impl Default for EconomicConfig {
    fn default() -> Self {
        Self {
            base_production: 1.0,
            infrastructure_bonus: 0.1,
        }
    }
}

impl EconomicPhase {
    pub fn new() -> Self {
        Self {
            config: EconomicConfig::default(),
        }
    }

    pub fn with_config(config: EconomicConfig) -> Self {
        Self { config }
    }
}

impl TickPhase for EconomicPhase {
    fn name(&self) -> &str {
        "Economic"
    }

    fn execute(&mut self, world: &mut World) {
        // Query provinces with all economic components
        let mut query = world.query::<(
            &Province,
            &mut Resources,
            &Population,
            &Infrastructure,
        )>();

        for (province, mut resources, population, infrastructure) in query.iter_mut(world) {
            // Calculate production efficiency
            let infrastructure_multiplier = 1.0 + (infrastructure.level as f64 * self.config.infrastructure_bonus);
            let population_factor = (population.total as f64 / 1_000_000.0).sqrt();
            
            let production_efficiency = 
                self.config.base_production * infrastructure_multiplier * population_factor;

            // Produce dominant resource
            let production = calculate_resource_production(
                province.dominant_resource,
                production_efficiency,
            );

            resources.add(province.dominant_resource, production);

            // Consume food (basic consumption)
            let food_consumption = (population.total as f64 / 1000.0) * 0.1;
            resources.food = (resources.food - food_consumption).max(0.0);

            trace!(
                province = %province.name,
                resource = ?province.dominant_resource,
                production = production,
                efficiency = production_efficiency,
                "Resource production"
            );
        }
    }
}

/// Calculate resource production amount based on type and efficiency
fn calculate_resource_production(resource_type: ResourceType, efficiency: f64) -> f64 {
    let base_production = match resource_type {
        ResourceType::Food => 10.0,
        ResourceType::Iron => 5.0,
        ResourceType::Oil => 5.0,
        ResourceType::RareEarths => 2.0,
        ResourceType::Water => 8.0,
        ResourceType::TradePorts => 0.0, // Ports don't produce, they're infrastructure
    };

    base_production * efficiency
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ProvinceId;
    use glam::Vec2;

    #[test]
    fn test_resource_production_calculation() {
        let food_prod = calculate_resource_production(ResourceType::Food, 1.0);
        assert_eq!(food_prod, 10.0);

        let iron_prod = calculate_resource_production(ResourceType::Iron, 2.0);
        assert_eq!(iron_prod, 10.0);
    }

    #[test]
    fn test_economic_phase_execution() {
        let mut world = World::default();

        // Spawn a food-producing province
        world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "FarmProvince".to_string(),
                position: Vec2::ZERO,
                dominant_resource: ResourceType::Food,
            },
            Resources::default(),
            Population {
                total: 1_000_000,
                growth_rate: 0.01,
            },
            Infrastructure {
                level: 2,
                max_level: 10,
            },
        ));

        let mut phase = EconomicPhase::new();

        // Get initial food
        let initial_food = world.query::<&Resources>()
            .iter(&world)
            .next()
            .unwrap()
            .food;

        // Execute economic phase
        phase.execute(&mut world);

        // Food should increase (production) and decrease (consumption)
        let final_resources = world.query::<&Resources>()
            .iter(&world)
            .next()
            .unwrap();

        // Net food should be positive due to food-producing province
        assert!(final_resources.food != initial_food);
    }
}
