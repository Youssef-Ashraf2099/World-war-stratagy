//! Vassalage subsystem
//!
//! Handles vassal relationships, tribute, and peaceful annexation.

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::core::tick::TickPhase;
use crate::core::types::{
    AutonomyLevel, CasusBelli, GDP, Legitimacy, MilitaryCapacity, Nation, NationId, Resources,
    Tick, VassalRelation, WarDeclaration, WarGoal, WarId, WarState,
};
use crate::subsystems::notifications::{
    create_vassalization_notification,
    create_vassal_rebellion_notification,
};

const LOYALTY_DECAY_RATE: f64 = 0.1;  // per tick if mistreated
const INDEPENDENCE_THRESHOLD: f64 = 20.0;

/// Vassalage phase - manages vassal relationships
pub struct VassalagePhase;

impl VassalagePhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for VassalagePhase {
    fn name(&self) -> &str {
        "Vassalage"
    }
    
    fn execute(&mut self, world: &mut World) {
        debug!("VassalagePhase: Starting");
        
        // 1. Transfer tribute from vassals to overlords
        transfer_tribute(world);
        
        // 2. Update loyalty levels
        update_loyalty(world);
        
        // 3. Check for independence wars
        check_independence_triggers(world);
        
        debug!("VassalagePhase: Complete");
    }
}

/// Transfer tribute from vassals to overlords
fn transfer_tribute(_world: &mut World) {
    let relations: Vec<VassalRelation> = {
        let mut query = _world.query::<&VassalRelation>();
        query.iter(_world).cloned().collect()
    };

    if relations.is_empty() {
        return;
    }

    #[derive(Default, Clone, Copy)]
    struct ResourceDelta {
        food: f64,
        iron: f64,
        oil: f64,
        rare_earths: f64,
        water: f64,
    }

    let resources_by_nation: HashMap<NationId, Resources> = {
        let mut map = HashMap::new();
        let mut query = _world.query::<(&Nation, &Resources)>();
        for (nation, resources) in query.iter(_world) {
            map.insert(nation.id, resources.clone());
        }
        map
    };

    let mut deltas: HashMap<NationId, ResourceDelta> = HashMap::new();

    for relation in relations {
        let Some(vassal_resources) = resources_by_nation.get(&relation.vassal) else {
            continue;
        };

        let tribute_rate = (relation.tribute_percentage / 100.0).clamp(0.0, 0.80);
        if tribute_rate <= 0.0 {
            continue;
        }

        let tribute = ResourceDelta {
            food: vassal_resources.food * tribute_rate,
            iron: vassal_resources.iron * tribute_rate,
            oil: vassal_resources.oil * tribute_rate,
            rare_earths: vassal_resources.rare_earths * tribute_rate,
            water: vassal_resources.water * tribute_rate,
        };

        let vassal_delta = deltas.entry(relation.vassal).or_default();
        vassal_delta.food -= tribute.food;
        vassal_delta.iron -= tribute.iron;
        vassal_delta.oil -= tribute.oil;
        vassal_delta.rare_earths -= tribute.rare_earths;
        vassal_delta.water -= tribute.water;

        let overlord_delta = deltas.entry(relation.overlord).or_default();
        overlord_delta.food += tribute.food;
        overlord_delta.iron += tribute.iron;
        overlord_delta.oil += tribute.oil;
        overlord_delta.rare_earths += tribute.rare_earths;
        overlord_delta.water += tribute.water;
    }

    let mut query = _world.query::<(&Nation, &mut Resources)>();
    for (nation, mut resources) in query.iter_mut(_world) {
        if let Some(delta) = deltas.get(&nation.id) {
            resources.food = (resources.food + delta.food).max(0.0);
            resources.iron = (resources.iron + delta.iron).max(0.0);
            resources.oil = (resources.oil + delta.oil).max(0.0);
            resources.rare_earths = (resources.rare_earths + delta.rare_earths).max(0.0);
            resources.water = (resources.water + delta.water).max(0.0);
        }
    }
}

/// Update loyalty for all vassal relationships
fn update_loyalty(world: &mut World) {
    let mut query = world.query::<&mut VassalRelation>();
    
    for mut vassal in query.iter_mut(world) {
        // High tribute reduces loyalty
        if vassal.tribute_percentage > 30.0 {
            vassal.loyalty = (vassal.loyalty - LOYALTY_DECAY_RATE).max(0.0);
        } else if vassal.tribute_percentage < 15.0 {
            // Low tribute increases loyalty
            vassal.loyalty = (vassal.loyalty + 0.05).min(100.0);
        }
        
        if vassal.loyalty < INDEPENDENCE_THRESHOLD {
            warn!(
                "Vassal {:?} has low loyalty: {:.1}",
                vassal.vassal, vassal.loyalty
            );
        }
    }
}

/// Check if any vassals should declare independence
fn check_independence_triggers(_world: &mut World) {
    let mut breakaway_relations: Vec<(Entity, VassalRelation)> = Vec::new();

    {
        let mut query = _world.query::<(Entity, &VassalRelation)>();
        for (entity, relation) in query.iter(_world) {
            if relation.loyalty < INDEPENDENCE_THRESHOLD {
                breakaway_relations.push((entity, relation.clone()));
            }
        }
    }

    for (relation_entity, relation) in breakaway_relations {
        if let Some(mut war_state) = find_war_state_mut(_world, relation.vassal) {
            if !war_state.at_war_with.contains(&relation.overlord) {
                war_state.at_war_with.push(relation.overlord);
            }
        }

        if let Some(mut war_state) = find_war_state_mut(_world, relation.overlord) {
            if !war_state.at_war_with.contains(&relation.vassal) {
                war_state.at_war_with.push(relation.vassal);
            }
        }

        _world.spawn(WarDeclaration {
            war_id: WarId::new(),
            aggressor: relation.vassal,
            defender: relation.overlord,
            casus_belli: CasusBelli::PreemptiveStrike,
            war_goal: WarGoal::Total,
            declared_tick: 0,
        });

        // Create notification for rebellion
        create_vassal_rebellion_notification(_world, relation.vassal, relation.overlord, 0);

        _world.despawn(relation_entity);

        warn!(
            "Vassal {:?} declared independence war against {:?}",
            relation.vassal, relation.overlord
        );
    }
}

/// Offer vassalization to a target nation
pub fn offer_vassalization(
    world: &mut World,
    overlord: NationId,
    target: NationId,
    tribute_percentage: f64,
    _autonomy: AutonomyLevel,
    current_tick: Tick,
) -> bool {
    let accepted = evaluate_vassal_offer(world, overlord, target, tribute_percentage);
    
    if accepted {
        world.spawn(VassalRelation {
            overlord,
            vassal: target,
            tribute_percentage,
            established_tick: current_tick,
            loyalty: 50.0,
        });
        
        // Create notification for both nations
        create_vassalization_notification(world, overlord, target, current_tick);
        
        info!(
            "Vassalization accepted: {:?} → {:?}",
            target, overlord
        );
    }
    
    accepted
}

/// Evaluate whether a nation accepts vassalage
fn evaluate_vassal_offer(
    world: &mut World,
    overlord: NationId,
    target: NationId,
    tribute: f64,
) -> bool {
    let Some(overlord_power) = find_military_power(world, overlord) else {
        return false;
    };
    let Some(target_power) = find_military_power(world, target) else {
        return false;
    };

    let power_ratio = overlord_power / target_power.max(1.0);
    let target_at_war = is_nation_at_war(world, target);

    power_ratio >= 5.0 || (target_at_war && power_ratio >= 2.0) || tribute <= 10.0
}

/// Attempt to annex a vassal or weak nation
pub fn attempt_annexation(
    world: &mut World,
    annexer: NationId,
    target: NationId,
    compensation: f64,
) -> bool {
    let accepted = evaluate_annexation(world, target, compensation);
    
    if accepted {
        execute_annexation(world, annexer, target);
        info!("Annexation successful: {:?} → {:?}", target, annexer);
    }
    
    accepted
}

/// Evaluate annexation offer
fn evaluate_annexation(_world: &mut World, _target: NationId, _compensation: f64) -> bool {
    let Some(target_legitimacy) = find_legitimacy(_world, _target) else {
        return false;
    };
    let Some(target_gdp) = find_gdp(_world, _target) else {
        return false;
    };

    target_legitimacy < 15.0 || _compensation > target_gdp * 5.0
}

/// Execute annexation (transfer provinces, merge)
fn execute_annexation(_world: &mut World, _annexer: NationId, _target: NationId) {
    let mut province_query = _world.query::<&mut crate::core::types::OwnedBy>();
    for mut owned_by in province_query.iter_mut(_world) {
        if owned_by.nation_id == _target {
            owned_by.nation_id = _annexer;
        }
    }

    let relations_to_remove: Vec<Entity> = {
        let mut relation_query = _world.query::<(Entity, &VassalRelation)>();
        relation_query
            .iter(_world)
            .filter(|(_, r)| r.vassal == _target || r.overlord == _target)
            .map(|(entity, _)| entity)
            .collect()
    };

    for relation_entity in relations_to_remove {
        _world.despawn(relation_entity);
    }

    let target_entity = {
        let mut nation_query = _world.query::<(Entity, &Nation)>();
        nation_query
            .iter(_world)
            .find(|(_, nation)| nation.id == _target)
            .map(|(entity, _)| entity)
    };

    if let Some(entity) = target_entity {
        _world.despawn(entity);
    }
}

fn find_military_power(world: &mut World, nation_id: NationId) -> Option<f64> {
    let mut query = world.query::<(&Nation, &MilitaryCapacity)>();
    query
        .iter(world)
        .find(|(nation, _)| nation.id == nation_id)
        .map(|(_, military)| military.value)
}

fn is_nation_at_war(world: &mut World, nation_id: NationId) -> bool {
    let mut query = world.query::<(&Nation, &WarState)>();
    query
        .iter(world)
        .find(|(nation, _)| nation.id == nation_id)
        .map(|(_, war_state)| !war_state.at_war_with.is_empty())
        .unwrap_or(false)
}

fn find_legitimacy(world: &mut World, nation_id: NationId) -> Option<f64> {
    let mut query = world.query::<(&Nation, &Legitimacy)>();
    query
        .iter(world)
        .find(|(nation, _)| nation.id == nation_id)
        .map(|(_, legitimacy)| legitimacy.value)
}

fn find_gdp(world: &mut World, nation_id: NationId) -> Option<f64> {
    let mut query = world.query::<(&Nation, &GDP)>();
    query
        .iter(world)
        .find(|(nation, _)| nation.id == nation_id)
        .map(|(_, gdp)| gdp.value)
}

fn find_war_state_mut<'a>(world: &'a mut World, nation_id: NationId) -> Option<bevy_ecs::world::Mut<'a, WarState>> {
    let mut nation_query = world.query::<(Entity, &Nation)>();
    let nation_entity = nation_query
        .iter(world)
        .find(|(_, nation)| nation.id == nation_id)
        .map(|(entity, _)| entity)?;

    world.get_mut::<WarState>(nation_entity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{OwnedBy, Province, ProvinceId, ResourceType};
    use crate::core::world::WorldState;
    use glam::Vec2;

    #[test]
    fn test_vassalage_phase_creation() {
        let _phase = VassalagePhase::new();
        // Ensure it compiles
    }

    #[test]
    fn test_high_tribute_reduces_loyalty() {
        let mut vassal = VassalRelation {
            overlord: NationId::new(),
            vassal: NationId::new(),
            tribute_percentage: 40.0,
            established_tick: 0,
            loyalty: 50.0,
        };
        
        let initial_loyalty = vassal.loyalty;
        
        // Simulate loyalty decay
        if vassal.tribute_percentage > 30.0 {
            vassal.loyalty = (vassal.loyalty - LOYALTY_DECAY_RATE).max(0.0);
        }
        
        assert!(vassal.loyalty < initial_loyalty);
    }

    #[test]
    fn test_transfer_tribute_moves_resources() {
        let mut world_state = WorldState::new(42);
        let overlord_entity = world_state.spawn_nation("Overlord".to_string(), [255, 0, 0], false);
        let vassal_entity = world_state.spawn_nation("Vassal".to_string(), [0, 255, 0], false);

        let overlord_id = world_state.world.get::<Nation>(overlord_entity).unwrap().id;
        let vassal_id = world_state.world.get::<Nation>(vassal_entity).unwrap().id;

        world_state.world.spawn(VassalRelation {
            overlord: overlord_id,
            vassal: vassal_id,
            tribute_percentage: 20.0,
            established_tick: 0,
            loyalty: 60.0,
        });

        let overlord_food_before = world_state.world.get::<Resources>(overlord_entity).unwrap().food;
        let vassal_food_before = world_state.world.get::<Resources>(vassal_entity).unwrap().food;

        transfer_tribute(&mut world_state.world);

        let overlord_food_after = world_state.world.get::<Resources>(overlord_entity).unwrap().food;
        let vassal_food_after = world_state.world.get::<Resources>(vassal_entity).unwrap().food;

        assert!(overlord_food_after > overlord_food_before);
        assert!(vassal_food_after < vassal_food_before);
    }

    #[test]
    fn test_independence_trigger_starts_war() {
        let mut world_state = WorldState::new(42);
        let overlord_entity = world_state.spawn_nation("Overlord".to_string(), [255, 0, 0], false);
        let vassal_entity = world_state.spawn_nation("Vassal".to_string(), [0, 255, 0], false);

        let overlord_id = world_state.world.get::<Nation>(overlord_entity).unwrap().id;
        let vassal_id = world_state.world.get::<Nation>(vassal_entity).unwrap().id;

        world_state.world.spawn(VassalRelation {
            overlord: overlord_id,
            vassal: vassal_id,
            tribute_percentage: 35.0,
            established_tick: 0,
            loyalty: 10.0,
        });

        check_independence_triggers(&mut world_state.world);

        let vassal_war_state = world_state.world.get::<WarState>(vassal_entity).unwrap();
        let overlord_war_state = world_state.world.get::<WarState>(overlord_entity).unwrap();

        assert!(vassal_war_state.at_war_with.contains(&overlord_id));
        assert!(overlord_war_state.at_war_with.contains(&vassal_id));
    }

    #[test]
    fn test_annexation_transfers_provinces() {
        let mut world_state = WorldState::new(42);
        let annexer_entity = world_state.spawn_nation("Annexer".to_string(), [255, 0, 0], false);
        let target_entity = world_state.spawn_nation("Target".to_string(), [0, 255, 0], false);

        let annexer_id = world_state.world.get::<Nation>(annexer_entity).unwrap().id;
        let target_id = world_state.world.get::<Nation>(target_entity).unwrap().id;

        world_state.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: "Target Province".to_string(),
                position: Vec2::new(0.0, 0.0),
                dominant_resource: ResourceType::Food,
            },
            OwnedBy { nation_id: target_id },
        ));

        execute_annexation(&mut world_state.world, annexer_id, target_id);

        let mut query = world_state.world.query::<&OwnedBy>();
        let owners: Vec<NationId> = query.iter(&world_state.world).map(|o| o.nation_id).collect();
        assert!(owners.iter().all(|owner| *owner == annexer_id));
    }
}
