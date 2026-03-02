//! Faction and Civil War subsystem (V0.6)
//!
//! When a nation's legitimacy reaches 0, it collapses into multiple factions:
//! - Provinces are deterministically split between 2-4 factions
//! - Each faction inherits proportional resources and armies
//! - Factions start with low legitimacy (10-20 range)
//! - Parent nation is marked as collapsed (Factionalized component)
//!
//! ## Phase Execution
//! This phase runs after stability calculations to detect and spawn factions.

use bevy_ecs::prelude::*;
use tracing::{debug, info, warn};

use crate::core::tick::TickPhase;
use crate::core::types::{
    Nation, NationId, Legitimacy, Province, ProvinceId, OwnedBy, Resources,
    Population, GDP, WarState, EconomicStress, CasualtyLog, AllianceCrisisLog,
    DiplomaticIsolationLog, Army, ArmyId,
};
use crate::core::deterministic::DeterministicRng;

// ============================================================================
// FACTION TYPES
// ============================================================================

/// Unique identifier for factions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct FactionId(pub uuid::Uuid);

impl FactionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for FactionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Faction entity component - represents a breakaway faction from a parent nation
#[derive(Debug, Clone, Component)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub parent_nation_id: NationId,
    pub color: [u8; 3],
    pub formation_tick: u64,
}

/// Marks the parent nation entity that spawned this faction
#[derive(Debug, Clone, Component)]
pub struct FactionParent {
    pub original_nation_id: NationId,
}

/// Tracks ongoing civil war state for a nation
#[derive(Debug, Clone, Component)]
pub struct CivilWarState {
    pub start_tick: u64,
    pub num_factions: usize,
    pub faction_ids: Vec<FactionId>,
}

/// Marker component indicating a nation has collapsed into factions
/// When present, the parent nation entity should be treated as defunct
#[derive(Debug, Clone, Component)]
pub struct Factionalized {
    pub collapse_tick: u64,
    pub num_factions: usize,
}

// ============================================================================
// FACTION CIVIL WAR PHASE
// ============================================================================

pub struct FactionCivilWarPhase;

impl FactionCivilWarPhase {
    pub fn new() -> Self {
        Self
    }

    /// Detect nations with legitimacy = 0 and spawn factions
    fn detect_and_spawn_collapses(&mut self, world: &mut World) {
        let mut collapsing_nations: Vec<(Entity, NationId, String, [u8; 3])> = Vec::new();

        // Step 1: Find nations with legitimacy = 0 that haven't already collapsed
        {
            let mut query = world.query::<(Entity, &Nation, &Legitimacy)>();
            for (entity, nation, legitimacy) in query.iter(world) {
                // Check if legitimacy is at 0 and nation hasn't already factionalized
                if legitimacy.value <= 0.0 && world.get::<Factionalized>(entity).is_none() {
                    collapsing_nations.push((entity, nation.id, nation.name.clone(), nation.color));
                }
            }
        }

        // Step 2: Spawn factions for each collapsing nation
        for (nation_entity, nation_id, nation_name, nation_color) in collapsing_nations {
            warn!(
                nation_id = ?nation_id,
                nation_name = %nation_name,
                "Nation collapse detected - spawning factions"
            );

            self.spawn_factions_for_nation(world, nation_entity, nation_id, nation_name, nation_color);
        }
    }

    /// Spawn factions for a collapsing nation
    fn spawn_factions_for_nation(
        &mut self,
        world: &mut World,
        parent_entity: Entity,
        parent_nation_id: NationId,
        parent_name: String,
        parent_color: [u8; 3],
    ) {
        // Get provinces owned by parent nation
        let provinces = self.gather_nation_provinces(world, parent_nation_id);
        
        if provinces.is_empty() {
            warn!(
                nation_id = ?parent_nation_id,
                "No provinces found for collapsing nation - skipping faction spawn"
            );
            return;
        }

        // Determine number of factions (2-4 based on province count)
        let num_factions = calculate_faction_count(provinces.len());

        // Gather parent nation's resources, armies, and population
        let parent_resources = self.gather_nation_resources(world, parent_nation_id);
        let parent_population = self.count_nation_population(world, parent_nation_id);
        let parent_armies = self.gather_nation_armies(world, parent_nation_id);

        // Get deterministic RNG from world and split provinces and armies
        let (province_splits, army_splits) = {
            let rng = world.resource::<DeterministicRng>();
            let province_splits = split_provinces(&provinces, num_factions, rng);
            let army_splits = split_armies(&parent_armies, num_factions, rng);
            (province_splits, army_splits)
        };

        // Spawn faction entities and track their nation IDs
        let mut faction_ids = Vec::new();
        let mut faction_nation_ids = Vec::new();
        for (faction_index, faction_provinces) in province_splits.iter().enumerate() {
            let faction_armies = army_splits.get(faction_index).cloned().unwrap_or_default();
            let (faction_id, faction_nation_id) = self.spawn_faction(
                world,
                parent_nation_id,
                &parent_name,
                parent_color,
                faction_index,
                faction_provinces,
                &faction_armies,
                &parent_resources,
                parent_population,
                num_factions,
            );
            faction_ids.push(faction_id);
            faction_nation_ids.push(faction_nation_id);
        }

        // Set all factions at war with each other
        self.initiate_civil_war(world, &faction_nation_ids);

        // Mark parent nation as factionalized and add civil war state
        world.entity_mut(parent_entity).insert(Factionalized {
            collapse_tick: 0, // TODO: Add tick tracking
            num_factions,
        });
        world.entity_mut(parent_entity).insert(CivilWarState {
            start_tick: 0, // TODO: Add tick tracking
            num_factions,
            faction_ids: faction_ids.clone(),
        });

        info!(
            parent_nation = %parent_name,
            num_factions = num_factions,
            "Nation successfully factionalized"
        );
    }

    /// Spawn a single faction entity
    fn spawn_faction(
        &mut self,
        world: &mut World,
        parent_nation_id: NationId,
        parent_name: &str,
        parent_color: [u8; 3],
        faction_index: usize,
        faction_provinces: &[ProvinceId],
        faction_armies: &[ArmyId],
        parent_resources: &Resources,
        _parent_population: u64,
        total_factions: usize,
    ) -> (FactionId, NationId) {
        let faction_id = FactionId::new();
        
        // Generate faction name
        let faction_name = format!("{} Faction {}", parent_name, faction_index + 1);
        
        // Get RNG and generate color and legitimacy
        let (faction_color, initial_legitimacy) = {
            let rng = world.resource::<DeterministicRng>();
            let color = vary_color(parent_color, faction_index, rng);
            let legitimacy = 10.0 + rng.next_f64() * 10.0;
            (color, legitimacy)
        };

        // Calculate proportional resources (equal split)
        let faction_resources = scale_resources(parent_resources, 1.0 / total_factions as f64);

        // Spawn faction entity with all required components
        let faction_entity = world.spawn((
            Faction {
                id: faction_id,
                name: faction_name.clone(),
                parent_nation_id,
                color: faction_color,
                formation_tick: 0, // TODO: Add tick tracking
            },
            Nation {
                id: NationId::new(), // New NationId for the faction
                name: faction_name.clone(),
                color: faction_color,
            },
            Legitimacy::new(initial_legitimacy),
            faction_resources,
            GDP::default(),
            WarState::default(),
            EconomicStress::default(),
            CasualtyLog::default(),
            AllianceCrisisLog::default(),
            DiplomaticIsolationLog::default(),
            FactionParent {
                original_nation_id: parent_nation_id,
            },
        )).id();

        // Store faction's nation ID before reassigning
        let faction_nation_id = world.get::<Nation>(faction_entity).unwrap().id;

        // Update province ownership
        self.reassign_provinces(world, faction_provinces, faction_entity);

        // Reassign armies to faction
        self.reassign_armies(world, faction_armies, faction_nation_id);

        debug!(
            faction_name = %faction_name,
            provinces = faction_provinces.len(),
            armies = faction_armies.len(),
            legitimacy = initial_legitimacy,
            "Faction spawned"
        );

        (faction_id, faction_nation_id)
    }

    /// Gather all province IDs owned by a nation
    fn gather_nation_provinces(&self, world: &mut World, nation_id: NationId) -> Vec<ProvinceId> {
        let mut provinces = Vec::new();
        let mut query = world.query::<(&Province, &OwnedBy)>();
        for (province, owner) in query.iter(world) {
            if owner.nation_id == nation_id {
                provinces.push(province.id);
            }
        }
        provinces
    }

    /// Gather total resources from all provinces owned by nation
    fn gather_nation_resources(&self, world: &mut World, nation_id: NationId) -> Resources {
        let mut total = Resources::default();
        let mut query = world.query::<(&OwnedBy, &Resources)>();
        for (owner, resources) in query.iter(world) {
            if owner.nation_id == nation_id {
                total.food += resources.food;
                total.iron += resources.iron;
                total.oil += resources.oil;
                total.rare_earths += resources.rare_earths;
                total.water += resources.water;
                total.trade_ports += resources.trade_ports;
            }
        }
        total
    }

    /// Count total population of nation
    fn count_nation_population(&self, world: &mut World, nation_id: NationId) -> u64 {
        let mut total = 0;
        let mut query = world.query::<(&OwnedBy, &Population)>();
        for (owner, population) in query.iter(world) {
            if owner.nation_id == nation_id {
                total += population.total;
            }
        }
        total
    }

    /// Reassign province ownership to a faction
    fn reassign_provinces(&self, world: &mut World, provinces: &[ProvinceId], faction_entity: Entity) {
        let faction_nation_id = world.get::<Nation>(faction_entity).unwrap().id;
        
        let mut query = world.query::<(&Province, &mut OwnedBy)>();
        for (province, mut owner) in query.iter_mut(world) {
            if provinces.contains(&province.id) {
                owner.nation_id = faction_nation_id;
            }
        }
    }

    /// Gather all armies owned by a nation
    fn gather_nation_armies(&self, world: &mut World, nation_id: NationId) -> Vec<ArmyId> {
        let mut armies = Vec::new();
        let mut query = world.query::<&Army>();
        for army in query.iter(world) {
            if army.owner == nation_id {
                armies.push(army.army_id);
            }
        }
        armies
    }

    /// Reassign armies to a faction
    fn reassign_armies(&self, world: &mut World, armies: &[ArmyId], faction_nation_id: NationId) {
        let mut query = world.query::<&mut Army>();
        for mut army in query.iter_mut(world) {
            if armies.contains(&army.army_id) {
                army.owner = faction_nation_id;
            }
        }
    }

    /// Set all factions at war with each other (civil war)
    fn initiate_civil_war(&self, world: &mut World, faction_nation_ids: &[NationId]) {
        // Build war declarations: each faction is at war with all other factions
        let mut war_declarations: Vec<(NationId, Vec<NationId>)> = Vec::new();
        
        for faction_id in faction_nation_ids {
            let enemies: Vec<NationId> = faction_nation_ids
                .iter()
                .filter(|&&id| id != *faction_id)
                .copied()
                .collect();
            war_declarations.push((*faction_id, enemies));
        }

        // Apply war declarations to WarState components
        let mut query = world.query::<(&Nation, &mut WarState)>();
        for (nation, mut war_state) in query.iter_mut(world) {
            if let Some((_, enemies)) = war_declarations.iter().find(|(id, _)| id == &nation.id) {
                war_state.at_war_with = enemies.clone();
                info!(
                    faction = ?nation.id,
                    enemies = enemies.len(),
                    "Faction civil war initiated"
                );
            }
        }
    }
}

impl TickPhase for FactionCivilWarPhase {
    fn name(&self) -> &str {
        "FactionCivilWar"
    }

    fn execute(&mut self, world: &mut World) {
        // Detect and spawn factions for nations with legitimacy = 0
        self.detect_and_spawn_collapses(world);

        debug!("FactionCivilWarPhase complete");
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Calculate number of factions based on province count
/// 1-3 provinces: 2 factions
/// 4-7 provinces: 3 factions
/// 8+ provinces: 4 factions
fn calculate_faction_count(province_count: usize) -> usize {
    if province_count <= 3 {
        2
    } else if province_count <= 7 {
        3
    } else {
        4
    }
}

/// Deterministically split provinces into N groups
fn split_provinces(
    provinces: &[ProvinceId],
    num_factions: usize,
    rng: &DeterministicRng,
) -> Vec<Vec<ProvinceId>> {
    let mut remaining: Vec<ProvinceId> = provinces.to_vec();
    let mut splits: Vec<Vec<ProvinceId>> = vec![Vec::new(); num_factions];

    // Shuffle provinces deterministically
    for i in (1..remaining.len()).rev() {
        let j = rng.next_usize(i + 1);
        remaining.swap(i, j);
    }

    // Distribute provinces round-robin
    for (idx, province_id) in remaining.iter().enumerate() {
        splits[idx % num_factions].push(*province_id);
    }

    splits
}

/// Deterministically split armies into N groups
fn split_armies(
    armies: &[ArmyId],
    num_factions: usize,
    rng: &DeterministicRng,
) -> Vec<Vec<ArmyId>> {
    if armies.is_empty() {
        return vec![Vec::new(); num_factions];
    }

    let mut remaining: Vec<ArmyId> = armies.to_vec();
    let mut splits: Vec<Vec<ArmyId>> = vec![Vec::new(); num_factions];

    // Shuffle armies deterministically
    for i in (1..remaining.len()).rev() {
        let j = rng.next_usize(i + 1);
        remaining.swap(i, j);
    }

    // Distribute armies round-robin
    for (idx, army_id) in remaining.iter().enumerate() {
        splits[idx % num_factions].push(*army_id);
    }

    splits
}

/// Scale resources by a factor
fn scale_resources(resources: &Resources, factor: f64) -> Resources {
    Resources {
        food: resources.food * factor,
        iron: resources.iron * factor,
        oil: resources.oil * factor,
        rare_earths: resources.rare_earths * factor,
        water: resources.water * factor,
        trade_ports: ((resources.trade_ports as f64) * factor).max(1.0) as u32,
    }
}

/// Vary color slightly for visual distinction (deterministic)
fn vary_color(base: [u8; 3], _index: usize, rng: &DeterministicRng) -> [u8; 3] {
    let variation = 30;
    [
        base[0].saturating_add((rng.next_u32() % variation) as u8),
        base[1].saturating_add((rng.next_u32() % variation) as u8),
        base[2].saturating_add((rng.next_u32() % variation) as u8),
    ]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::world::WorldState;

    #[test]
    fn test_collapse_detection() {
        let mut world_state = WorldState::new(42);
        
        // Spawn nation with legitimacy = 0
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        world_state.world.entity_mut(nation).insert(Legitimacy::new(0.0));
        
        // Spawn a province for the nation
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        let _province = world_state.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "Test Province".to_string(),
                position: glam::Vec2::ZERO,
                dominant_resource: crate::core::types::ResourceType::Food,
            },
            OwnedBy { nation_id },
            Population::default(),
            Resources::default(),
        )).id();
        
        // Run faction phase
        let mut phase = FactionCivilWarPhase::new();
        phase.execute(&mut world_state.world);
        
        // Verify nation is marked as factionalized
        let factionalized = world_state.world.get::<Factionalized>(nation);
        assert!(factionalized.is_some(), "Nation should be marked as factionalized");
        assert_eq!(factionalized.unwrap().num_factions, 2, "Should spawn 2 factions for small nation");
    }

    #[test]
    fn test_province_splitting_deterministic() {
        let provinces: Vec<ProvinceId> = (0..6).map(|_| ProvinceId::new()).collect();
        let rng1 = DeterministicRng::new(42);
        let rng2 = DeterministicRng::new(42);
        
        let split1 = split_provinces(&provinces, 3, &rng1);
        let split2 = split_provinces(&provinces, 3, &rng2);
        
        // Verify determinism
        assert_eq!(split1.len(), 3);
        assert_eq!(split2.len(), 3);
        for i in 0..3 {
            assert_eq!(split1[i], split2[i], "Splits should be identical with same seed");
        }
    }

    #[test]
    fn test_faction_spawning() {
        let mut world_state = WorldState::new(99);
        
        // Spawn parent nation with 4 provinces
        let nation = world_state.spawn_nation("Parent Nation".to_string(), [100, 150, 200], false);
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        
        for i in 0..4 {
            world_state.world.spawn((
                Province {
                    id: ProvinceId::new(),
                    name: format!("Province {}", i),
                    position: glam::Vec2::ZERO,
                    dominant_resource: crate::core::types::ResourceType::Food,
                },
                OwnedBy { nation_id },
                Population { total: 1_000_000, growth_rate: 0.01 },
                Resources {
                    food: 500.0,
                    iron: 100.0,
                    oil: 100.0,
                    rare_earths: 20.0,
                    water: 200.0,
                    trade_ports: 2,
                },
            ));
        }
        
        // Set legitimacy to 0
        world_state.world.entity_mut(nation).insert(Legitimacy::new(0.0));
        
        // Run faction phase
        let mut phase = FactionCivilWarPhase::new();
        phase.execute(&mut world_state.world);
        
        // Count spawned factions
        let faction_count = world_state.world.query::<&Faction>().iter(&world_state.world).count();
        assert_eq!(faction_count, 3, "Should spawn 3 factions for 4 provinces");
        
        // Gather faction nation IDs first to avoid borrow checker issues
        let faction_nation_ids: Vec<NationId> = world_state.world
            .query::<(&Faction, &Nation)>()
            .iter(&world_state.world)
            .map(|(_, nation)| nation.id)
            .collect();
        
        // Verify each faction has provinces
        for faction_nation_id in faction_nation_ids {
            let province_count = world_state.world
                .query::<&OwnedBy>()
                .iter(&world_state.world)
                .filter(|owner| owner.nation_id == faction_nation_id)
                .count();
            assert!(province_count > 0, "Each faction should own at least one province");
        }
    }

    #[test]
    fn test_resource_distribution() {
        let base = Resources {
            food: 1000.0,
            iron: 500.0,
            oil: 300.0,
            rare_earths: 50.0,
            water: 800.0,
            trade_ports: 6,
        };
        
        let scaled = scale_resources(&base, 0.5);
        
        assert_eq!(scaled.food, 500.0);
        assert_eq!(scaled.iron, 250.0);
        assert_eq!(scaled.oil, 150.0);
        assert_eq!(scaled.rare_earths, 25.0);
        assert_eq!(scaled.water, 400.0);
        assert_eq!(scaled.trade_ports, 3);
    }

    #[test]
    fn test_faction_count_calculation() {
        assert_eq!(calculate_faction_count(1), 2);
        assert_eq!(calculate_faction_count(3), 2);
        assert_eq!(calculate_faction_count(4), 3);
        assert_eq!(calculate_faction_count(7), 3);
        assert_eq!(calculate_faction_count(8), 4);
        assert_eq!(calculate_faction_count(20), 4);
    }

    #[test]
    fn test_no_duplicate_collapse() {
        let mut world_state = WorldState::new(42);
        
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        
        // Spawn provinces
        for i in 0..3 {
            world_state.world.spawn((
                Province {
                    id: ProvinceId::new(),
                    name: format!("Province {}", i),
                    position: glam::Vec2::ZERO,
                    dominant_resource: crate::core::types::ResourceType::Food,
                },
                OwnedBy { nation_id },
                Population::default(),
                Resources::default(),
            ));
        }
        
        world_state.world.entity_mut(nation).insert(Legitimacy::new(0.0));
        
        // Run phase twice
        let mut phase = FactionCivilWarPhase::new();
        phase.execute(&mut world_state.world);
        
        let factions_after_first = world_state.world.query::<&Faction>().iter(&world_state.world).count();
        
        phase.execute(&mut world_state.world);
        
        let factions_after_second = world_state.world.query::<&Faction>().iter(&world_state.world).count();
        
        assert_eq!(factions_after_first, factions_after_second, 
            "Should not spawn duplicate factions on subsequent ticks");
    }

    #[test]
    fn test_army_splitting_deterministic() {
        let armies: Vec<ArmyId> = (0..6).map(|_| ArmyId::new()).collect();
        let rng1 = DeterministicRng::new(42);
        let rng2 = DeterministicRng::new(42);
        
        let split1 = split_armies(&armies, 3, &rng1);
        let split2 = split_armies(&armies, 3, &rng2);
        
        // Verify determinism
        assert_eq!(split1.len(), 3);
        assert_eq!(split2.len(), 3);
        for i in 0..3 {
            assert_eq!(split1[i], split2[i], "Army splits should be identical with same seed");
        }
    }

    #[test]
    fn test_army_splitting_empty() {
        let armies: Vec<ArmyId> = Vec::new();
        let rng = DeterministicRng::new(42);
        
        let splits = split_armies(&armies, 3, &rng);
        
        assert_eq!(splits.len(), 3);
        for split in splits {
            assert!(split.is_empty());
        }
    }

    #[test]
    fn test_faction_civil_war() {
        let mut world_state = WorldState::new(99);
        
        // Spawn parent nation with provinces and armies
        let nation = world_state.spawn_nation("Parent Nation".to_string(), [100, 150, 200], false);
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        
        // Add 4 provinces
        for i in 0..4 {
            world_state.world.spawn((
                Province {
                    id: ProvinceId::new(),
                    name: format!("Province {}", i),
                    position: glam::Vec2::ZERO,
                    dominant_resource: crate::core::types::ResourceType::Food,
                },
                OwnedBy { nation_id },
                Population::default(),
                Resources::default(),
            ));
        }
        
        // Add 3 armies
        for i in 0..3 {
            world_state.world.spawn(Army {
                army_id: ArmyId::new(),
                owner: nation_id,
                location: ProvinceId::new(),
                infantry: 10_000,
                armor: 1_000,
                artillery: 500,
                morale: 80.0,
                organization: 100.0,
                supply_state: 1.0,
                entrenchment: 0.0,
                movement_points: 100.0,
                destination: None,
            });
        }
        
        // Set legitimacy to 0
        world_state.world.entity_mut(nation).insert(Legitimacy::new(0.0));
        
        // Run faction phase
        let mut phase = FactionCivilWarPhase::new();
        phase.execute(&mut world_state.world);
        
        // Count spawned factions (should be 3 for 4 provinces)
        let faction_count = world_state.world.query::<&Faction>().iter(&world_state.world).count();
        assert_eq!(faction_count, 3, "Should spawn 3 factions");
        
        // Verify each faction is at war with all other factions
        let faction_nation_ids: Vec<NationId> = world_state.world
            .query::<(&Faction, &Nation)>()
            .iter(&world_state.world)
            .map(|(_, nation)| nation.id)
            .collect();
        
        for faction_nation_id in &faction_nation_ids {
            let mut query = world_state.world.query::<(&Nation, &WarState)>();
            let war_state = query
                .iter(&world_state.world)
                .find(|(nation, _)| nation.id == *faction_nation_id)
                .map(|(_, ws)| ws);
            
            assert!(war_state.is_some(), "Faction should have war state");
            let enemies = &war_state.unwrap().at_war_with;
            assert_eq!(enemies.len(), faction_count - 1, "Faction should be at war with all other factions");
            
            // Verify faction is at war with all other faction nations
            for other_faction_id in &faction_nation_ids {
                if other_faction_id != faction_nation_id {
                    assert!(enemies.contains(other_faction_id), "Faction should be at war with sibling faction");
                }
            }
        }
    }

    #[test]
    fn test_faction_armies_reassigned() {
        let mut world_state = WorldState::new(123);
        
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        
        // Add provinces
        for i in 0..4 {
            world_state.world.spawn((
                Province {
                    id: ProvinceId::new(),
                    name: format!("Province {}", i),
                    position: glam::Vec2::ZERO,
                    dominant_resource: crate::core::types::ResourceType::Food,
                },
                OwnedBy { nation_id },
                Population::default(),
                Resources::default(),
            ));
        }
        
        // Add 6 armies
        let mut army_ids = Vec::new();
        for _ in 0..6 {
            let army_id = ArmyId::new();
            army_ids.push(army_id);
            world_state.world.spawn(Army {
                army_id,
                owner: nation_id,
                location: ProvinceId::new(),
                infantry: 10_000,
                armor: 1_000,
                artillery: 500,
                morale: 80.0,
                organization: 100.0,
                supply_state: 1.0,
                entrenchment: 0.0,
                movement_points: 100.0,
                destination: None,
            });
        }
        
        // Collapse nation
        world_state.world.entity_mut(nation).insert(Legitimacy::new(0.0));
        let mut phase = FactionCivilWarPhase::new();
        phase.execute(&mut world_state.world);
        
        // Verify armies are now owned by factions (not parent nation)
        let armies_owned_by_parent = world_state.world
            .query::<&Army>()
            .iter(&world_state.world)
            .filter(|army| army.owner == nation_id)
            .count();
        
        assert_eq!(armies_owned_by_parent, 0, "No armies should remain with parent nation");
        
        // Verify all armies are owned by faction nations
        let faction_nation_ids: Vec<NationId> = world_state.world
            .query::<(&Faction, &Nation)>()
            .iter(&world_state.world)
            .map(|(_, nation)| nation.id)
            .collect();
        
        let total_armies = world_state.world.query::<&Army>().iter(&world_state.world).count();
        let armies_owned_by_factions = world_state.world
            .query::<&Army>()
            .iter(&world_state.world)
            .filter(|army| faction_nation_ids.contains(&army.owner))
            .count();
        
        assert_eq!(armies_owned_by_factions, total_armies, "All armies should be owned by factions");
        assert_eq!(total_armies, 6, "All 6 armies should still exist");
    }

    #[test]
    fn test_civil_war_state_added() {
        let mut world_state = WorldState::new(42);
        
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        
        // Add provinces
        for i in 0..3 {
            world_state.world.spawn((
                Province {
                    id: ProvinceId::new(),
                    name: format!("Province {}", i),
                    position: glam::Vec2::ZERO,
                    dominant_resource: crate::core::types::ResourceType::Food,
                },
                OwnedBy { nation_id },
                Population::default(),
                Resources::default(),
            ));
        }
        
        // Collapse nation
        world_state.world.entity_mut(nation).insert(Legitimacy::new(0.0));
        let mut phase = FactionCivilWarPhase::new();
        phase.execute(&mut world_state.world);
        
        // Verify CivilWarState component was added
        let civil_war_state = world_state.world.get::<CivilWarState>(nation);
        assert!(civil_war_state.is_some(), "Parent nation should have CivilWarState component");
        
        let state = civil_war_state.unwrap();
        assert_eq!(state.num_factions, 2, "Should have 2 factions for 3 provinces");
        assert_eq!(state.faction_ids.len(), 2, "Should track 2 faction IDs");
    }
}
