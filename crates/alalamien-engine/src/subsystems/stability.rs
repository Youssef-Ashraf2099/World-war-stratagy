//! Stability subsystem for internal nation stability
//!
//! Handles legitimacy degradation from:
//! - Neighbor threats
//! - Multi-front wars
//! - Resource shortages
//! - War exhaustion
//!
//! When legitimacy drops too low, triggers:
//! - Protests
//! - Rebel movements
//! - Civil war

use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

use crate::core::tick::TickPhase;
use crate::core::types::{
    Nation, NationId, Legitimacy, Province, ProvinceId, OwnedBy, WarState, 
    Population, Resources
};
use crate::core::province_graph::ProvinceGraph;

/// Stability simulation phase
pub struct StabilityPhase {
    config: StabilityConfig,
}

#[derive(Debug, Clone)]
pub struct StabilityConfig {
    /// Legitimacy loss per neighboring hostile nation
    pub threat_penalty_per_neighbor: f64,
    /// Legitimacy loss per front in active war
    pub war_front_penalty: f64,
    /// Legitimacy loss per tick when losing territory
    pub territory_loss_penalty: f64,
    /// Legitimacy threshold for protests
    pub protest_threshold: f64,
    /// Legitimacy threshold for rebel activity
    pub rebel_threshold: f64,
    /// Legitimacy threshold for civil war
    pub civil_war_threshold: f64,
    /// Resource deficit penalty multiplier
    pub resource_deficit_multiplier: f64,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self {
            threat_penalty_per_neighbor: 0.5,
            war_front_penalty: 2.0,
            territory_loss_penalty: 3.0,
            protest_threshold: 35.0,
            rebel_threshold: 25.0,
            civil_war_threshold: 15.0,
            resource_deficit_multiplier: 1.0,
        }
    }
}

impl StabilityPhase {
    pub fn new() -> Self {
        Self {
            config: StabilityConfig::default(),
        }
    }

    pub fn with_config(config: StabilityConfig) -> Self {
        Self { config }
    }
}

impl TickPhase for StabilityPhase {
    fn name(&self) -> &str {
        "Stability"
    }

    fn execute(&mut self, world: &mut World) {
        let graph = world.resource::<ProvinceGraph>().clone();

        // Step 1: Gather nation-level threat data
        let nation_threats = calculate_neighbor_threats(world, &graph);
        let nation_war_fronts = calculate_war_fronts(world);
        let nation_deficits = calculate_resource_deficits(world);

        // Step 2: Calculate legitimacy changes for each nation
        let mut legitimacy_changes: HashMap<Entity, f64> = HashMap::new();

        {
            let mut nation_query = world.query::<(Entity, &Nation, &Legitimacy, &WarState)>();
            for (entity, nation, legitimacy, war_state) in nation_query.iter(world) {
                let mut delta = 0.0;

                // Penalty from neighbor threats
                let threat_count = nation_threats.get(&nation.id).unwrap_or(&0);
                delta -= *threat_count as f64 * self.config.threat_penalty_per_neighbor;

                // Penalty from active war fronts
                let front_count = nation_war_fronts.get(&nation.id).unwrap_or(&0);
                delta -= *front_count as f64 * self.config.war_front_penalty;

                // Penalty from resource deficits
                let deficit = nation_deficits.get(&nation.id).unwrap_or(&0.0);
                delta -= deficit * self.config.resource_deficit_multiplier;

                // Base stability drain if already unstable
                if legitimacy.value < 30.0 {
                    delta -= 0.5; // Instability feeds on itself
                }

                // Base slow recovery when stable
                if legitimacy.value > 60.0 && war_state.at_war_with.is_empty() {
                    delta += 0.1; // Slow recovery during peace
                }

                if delta != 0.0 {
                    legitimacy_changes.insert(entity, delta);
                    debug!(
                        nation = %nation.name,
                        legitimacy = legitimacy.value,
                        delta = delta,
                        threats = threat_count,
                        fronts = front_count,
                        "Legitimacy change calculated"
                    );
                }
            }
        }

        // Step 3: Apply legitimacy changes
        for (entity, delta) in legitimacy_changes {
            // First, get nation info before any mutable borrows
            let nation_id_opt = world.get::<Nation>(entity).map(|n| (n.id, n.name.clone()));
            
            if let Some(mut legitimacy) = world.get_mut::<Legitimacy>(entity) {
                let old_value = legitimacy.value;
                legitimacy.modify(delta);
                let new_value = legitimacy.value;

                // Check for stability events
                if let Some((nation_id, nation_name)) = nation_id_opt {
                    if new_value < self.config.protest_threshold && old_value >= self.config.protest_threshold {
                        warn!(nation = %nation_name, "Protests breaking out due to low legitimacy");
                        spawn_protest_event(world, entity, nation_id);
                    }

                    if new_value < self.config.rebel_threshold && old_value >= self.config.rebel_threshold {
                        warn!(nation = %nation_name, "Rebel movements forming");
                        spawn_rebel_event(world, entity, nation_id);
                    }

                    if new_value < self.config.civil_war_threshold && old_value >= self.config.civil_war_threshold {
                        warn!(nation = %nation_name, "CIVIL WAR ERUPTING");
                        trigger_civil_war(world, entity, nation_id);
                    }

                    if new_value < self.config.civil_war_threshold {
                        // Ongoing civil war effects
                        apply_civil_war_effects(world, nation_id);
                    }
                }
            }
        }
    }
}

/// Calculate how many hostile neighbors each nation has
fn calculate_neighbor_threats(world: &mut World, graph: &ProvinceGraph) -> HashMap<NationId, usize> {
    let mut threats: HashMap<NationId, HashSet<NationId>> = HashMap::new();

    // Build a set of provinces owned by each nation
    let mut nation_provinces: HashMap<NationId, HashSet<ProvinceId>> = HashMap::new();
    {
        let mut query = world.query::<(&Province, &OwnedBy)>();
        for (province, owner) in query.iter(world) {
            nation_provinces
                .entry(owner.nation_id)
                .or_insert_with(HashSet::new)
                .insert(province.id);
        }
    }

    // Find nations at war
    let mut at_war: HashMap<NationId, HashSet<NationId>> = HashMap::new();
    {
        let mut query = world.query::<(&Nation, &WarState)>();
        for (nation, war_state) in query.iter(world) {
            if !war_state.at_war_with.is_empty() {
                at_war.insert(nation.id, war_state.at_war_with.iter().copied().collect());
            }
        }
    }

    // For each nation, find neighboring nations
    for (nation_id, provinces) in &nation_provinces {
        let mut neighbors = HashSet::new();

        for &province_id in provinces {
            for neighbor_id in graph.get_neighbors(province_id) {
                // Find owner of neighbor province
                let mut neighbor_query = world.query::<(&Province, &OwnedBy)>();
                for (neighbor_prov, neighbor_owner) in neighbor_query.iter(world) {
                    if neighbor_prov.id == neighbor_id && neighbor_owner.nation_id != *nation_id {
                        neighbors.insert(neighbor_owner.nation_id);
                    }
                }
            }
        }

        // Count how many neighbors are at war with this nation
        let hostile_neighbors: HashSet<_> = neighbors
            .into_iter()
            .filter(|&neighbor_id| {
                at_war
                    .get(nation_id)
                    .map(|enemies| enemies.contains(&neighbor_id))
                    .unwrap_or(false)
            })
            .collect();

        threats.insert(*nation_id, hostile_neighbors);
    }

    threats.into_iter().map(|(id, set)| (id, set.len())).collect()
}

/// Calculate how many war fronts each nation is fighting on
fn calculate_war_fronts(world: &mut World) -> HashMap<NationId, usize> {
    let mut fronts: HashMap<NationId, usize> = HashMap::new();

    let mut query = world.query::<(&Nation, &WarState)>();
    for (nation, war_state) in query.iter(world) {
        // Each enemy is a separate front
        fronts.insert(nation.id, war_state.at_war_with.len());
    }

    fronts
}

/// Calculate aggregate resource deficits for each nation
fn calculate_resource_deficits(world: &mut World) -> HashMap<NationId, f64> {
    let mut deficits: HashMap<NationId, f64> = HashMap::new();

    let mut query = world.query::<(&Province, &OwnedBy, &Resources)>();
    for (_province, owner, resources) in query.iter(world) {
        let mut deficit = 0.0;

        // Food deficit is most critical
        if resources.food < 0.0 {
            deficit += resources.food.abs() * 2.0; // Double weight for food
        }

        // Other resource deficits
        if resources.iron < 0.0 {
            deficit += resources.iron.abs() * 0.5;
        }
        if resources.oil < 0.0 {
            deficit += resources.oil.abs() * 0.5;
        }

        *deficits.entry(owner.nation_id).or_insert(0.0) += deficit;
    }

    deficits
}

/// Spawn a protest event marker
fn spawn_protest_event(world: &mut World, _nation_entity: Entity, nation_id: NationId) {
    world.spawn(ProtestEvent {
        nation_id,
        intensity: 1.0,
    });
    info!(nation_id = ?nation_id, "Protests spawned");
}

/// Spawn rebel movement
fn spawn_rebel_event(world: &mut World, _nation_entity: Entity, nation_id: NationId) {
    world.spawn(RebelMovement {
        nation_id,
        strength: 0.3,
    });
    info!(nation_id = ?nation_id, "Rebel movement spawned");
}

/// Trigger civil war
fn trigger_civil_war(world: &mut World, _nation_entity: Entity, nation_id: NationId) {
    world.spawn(CivilWar {
        nation_id,
        rebel_strength: 0.5,
    });
    warn!(nation_id = ?nation_id, "Civil war triggered");
}

/// Apply ongoing civil war effects
fn apply_civil_war_effects(world: &mut World, nation_id: NationId) {
    // Reduce population in provinces undergoing civil war
    let mut query = world.query::<(&OwnedBy, &mut Population, &mut Resources)>();
    for (owner, mut population, mut resources) in query.iter_mut(world) {
        if owner.nation_id == nation_id {
            // Population casualties from civil war
            let casualties = (population.total as f64 * 0.001) as u64; // 0.1% per tick
            population.total = population.total.saturating_sub(casualties);

            // Resource destruction
            resources.food *= 0.98;
            resources.iron *= 0.95;
            resources.oil *= 0.95;
        }
    }

    debug!(nation_id = ?nation_id, "Civil war effects applied");
}

// ============================================================================
// EVENT MARKERS
// ============================================================================

/// Marker for protest events
#[derive(Debug, Clone, Component)]
pub struct ProtestEvent {
    pub nation_id: NationId,
    pub intensity: f64,
}

/// Marker for rebel movements
#[derive(Debug, Clone, Component)]
pub struct RebelMovement {
    pub nation_id: NationId,
    pub strength: f64,
}

/// Marker for civil war
#[derive(Debug, Clone, Component)]
pub struct CivilWar {
    pub nation_id: NationId,
    pub rebel_strength: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::world::WorldState;

    #[test]
    fn test_legitimacy_degradation_from_war() {
        let mut world_state = WorldState::new(42);
        
        // Create two nations at war
        let nation_a = world_state.spawn_nation("Nation A".to_string(), [255, 0, 0], false);
        let nation_b = world_state.spawn_nation("Nation B".to_string(), [0, 255, 0], false);
        
        let nation_b_id = world_state.world.get::<Nation>(nation_b).unwrap().id;
        
        // Set them at war
        if let Some(mut war_state) = world_state.world.get_mut::<WarState>(nation_a) {
            war_state.at_war_with.push(nation_b_id);
        }
        
        // Get initial legitimacy
        let initial_legitimacy = world_state.world.get::<Legitimacy>(nation_a).unwrap().value;
        
        // Run stability phase
        let mut phase = StabilityPhase::new();
        phase.execute(&mut world_state.world);
        
        // Legitimacy should have decreased
        let final_legitimacy = world_state.world.get::<Legitimacy>(nation_a).unwrap().value;
        assert!(final_legitimacy < initial_legitimacy, 
            "Legitimacy should decrease during war");
    }

    #[test]
    fn test_protest_threshold() {
        let mut world_state = WorldState::new(42);
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        
        // Set legitimacy just above protest threshold
        if let Some(mut legitimacy) = world_state.world.get_mut::<Legitimacy>(nation) {
            legitimacy.value = 36.0;
        }
        
        // Force legitimacy drop
        if let Some(mut war_state) = world_state.world.get_mut::<WarState>(nation) {
            // Create multiple war fronts
            war_state.at_war_with = vec![NationId::new(), NationId::new()];
        }
        
        let mut phase = StabilityPhase::new();
        phase.execute(&mut world_state.world);
        
        // Check if protest event was spawned
        let protest_count = world_state.world.query::<&ProtestEvent>().iter(&world_state.world).count();
        assert!(protest_count > 0, "Protest should be spawned when legitimacy drops below threshold");
    }
}
