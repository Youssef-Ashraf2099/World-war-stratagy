//! External Intervention subsystem (V0.6 Week 4)
//!
//! When a nation collapses into factions, neighboring nations can intervene:
//! - Choose which faction to support
//! - Provide military aid (armies/resources)
//! - Gain/lose reputation based on faction outcomes
//! - Accept refugees from defeated factions
//!
//! ## Intervention Timeline
//! 1. Collapse detected (Factionalized component added)
//! 2. Neighboring nations decide to intervene (random 50% chance if good relations)
//! 3. Military support transfers to chosen faction
//! 4. Civil war resolves (one faction wins)
//! 5. Consequences applied (reputation, refugees, alliances)

use bevy_ecs::prelude::*;
use tracing::{debug, info};

use crate::core::tick::TickPhase;
use crate::core::types::{
    Nation, NationId, DiplomaticRelation, OwnedBy, Resources,
};
use crate::core::deterministic::DeterministicRng;
use crate::subsystems::factions::{Faction, Factionalized, CivilWarState};

// Types needed for tests
#[cfg(test)]
use crate::core::types::{Province, ProvinceId, Population, Legitimacy};

// ============================================================================
// INTERVENTION TYPES
// ============================================================================

/// Unique identifier for interventions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct InterventionId(pub uuid::Uuid);

impl InterventionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for InterventionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Records an active intervention by one nation in another nation's civil war
#[derive(Debug, Clone, Component)]
pub struct Intervention {
    pub id: InterventionId,
    pub intervener_nation_id: NationId,
    pub civil_war_parent_id: NationId,  // Original nation that collapsed
    pub supported_faction_id: NationId,
    pub start_tick: u64,
    pub military_aid: u32,              // Number of armies sent as aid
    pub resource_aid: Resources,        // Resources provided
    pub status: InterventionStatus,     // Active, succeeded, failed
}

/// Status of an intervention
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterventionStatus {
    /// Intervention is ongoing
    Active,
    /// Supported faction won - intervener gains benefits
    Succeeded,
    /// Supported faction lost - intervener has consequences
    Failed,
    /// Civil war ended (original nation reunified) before faction fully defeated
    Withdrawn,
}

/// Marker component indicating this entity tracks the outcome of an intervention
#[derive(Debug, Clone, Component)]
pub struct InterventionOutcome {
    pub intervention_id: InterventionId,
    pub intervener_id: NationId,
    pub result: OutcomeType,
}

/// Types of intervention outcomes
#[derive(Debug, Clone)]
pub enum OutcomeType {
    /// Intervention succeeded, supported faction won
    FactionalVictory { faction_id: NationId },
    /// Intervention failed, supported faction defeated
    FactionalDefeat { faction_id: NationId },
}

/// Marker for nations that are currently intervening in civil wars
#[derive(Debug, Clone, Component)]
pub struct ActiveIntervenor {
    pub interventions: Vec<InterventionId>,
}

/// Tracks refugee crisis in a nation (from failed interventions)
#[derive(Debug, Clone, Component)]
pub struct RefugeeCrisis {
    pub source_nation_id: NationId,          // Original nation from civil war
    pub lost_faction_id: NationId,           // Faction that lost
    pub refugee_population: u64,             // Number of people
    pub arrived_tick: u64,                   // When refugees arrived
    pub integration_ticks_remaining: u64,    // How long until fully integrated (500 ticks)
    pub morale_penalty: f64,                 // -2.0 to -5.0
}

/// Protectorate alliance formed after faction victory with supporter aid
#[derive(Debug, Clone, Component)]
pub struct ProtectorateAlly {
    pub protector_nation_id: NationId,       // Nation that supported faction
    pub protected_faction_id: NationId,      // Original faction that won
    pub original_parent_id: NationId,        // Original nation that collapsed
    pub formed_tick: u64,
    pub mutual_defense: bool,                // Both defend each other
    pub trade_bonus: f64,                    // +10% resources from trade
}

// ============================================================================
// INTERVENTION PHASE
// ============================================================================

pub struct InterventionPhase {
    processed_wars: std::collections::HashSet<NationId>,
}

impl InterventionPhase {
    pub fn new() -> Self {
        Self {
            processed_wars: std::collections::HashSet::new(),
        }
    }

    /// Detect new civil wars and enable interventions
    fn detect_new_civil_wars(&mut self, world: &mut World) {
        let mut new_civil_wars: Vec<(Entity, NationId, usize)> = Vec::new();

        // Find newly factionalized nations (those with Factionalized but no active interventions)
        let mut query = world.query::<(Entity, &Nation, &Factionalized)>();
        for (entity, nation, factionalized) in query.iter(world) {
            if !self.processed_wars.contains(&nation.id) {
                new_civil_wars.push((entity, nation.id, factionalized.num_factions));
                self.processed_wars.insert(nation.id);
            }
        }

        // Process each new civil war
        for (parent_entity, parent_nation_id, num_factions) in new_civil_wars {
            debug!(
                parent_nation = ?parent_nation_id,
                num_factions = num_factions,
                "New civil war detected - enabling interventions"
            );
            
            self.enable_interventions_for_war(world, parent_entity, parent_nation_id);
        }
    }

    /// Enable neighboring nations to intervene in a civil war
    fn enable_interventions_for_war(
        &self,
        world: &mut World,
        _parent_entity: Entity,
        parent_nation_id: NationId,
    ) {
        // Find all provinces of the collapsing nation (now owned by factions)
        let neighbor_nations = self.find_neighboring_nations(world, parent_nation_id);

        // For each neighbor, decide if they intervene
        for neighbor_id in neighbor_nations {
            let should_intervene = self.should_intervene(world, neighbor_id, parent_nation_id);
            
            if should_intervene {
                // Pick which faction to support
                if let Some(faction_id) = self.select_faction_to_support(world, parent_nation_id, neighbor_id) {
                    debug!(
                        intervener = ?neighbor_id,
                        supported_faction = ?faction_id,
                        "Neighbor nation decided to intervene"
                    );
                    
                    self.create_intervention(world, neighbor_id, parent_nation_id, faction_id);
                }
            }
        }
    }

    /// Find all nations neighboring a collapsing nation
    fn find_neighboring_nations(&self, world: &mut World, parent_nation_id: NationId) -> Vec<NationId> {
        let mut neighbors = std::collections::HashSet::new();

        // Get all provinces owned by any faction of the parent nation
        let factions: Vec<NationId> = world
            .query::<(&Faction, &Nation)>()
            .iter(world)
            .filter(|(faction, _)| faction.parent_nation_id == parent_nation_id)
            .map(|(_, nation)| nation.id)
            .collect();

        // Get all province owners to detect neighbours
        let faction_owners: Vec<NationId> = world
            .query::<&OwnedBy>()
            .iter(world)
            .filter(|owner| factions.contains(&owner.nation_id))
            .map(|owner| owner.nation_id)
            .collect();

        // If any faction owns provinces, find all other nation owners
        if !faction_owners.is_empty() {
            let all_nation_owners: Vec<NationId> = world
                .query::<&OwnedBy>()
                .iter(world)
                .map(|owner| owner.nation_id)
                .collect();

            for owner_id in all_nation_owners {
                // This is a non-faction nation
                if !factions.contains(&owner_id) && owner_id != parent_nation_id {
                    neighbors.insert(owner_id);
                }
            }
        }

        neighbors.into_iter().collect()
    }

    /// Determine if a nation should intervene (currently 50% chance if relations good)
    fn should_intervene(
        &self,
        world: &mut World,
        intervener_id: NationId,
        parent_nation_id: NationId,
    ) -> bool {
        // Check if intervener has good relations with parent nation
        let rng = world.resource::<DeterministicRng>();
        let random_value = rng.next_f64();

        // 50% chance to intervene if we have any relation
        if random_value > 0.5 {
            return false;
        }

        // Check diplomatic relation (prefer to intervene if we have good relations with parent)
        let mut query = world.query::<&DiplomaticRelation>();
        for relation in query.iter(world) {
            if (relation.nation_a == intervener_id && relation.nation_b == parent_nation_id)
                || (relation.nation_a == parent_nation_id && relation.nation_b == intervener_id)
            {
                // Only intervene if reputation > 0 or we're indifferent
                return relation.reputation >= -10.0;
            }
        }

        // Default: intervene with 50% chance even without relation
        true
    }

    /// Select which faction to support (favor closest/largest)
    fn select_faction_to_support(
        &self,
        world: &mut World,
        parent_nation_id: NationId,
        _intervener_id: NationId,
    ) -> Option<NationId> {
        // Find all factions from this civil war
        let factions: Vec<NationId> = world
            .query::<(&Faction, &Nation)>()
            .iter(world)
            .filter(|(faction, _)| faction.parent_nation_id == parent_nation_id)
            .map(|(_, nation)| nation.id)
            .collect();

        if factions.is_empty() {
            return None;
        }

        // Pick a random faction to support
        let rng = world.resource::<DeterministicRng>();
        let idx = rng.next_usize(factions.len());
        Some(factions[idx])
    }

    /// Create intervention record and transfer military aid
    fn create_intervention(
        &self,
        world: &mut World,
        intervener_id: NationId,
        parent_nation_id: NationId,
        supported_faction_id: NationId,
    ) {
        let intervention_id = InterventionId::new();
        let rng = world.resource::<DeterministicRng>();
        
        // Determine amount of military aid (1-3 as tech level)
        let military_aid = (rng.next_usize(3) + 1) as u32;
        
        // Small resource aid
        let resource_aid = Resources {
            food: 100.0,
            iron: 50.0,
            oil: 25.0,
            rare_earths: 10.0,
            water: 75.0,
            trade_ports: 0,
        };

        // Create intervention record
        world.spawn((
            Intervention {
                id: intervention_id,
                intervener_nation_id: intervener_id,
                civil_war_parent_id: parent_nation_id,
                supported_faction_id,
                start_tick: 0, // TODO: Add tick tracking
                military_aid,
                resource_aid: resource_aid.clone(),
                status: InterventionStatus::Active,
            },
        ));

        info!(
            intervention_id = ?intervention_id,
            intervener = ?intervener_id,
            supported_faction = ?supported_faction_id,
            military_aid = military_aid,
            "Intervention created - military aid committed"
        );

        // Transfer military aid from intervener to supported faction
        // Find aid armies from intervener
        let mut intervener_armies: Vec<crate::core::types::ArmyId> = world
            .query::<&crate::core::types::Army>()
            .iter(world)
            .filter(|army| army.owner == intervener_id)
            .map(|army| army.army_id)
            .collect();

        // Select random armies from intervener to transfer as aid
        if !intervener_armies.is_empty() {
            let rng = world.resource::<DeterministicRng>();
            let num_to_transfer = (military_aid as usize).min(intervener_armies.len());
            
            // Shuffle and select first N armies
            for i in (1..intervener_armies.len()).rev() {
                let j = rng.next_usize(i + 1);
                intervener_armies.swap(i, j);
            }

            let armies_to_transfer = &intervener_armies[..num_to_transfer];

            // Transfer ownership
            let mut query = world.query::<&mut crate::core::types::Army>();
            for mut army in query.iter_mut(world) {
                if armies_to_transfer.contains(&army.army_id) {
                    army.owner = supported_faction_id;
                }
            }

            debug!(
                intervener = ?intervener_id,
                faction = ?supported_faction_id,
                armies_transferred = num_to_transfer,
                "Military aid transferred to faction"
            );
        }

        // Transfer resource aid to faction
        let mut query = world.query::<(&OwnedBy, &mut Resources)>();
        let mut transferred = false;
        for (owner, mut resources) in query.iter_mut(world) {
            if owner.nation_id == supported_faction_id {
                resources.food += resource_aid.food;
                resources.iron += resource_aid.iron;
                resources.oil += resource_aid.oil;
                resources.rare_earths += resource_aid.rare_earths;
                resources.water += resource_aid.water;
                transferred = true;
                break;
            }
        }

        if transferred {
            debug!(
                faction = ?supported_faction_id,
                food = resource_aid.food,
                iron = resource_aid.iron,
                "Resource aid transferred to faction"
            );
        }
    }

    /// Resolve completed interventions based on faction outcomes
    fn resolve_completed_interventions(&mut self, world: &mut World) {
        let mut completed_interventions = Vec::new();

        // Find interventions where supported faction is no longer at war (faction won)
        // or has been conquered (faction lost)
        let mut intervention_query = world.query::<(Entity, &Intervention)>();
        let interventions: Vec<(Entity, Intervention)> = intervention_query
            .iter(world)
            .map(|(e, i)| (e, i.clone()))
            .collect();

        for (entity, intervention) in interventions {
            if intervention.status == InterventionStatus::Active {
                // Check if civil war has ended
                let civil_war_parent = {
                    let mut query = world.query::<(&Nation, &CivilWarState)>();
                    query
                        .iter(world)
                        .find(|(nation, _)| nation.id == intervention.civil_war_parent_id)
                        .map(|(_, cws)| cws.clone())
                };

                if civil_war_parent.is_none() {
                    // Civil war has ended (no CivilWarState component = faction won)
                    completed_interventions.push((entity, intervention.clone(), true));
                }
            }
        }

        // Resolve each completed intervention
        for (entity, intervention, faction_won) in completed_interventions {
            if faction_won {
                debug!(
                    intervener = ?intervention.intervener_nation_id,
                    supported_faction = ?intervention.supported_faction_id,
                    "Intervention resolved - faction victory"
                );
                self.apply_intervention_success(world, &intervention);
            } else {
                debug!(
                    intervener = ?intervention.intervener_nation_id,
                    supported_faction = ?intervention.supported_faction_id,
                    "Intervention resolved - faction defeat"
                );
                self.apply_intervention_failure(world, &intervention);
            }

            // Remove the intervention record
            world.entity_mut(entity).despawn();
        }
    }

    /// Apply consequences when supported faction wins
    fn apply_intervention_success(
        &self,
        world: &mut World,
        intervention: &Intervention,
    ) {
        // Add reputation bonus to intervener
        let mut query = world.query::<&mut DiplomaticRelation>();
        for mut relation in query.iter_mut(world) {
            if (relation.nation_a == intervention.intervener_nation_id 
                && relation.nation_b == intervention.supported_faction_id)
                || (relation.nation_a == intervention.supported_faction_id 
                    && relation.nation_b == intervention.intervener_nation_id)
            {
                relation.reputation += 15.0; // Success bonus
                relation.reputation = relation.reputation.clamp(-100.0, 100.0);
            }
        }

        // Form protectorate alliance
        let rng = world.resource::<DeterministicRng>();
        let should_form_protectorate = rng.next_f64() > 0.3; // 70% chance

        if should_form_protectorate {
            world.spawn(ProtectorateAlly {
                protector_nation_id: intervention.intervener_nation_id,
                protected_faction_id: intervention.supported_faction_id,
                original_parent_id: intervention.civil_war_parent_id,
                formed_tick: 0, // TODO: Add tick tracking
                mutual_defense: true,
                trade_bonus: 0.1,
            });

            info!(
                intervener = ?intervention.intervener_nation_id,
                faction = ?intervention.supported_faction_id,
                "Protectorate alliance formed"
            );
        }

        info!(
            intervener = ?intervention.intervener_nation_id,
            faction = ?intervention.supported_faction_id,
            "Intervention succeeded - reputation bonus applied (+15)"
        );
    }

    /// Apply consequences when supported faction loses
    fn apply_intervention_failure(
        &self,
        world: &mut World,
        intervention: &Intervention,
    ) {
        // Apply reputation penalty
        let mut query = world.query::<&mut DiplomaticRelation>();
        for mut relation in query.iter_mut(world) {
            if (relation.nation_a == intervention.intervener_nation_id 
                && relation.nation_b == intervention.supported_faction_id)
                || (relation.nation_a == intervention.supported_faction_id 
                    && relation.nation_b == intervention.intervener_nation_id)
            {
                relation.reputation -= 10.0;
                relation.reputation = relation.reputation.clamp(-100.0, 100.0);
            }
        }

        // Generate refugee crisis
        let rng = world.resource::<DeterministicRng>();
        let refugee_population = (rng.next_u64() % 500_000) + 100_000; // 100k-600k refugees
        let morale_penalty = -(2.0 + rng.next_f64() * 3.0); // -2 to -5

        // Create refugee crisis component
        world.spawn(RefugeeCrisis {
            source_nation_id: intervention.civil_war_parent_id,
            lost_faction_id: intervention.supported_faction_id,
            refugee_population,
            arrived_tick: 0, // TODO: Add tick tracking
            integration_ticks_remaining: 500,
            morale_penalty,
        });

        info!(
            intervener = ?intervention.intervener_nation_id,
            refugees = refugee_population,
            "Intervention failed - refugee crisis created ({} population, morale penalty: {:.1})",
            refugee_population,
            morale_penalty
        );
    }
}

impl Default for InterventionPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl TickPhase for InterventionPhase {
    fn name(&self) -> &str {
        "Intervention"
    }

    fn execute(&mut self, world: &mut World) {
        // Step 1: Detect new civil wars
        self.detect_new_civil_wars(world);

        // Step 2: Resolve completed interventions
        self.resolve_completed_interventions(world);

        debug!("InterventionPhase complete");
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::world::WorldState;

    #[test]
    fn test_intervention_creation() {
        let mut world_state = WorldState::new(42);

        // Create parent nation
        let parent = world_state.spawn_nation("Parent Nation".to_string(), [100, 150, 200], false);
        let parent_id = world_state.world.get::<Nation>(parent).unwrap().id;

        // Create neighbor nation
        let neighbor = world_state.spawn_nation("Neighbor Nation".to_string(), [200, 100, 50], false);
        let neighbor_id = world_state.world.get::<Nation>(neighbor).unwrap().id;

        // Spawn provinces for both nations
        for i in 0..3 {
            world_state.world.spawn((
                Province {
                    id: ProvinceId::new(),
                    name: format!("Parent Province {}", i),
                    position: glam::Vec2::new(i as f32 * 10.0, 0.0),
                    dominant_resource: crate::core::types::ResourceType::Food,
                },
                OwnedBy { nation_id: parent_id },
                Population::default(),
                Resources::default(),
            ));
        }

        // Neighbor province
        world_state.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "Neighbor Province".to_string(),
                position: glam::Vec2::new(30.0, 0.0),
                dominant_resource: crate::core::types::ResourceType::Iron,
            },
            OwnedBy { nation_id: neighbor_id },
            Population::default(),
            Resources::default(),
        ));

        // Create diplomatic relation
        world_state.world.spawn(DiplomaticRelation {
            nation_a: parent_id,
            nation_b: neighbor_id,
            reputation: 10.0,
            trade_dependency: 0.2,
            threat_alignment: 0.5,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        });

        // Trigger collapse
        world_state.world.entity_mut(parent).insert(Legitimacy::new(0.0));

        // Run factions phase to spawn factions
        use crate::subsystems::factions::FactionCivilWarPhase;
        let mut factions_phase = FactionCivilWarPhase::new();
        factions_phase.execute(&mut world_state.world);

        // Run intervention phase
        let mut intervention_phase = InterventionPhase::new();
        intervention_phase.execute(&mut world_state.world);

        // Verify intervention was created (if random chance allowed)
        let intervention_count = world_state.world.query::<&Intervention>().iter(&world_state.world).count();
        // Note: This might be 0 or 1 depending on random chance
        println!("Interventions created: {}", intervention_count);
    }

    #[test]
    fn test_intervention_resolution() {
        let mut world_state = WorldState::new(99);

        let parent = world_state.spawn_nation("Parent".to_string(), [100, 100, 100], false);
        let parent_id = world_state.world.get::<Nation>(parent).unwrap().id;

        // Create provinces
        for i in 0..2 {
            world_state.world.spawn((
                Province {
                    id: ProvinceId::new(),
                    name: format!("Province {}", i),
                    position: glam::Vec2::ZERO,
                    dominant_resource: crate::core::types::ResourceType::Food,
                },
                OwnedBy { nation_id: parent_id },
                Population::default(),
                Resources::default(),
            ));
        }

        // Collapse and create factions
        world_state.world.entity_mut(parent).insert(Legitimacy::new(0.0));

        use crate::subsystems::factions::FactionCivilWarPhase;
        let mut factions_phase = FactionCivilWarPhase::new();
        factions_phase.execute(&mut world_state.world);

        // Get faction IDs
        let faction_ids: Vec<NationId> = world_state.world
            .query::<(&Faction, &Nation)>()
            .iter(&world_state.world)
            .map(|(_, nation)| nation.id)
            .collect();

        // Manually create an intervention (bypassing random chance)
        if !faction_ids.is_empty() {
            let intervener = world_state.spawn_nation("Intervener".to_string(), [50, 50, 50], false);
            let intervener_id = world_state.world.get::<Nation>(intervener).unwrap().id;

            world_state.world.spawn(Intervention {
                id: InterventionId::new(),
                intervener_nation_id: intervener_id,
                civil_war_parent_id: parent_id,
                supported_faction_id: faction_ids[0],
                start_tick: 0,
                military_aid: 2,
                resource_aid: Resources::default(),
                status: InterventionStatus::Active,
            });

            let initial_interventions = world_state.world.query::<&Intervention>().iter(&world_state.world).count();
            assert_eq!(initial_interventions, 1, "Should have 1 active intervention");

            // Run resolution (intervention should still be active since civil war is ongoing)
            let mut intervention_phase = InterventionPhase::new();
            intervention_phase.execute(&mut world_state.world);

            let final_interventions = world_state.world.query::<&Intervention>().iter(&world_state.world).count();
            // Should still be 1 since civil war is ongoing
            assert_eq!(final_interventions, 1, "Intervention should still be active");
        }
    }

    #[test]
    fn test_neighboring_nations_detection() {
        let mut world_state = WorldState::new(42);

        let parent = world_state.spawn_nation("Parent".to_string(), [100, 100, 100], false);
        let parent_id = world_state.world.get::<Nation>(parent).unwrap().id;

        let neighbor = world_state.spawn_nation("Neighbor".to_string(), [200, 100, 50], false);
        let neighbor_id = world_state.world.get::<Nation>(neighbor).unwrap().id;

        // Create provinces
        world_state.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "Parent Province".to_string(),
                position: glam::Vec2::new(0.0, 0.0),
                dominant_resource: crate::core::types::ResourceType::Food,
            },
            OwnedBy { nation_id: parent_id },
            Population::default(),
            Resources::default(),
        ));

        world_state.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "Neighbor Province".to_string(),
                position: glam::Vec2::new(25.0, 0.0), // Close to parent
                dominant_resource: crate::core::types::ResourceType::Iron,
            },
            OwnedBy { nation_id: neighbor_id },
            Population::default(),
            Resources::default(),
        ));

        // Collapse parent
        world_state.world.entity_mut(parent).insert(Legitimacy::new(0.0));

        use crate::subsystems::factions::FactionCivilWarPhase;
        let mut factions_phase = FactionCivilWarPhase::new();
        factions_phase.execute(&mut world_state.world);

        // Find neighbors (implementation detail)
        let intervention_phase = InterventionPhase::new();
        let neighbors = intervention_phase.find_neighboring_nations(&mut world_state.world, parent_id);

        println!("Found neighbors: {:?}", neighbors);
        // Should find neighbor (if province adjacency detection works)
    }
}
