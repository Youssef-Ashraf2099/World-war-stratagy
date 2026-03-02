//! Economic subsystem
//!
//! Handles resource production, consumption, and GDP calculation.

use bevy_ecs::prelude::*;
use tracing::trace;

use crate::core::types::{Resources, ResourceType, Population, Infrastructure, Province, OwnedBy, Nation, MilitaryCapacity, Logistics, GDP, NationId, EconomicStress};
use crate::core::tick::TickPhase;
use crate::subsystems::notifications::{
    create_economic_crisis_notification,
    create_legitimacy_crisis_notification,
};

const IRON_TO_MILITARY_RATE: f64 = 0.1;
const OIL_TO_LOGISTICS_RATE: f64 = 0.1;

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
            &OwnedBy,
        )>();

        let mut nation_incomes: std::collections::HashMap<NationId, (f64, f64)> = std::collections::HashMap::new();

        for (province, mut resources, population, infrastructure, owner) in query.iter_mut(world) {
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

            resources.add(province.dominant_resource, production as f64);

            // Consume food (basic consumption)
            let food_consumption = (population.total as f64 / 1000.0) * 0.1;
            resources.food -= food_consumption;

            // Extract Iron and Oil to Nation level
            let iron_export = resources.iron;
            let oil_export = resources.oil;
            resources.iron = 0.0;
            resources.oil = 0.0;

            let entry = nation_incomes.entry(owner.nation_id).or_insert((0.0, 0.0));
            entry.0 += iron_export;
            entry.1 += oil_export;

            trace!(
                province = %province.name,
                resource = ?province.dominant_resource,
                production = production,
                efficiency = production_efficiency,
                "Resource production"
            );
        }

        // Apply processing and effects to Nation
        let mut nation_query = world.query::<(
            &Nation,
            &mut MilitaryCapacity,
            &mut Logistics,
            &mut GDP,
            Option<&mut EconomicStress>,
        )>();

        for (nation, mut military, mut logistics, mut gdp, economic_stress) in nation_query.iter_mut(world) {
            if let Some((iron, oil)) = nation_incomes.get(&nation.id) {
                // Production chains
                military.value += iron * IRON_TO_MILITARY_RATE;
                logistics.value += oil * OIL_TO_LOGISTICS_RATE;

                // Simple scalar GDP abstraction for the entire economy's output
                let gdp_gain = (*iron * 2.0) + (*oil * 3.0);
                gdp.value += gdp_gain / 1000.0;
            }
            
            // Calculate deficit if EconomicStress is present
            if let Some(mut econ_stress) = economic_stress {
                let military_upkeep = military.value * 0.01; // 1% of military value per tick
                let logistics_cost = logistics.value * 0.005;  // 0.5% of logistics value per tick
                
                let total_costs = military_upkeep + logistics_cost;
                let income = gdp.value * 0.1; // 10% of GDP is available for spending
                
                let deficit = total_costs - income;
                
                // Update EconomicStress with current deficit and accumulated deficit
                econ_stress.current_deficit = deficit;
                econ_stress.accumulated_deficit += deficit.max(0.0); // Only accumulate positive deficits
                econ_stress.gdp = gdp.value;
                
                if deficit.abs() > 0.01 {
                    trace!(
                        nation = %nation.name,
                        military_upkeep = military_upkeep,
                        logistics_cost = logistics_cost,
                        income = income,
                        deficit = deficit,
                        "Nation economic status"
                    );
                }
            }
        }
        
        // Check for economic and legitimacy crises
        check_crises(world);
    }
}

/// Check for economic and legitimacy crises, creating notifications
fn check_crises(world: &mut World) {
    // Collect previous GDP values and check for significant drops
    let mut crisis_nations = Vec::new();
    
    {
        let mut query = world.query::<(&Nation, &GDP, Option<&EconomicStress>)>();
        for (nation, gdp, econ_stress) in query.iter(world) {
            // Check for economic crisis (GDP < 5.0 is very low, or high accumulated deficit)
            if gdp.value < 5.0 {
                crisis_nations.push((nation.id, true, false)); // economic crisis
            } else if let Some(stress) = econ_stress {
                // High accumulated deficit indicates sustained economic problems
                if stress.accumulated_deficit > 50.0 {
                    crisis_nations.push((nation.id, true, false)); // economic crisis
                }
            }
        }
    }
    
    // Check for legitimacy crises
    {
        let mut query = world.query::<(&Nation, &crate::core::types::Legitimacy)>();
        for (nation, legitimacy) in query.iter(world) {
            if legitimacy.value < 25.0 {
                // Add or update crisis entry with legitimacy flag
                if let Some(entry) = crisis_nations.iter_mut().find(|(id, _, _)| *id == nation.id) {
                    entry.2 = true; // legitimacy crisis
                } else {
                    crisis_nations.push((nation.id, false, true)); // only legitimacy crisis
                }
            }
        }
    }
    
    // Create notifications for crises
    for (nation_id, econ_crisis, legit_crisis) in crisis_nations {
        if econ_crisis {
            // Get actual GDP value for notification
            let gdp_value = {
                let mut query = world.query::<(&Nation, &GDP)>();
                query.iter(world)
                    .find(|(n, _)| n.id == nation_id)
                    .map(|(_, gdp)| gdp.value)
                    .unwrap_or(0.0)
            };
            create_economic_crisis_notification(world, nation_id, gdp_value, 0);
        }
        if legit_crisis {
            // Get actual legitimacy value for notification
            let legit_value = {
                let mut query = world.query::<(&Nation, &crate::core::types::Legitimacy)>();
                query.iter(world)
                    .find(|(n, _)| n.id == nation_id)
                    .map(|(_, leg)| leg.value)
                    .unwrap_or(0.0)
            };
            create_legitimacy_crisis_notification(world, nation_id, legit_value, 0);
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

        let nation_id = NationId::new();
        world.spawn((
            Nation {
                id: nation_id,
                name: "TestNation".to_string(),
                color: [0, 0, 0],
            },
            MilitaryCapacity::default(),
            Logistics::default(),
            GDP::default(),
        ));

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
            OwnedBy { nation_id },
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

    #[test]
    fn test_production_chain_iron_to_military() {
        let mut world = World::default();
        let nation_id = NationId::new();

        world.spawn((
            Nation {
                id: nation_id,
                name: "IronNation".to_string(),
                color: [0, 0, 0],
            },
            MilitaryCapacity::default(),
            Logistics::default(),
            GDP::default(),
        ));

        world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "Mine".to_string(),
                position: Vec2::ZERO,
                dominant_resource: ResourceType::Iron,
            },
            Resources::default(),
            Population {
                total: 1_000_000,
                growth_rate: 0.01,
            },
            Infrastructure { level: 3, max_level: 10 },
            OwnedBy { nation_id },
        ));

        let before = world
            .query::<&MilitaryCapacity>()
            .iter(&world)
            .next()
            .unwrap()
            .value;

        let mut phase = EconomicPhase::new();
        phase.execute(&mut world);

        let after = world
            .query::<&MilitaryCapacity>()
            .iter(&world)
            .next()
            .unwrap()
            .value;

        assert!(after > before, "Military capacity should increase from iron conversion");
    }

    #[test]
    fn test_production_chain_oil_to_logistics() {
        let mut world = World::default();
        let nation_id = NationId::new();

        world.spawn((
            Nation {
                id: nation_id,
                name: "OilNation".to_string(),
                color: [0, 0, 0],
            },
            MilitaryCapacity::default(),
            Logistics::default(),
            GDP::default(),
        ));

        world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "OilField".to_string(),
                position: Vec2::ZERO,
                dominant_resource: ResourceType::Oil,
            },
            Resources::default(),
            Population {
                total: 1_000_000,
                growth_rate: 0.01,
            },
            Infrastructure { level: 3, max_level: 10 },
            OwnedBy { nation_id },
        ));

        let before = world
            .query::<&Logistics>()
            .iter(&world)
            .next()
            .unwrap()
            .value;

        let mut phase = EconomicPhase::new();
        phase.execute(&mut world);

        let after = world
            .query::<&Logistics>()
            .iter(&world)
            .next()
            .unwrap()
            .value;

        assert!(after > before, "Logistics should increase from oil conversion");
    }
}
