//! Combat subsystem
//!
//! Handles tactical battle resolution based on province-level combat.
//! Inspired by Spirits of Steel and Age of History mechanics.

use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet};
use tracing::debug;

use crate::core::tick::TickPhase;
use crate::core::types::{
    Army, ArmyId, BattleCasualties, BattleId, NationId, ProvinceBattle, ProvinceId, WarState,
};

const BASE_CASUALTY_RATE: f64 = 0.01;
const VICTORY_THRESHOLD: f64 = 3.0;  // 3:1 ratio for victory
const MAX_BATTLE_DURATION: u32 = 20;  // ticks

/// Combat phase - resolves province battles
pub struct CombatPhase;

impl CombatPhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for CombatPhase {
    fn name(&self) -> &str {
        "Combat"
    }
    
    fn execute(&mut self, world: &mut World) {
        debug!("CombatPhase: Starting");
        
        // 1. Identify provinces with active battles
        let active_battles = identify_battles(world);
        
        // 2. Resolve each battle
        for battle_id in active_battles {
            resolve_battle(world, battle_id);
        }
        
        // 3. Start new battles where armies meet
        detect_new_battles(world);
        
        debug!("CombatPhase: Complete");
    }
}

/// Identify all active battles
fn identify_battles(world: &mut World) -> Vec<BattleId> {
    let mut battles = Vec::new();
    
    let mut query = world.query::<&ProvinceBattle>();
    for battle in query.iter(world) {
        battles.push(battle.battle_id);
    }
    
    battles
}

/// Resolve a single battle for one tick
fn resolve_battle(_world: &mut World, _battle_id: BattleId) {
    // 1) Load battle
    let Some((battle_entity, mut battle)) = get_battle_snapshot(_world, _battle_id) else {
        return;
    };

    // 2) Calculate strengths with modifiers
    let attacker_raw = calculate_side_strength(_world, &battle.attackers);
    let defender_raw = calculate_side_strength(_world, &battle.defenders);

    let attacker_strength = attacker_raw * battle.weather_modifier;
    let defender_strength = defender_raw * battle.terrain_modifier * battle.weather_modifier;

    // 3) Resolve casualties for both sides
    let attacker_losses = calculate_casualties(defender_strength, attacker_strength, BASE_CASUALTY_RATE);
    let defender_losses = calculate_casualties(attacker_strength, defender_strength, BASE_CASUALTY_RATE);

    // 4) Apply casualties to armies
    apply_side_casualties(_world, &battle.attackers, &attacker_losses);
    apply_side_casualties(_world, &battle.defenders, &defender_losses);

    // 5) Update battle state
    battle.duration = battle.duration.saturating_add(1);
    add_casualties(&mut battle.attacker_casualties, &attacker_losses);
    add_casualties(&mut battle.defender_casualties, &defender_losses);

    // Recompute post-casualty strength for victory conditions
    let attacker_post = calculate_side_strength(_world, &battle.attackers);
    let defender_post = calculate_side_strength(_world, &battle.defenders);

    let end_battle = if defender_post <= 1.0 {
        true
    } else if attacker_post <= 1.0 {
        true
    } else if attacker_post >= defender_post * VICTORY_THRESHOLD {
        true
    } else if defender_post >= attacker_post * VICTORY_THRESHOLD {
        true
    } else {
        battle.duration >= MAX_BATTLE_DURATION
    };

    if end_battle {
        let _ = _world.despawn(battle_entity);
        debug!("Battle {:?} concluded after {} ticks", battle.battle_id, battle.duration);
    } else if let Some(mut live_battle) = _world.get_mut::<ProvinceBattle>(battle_entity) {
        *live_battle = battle;
    }
}

/// Detect new battles where opposing armies meet
fn detect_new_battles(_world: &mut World) {
    let existing_battle_provinces = existing_battle_provinces(_world);
    let war_map = war_relations(_world);
    let province_forces = armies_by_province(_world);

    for (province_id, forces) in province_forces {
        if existing_battle_provinces.contains(&province_id) {
            continue;
        }

        if let Some((attacker_nation, defender_nation)) = find_warring_pair(&forces, &war_map) {
            let mut attackers = Vec::new();
            let mut defenders = Vec::new();

            for (army_id, owner) in &forces {
                if *owner == attacker_nation {
                    attackers.push(*army_id);
                } else if *owner == defender_nation {
                    defenders.push(*army_id);
                }
            }

            if attackers.is_empty() || defenders.is_empty() {
                continue;
            }

            attackers.sort_by_key(|id| id.0);
            defenders.sort_by_key(|id| id.0);

            _world.spawn(ProvinceBattle {
                battle_id: BattleId::new(),
                province_id,
                tick_started: 0,
                duration: 0,
                attackers,
                defenders,
                attacker_casualties: BattleCasualties::default(),
                defender_casualties: BattleCasualties::default(),
                terrain_modifier: 1.0,
                weather_modifier: 1.0,
            });
        }
    }
}

/// Calculate total strength for a list of armies
fn calculate_side_strength(world: &mut World, army_ids: &[ArmyId]) -> f64 {
    let mut total_strength = 0.0;
    
    let mut query = world.query::<&Army>();
    for army in query.iter(world) {
        if army_ids.contains(&army.army_id) {
            total_strength += army.combat_strength();
        }
    }
    
    total_strength
}

/// Calculate casualties for one side
fn calculate_casualties(
    enemy_strength: f64,
    _own_strength: f64,
    base_rate: f64,
) -> BattleCasualties {
    let loss_rate = enemy_strength.sqrt() * base_rate;
    let total_casualties = (loss_rate * 100.0) as u64;
    
    // Distribute proportionally (60% infantry, 25% armor, 15% artillery)
    BattleCasualties {
        infantry_lost: (total_casualties as f64 * 0.6) as u64,
        armor_lost: (total_casualties as f64 * 0.25) as u64,
        artillery_lost: (total_casualties as f64 * 0.15) as u64,
    }
}

/// Apply casualties to an army
fn apply_casualties(army: &mut Army, casualties: &BattleCasualties) {
    army.infantry = army.infantry.saturating_sub(casualties.infantry_lost);
    army.armor = army.armor.saturating_sub(casualties.armor_lost);
    army.artillery = army.artillery.saturating_sub(casualties.artillery_lost);
    
    // Reduce morale and organization from combat
    army.morale = (army.morale - 2.0).max(0.0);
    army.organization = (army.organization - 5.0).max(0.0);
}

fn add_casualties(total: &mut BattleCasualties, delta: &BattleCasualties) {
    total.infantry_lost = total.infantry_lost.saturating_add(delta.infantry_lost);
    total.armor_lost = total.armor_lost.saturating_add(delta.armor_lost);
    total.artillery_lost = total.artillery_lost.saturating_add(delta.artillery_lost);
}

fn get_battle_snapshot(world: &mut World, battle_id: BattleId) -> Option<(Entity, ProvinceBattle)> {
    let mut query = world.query::<(Entity, &ProvinceBattle)>();
    query
        .iter(world)
        .find(|(_, battle)| battle.battle_id == battle_id)
        .map(|(entity, battle)| (entity, battle.clone()))
}

fn apply_side_casualties(world: &mut World, army_ids: &[ArmyId], casualties: &BattleCasualties) {
    if army_ids.is_empty() {
        return;
    }

    let id_set: HashSet<ArmyId> = army_ids.iter().copied().collect();
    let mut snapshot = Vec::<(Entity, ArmyId, u64)>::new();

    {
        let mut query = world.query::<(Entity, &Army)>();
        for (entity, army) in query.iter(world) {
            if id_set.contains(&army.army_id) {
                let total = army
                    .infantry
                    .saturating_add(army.armor)
                    .saturating_add(army.artillery)
                    .max(1);
                snapshot.push((entity, army.army_id, total));
            }
        }
    }

    if snapshot.is_empty() {
        return;
    }

    let total_force: u64 = snapshot.iter().map(|(_, _, total)| *total).sum::<u64>().max(1);

    for (entity, _, army_force) in snapshot {
        let share = army_force as f64 / total_force as f64;
        let split = BattleCasualties {
            infantry_lost: (casualties.infantry_lost as f64 * share) as u64,
            armor_lost: (casualties.armor_lost as f64 * share) as u64,
            artillery_lost: (casualties.artillery_lost as f64 * share) as u64,
        };

        if let Some(mut army) = world.get_mut::<Army>(entity) {
            apply_casualties(&mut army, &split);
        }
    }
}

fn existing_battle_provinces(world: &mut World) -> HashSet<ProvinceId> {
    let mut provinces = HashSet::new();
    let mut query = world.query::<&ProvinceBattle>();
    for battle in query.iter(world) {
        provinces.insert(battle.province_id);
    }
    provinces
}

fn war_relations(world: &mut World) -> HashMap<NationId, HashSet<NationId>> {
    let mut relations = HashMap::<NationId, HashSet<NationId>>::new();
    let mut query = world.query::<(&crate::core::types::Nation, &WarState)>();

    for (nation, war_state) in query.iter(world) {
        relations
            .entry(nation.id)
            .or_default()
            .extend(war_state.at_war_with.iter().copied());
    }

    relations
}

fn armies_by_province(world: &mut World) -> HashMap<ProvinceId, Vec<(ArmyId, NationId)>> {
    let mut map = HashMap::<ProvinceId, Vec<(ArmyId, NationId)>>::new();
    let mut query = world.query::<&Army>();

    for army in query.iter(world) {
        let size = army
            .infantry
            .saturating_add(army.armor)
            .saturating_add(army.artillery);
        if size == 0 {
            continue;
        }

        map.entry(army.location)
            .or_default()
            .push((army.army_id, army.owner));
    }

    map
}

fn find_warring_pair(
    forces: &[(ArmyId, NationId)],
    war_map: &HashMap<NationId, HashSet<NationId>>,
) -> Option<(NationId, NationId)> {
    let mut nations: Vec<NationId> = forces.iter().map(|(_, owner)| *owner).collect();
    nations.sort_by_key(|id| id.0);
    nations.dedup();

    for attacker in &nations {
        if let Some(enemies) = war_map.get(attacker) {
            for defender in &nations {
                if attacker != defender && enemies.contains(defender) {
                    return Some((*attacker, *defender));
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{Nation, WarState};
    use crate::core::WorldState;
    use glam::Vec2;

    #[test]
    fn test_calculate_casualties() {
        let casualties = calculate_casualties(1000.0, 500.0, BASE_CASUALTY_RATE);
        assert!(casualties.infantry_lost > 0);
        assert!(casualties.armor_lost > 0);
        assert!(casualties.artillery_lost > 0);
    }

    #[test]
    fn test_combat_phase_creation() {
        let _phase = CombatPhase::new();
        // Just ensure it compiles
    }

    #[test]
    fn test_detect_new_battle_for_warring_armies() {
        let mut world_state = WorldState::new(7);

        let n1_entity = world_state.spawn_nation("A".to_string(), [255, 0, 0], false);
        let n2_entity = world_state.spawn_nation("B".to_string(), [0, 0, 255], false);

        let nation_a = world_state.world.get::<Nation>(n1_entity).unwrap().id;
        let nation_b = world_state.world.get::<Nation>(n2_entity).unwrap().id;

        world_state.world.entity_mut(n1_entity).insert(WarState { at_war_with: vec![nation_b] });
        world_state.world.entity_mut(n2_entity).insert(WarState { at_war_with: vec![nation_a] });

        let province_entity = world_state.spawn_province(
            "Front".to_string(),
            Vec2::new(0.0, 0.0),
            crate::core::types::ResourceType::Iron,
            nation_a,
        );
        let province_id = world_state.world.get::<crate::core::types::Province>(province_entity).unwrap().id;

        world_state.world.spawn(Army {
            army_id: ArmyId::new(),
            owner: nation_a,
            location: province_id,
            ..Army::default()
        });
        world_state.world.spawn(Army {
            army_id: ArmyId::new(),
            owner: nation_b,
            location: province_id,
            ..Army::default()
        });

        detect_new_battles(&mut world_state.world);

        let mut query = world_state.world.query::<&ProvinceBattle>();
        let battle_count = query.iter(&world_state.world).count();
        assert_eq!(battle_count, 1);
    }

    #[test]
    fn test_resolve_battle_applies_losses() {
        let mut world_state = WorldState::new(9);

        let n1_entity = world_state.spawn_nation("A".to_string(), [255, 0, 0], false);
        let n2_entity = world_state.spawn_nation("B".to_string(), [0, 0, 255], false);

        let nation_a = world_state.world.get::<Nation>(n1_entity).unwrap().id;
        let nation_b = world_state.world.get::<Nation>(n2_entity).unwrap().id;

        let province_entity = world_state.spawn_province(
            "Front".to_string(),
            Vec2::new(0.0, 0.0),
            crate::core::types::ResourceType::Iron,
            nation_a,
        );
        let province_id = world_state.world.get::<crate::core::types::Province>(province_entity).unwrap().id;

        let army_a = Army {
            army_id: ArmyId::new(),
            owner: nation_a,
            location: province_id,
            infantry: 12_000,
            armor: 800,
            artillery: 600,
            ..Army::default()
        };
        let army_b = Army {
            army_id: ArmyId::new(),
            owner: nation_b,
            location: province_id,
            infantry: 10_000,
            armor: 700,
            artillery: 500,
            ..Army::default()
        };

        let army_a_id = army_a.army_id;
        let army_b_id = army_b.army_id;

        world_state.world.spawn(army_a);
        world_state.world.spawn(army_b);

        let battle_id = BattleId::new();
        world_state.world.spawn(ProvinceBattle {
            battle_id,
            province_id,
            tick_started: 0,
            duration: 0,
            attackers: vec![army_a_id],
            defenders: vec![army_b_id],
            attacker_casualties: BattleCasualties::default(),
            defender_casualties: BattleCasualties::default(),
            terrain_modifier: 1.0,
            weather_modifier: 1.0,
        });

        resolve_battle(&mut world_state.world, battle_id);

        let mut query = world_state.world.query::<&Army>();
        let mut found_a = false;
        let mut found_b = false;
        for army in query.iter(&world_state.world) {
            if army.army_id == army_a_id {
                found_a = true;
                assert!(army.infantry < 12_000 || army.armor < 800 || army.artillery < 600);
            }
            if army.army_id == army_b_id {
                found_b = true;
                assert!(army.infantry < 10_000 || army.armor < 700 || army.artillery < 500);
            }
        }

        assert!(found_a && found_b);
    }
}
