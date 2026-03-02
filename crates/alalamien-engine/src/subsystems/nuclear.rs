//! Nuclear Weapons System
//!
//! Manages nuclear weapon development, treaty compliance, and use mechanics.
//! Includes bot AI desperation thresholds and personality-based probability logic.

use bevy_ecs::prelude::*;
use tracing::{debug, info};
use rand::Rng;

use crate::core::tick::TickPhase;
use crate::core::types::*;
use crate::subsystems::notifications::{
    create_nuclear_use_notification,
    create_nuclear_treaty_violation_notification,
    create_nuclear_capability_notification,
};

/// Nuclear weapons phase - manages treaty compliance, development, and use gates
pub struct NuclearPhase;

impl NuclearPhase {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NuclearPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl TickPhase for NuclearPhase {
    fn name(&self) -> &str {
        "Nuclear"
    }

    fn execute(&mut self, world: &mut World) {
        debug!("NuclearPhase: Starting");

        // Pass 1: Check treaty compliance
        check_treaty_compliance(world);

        // Pass 2: Develop nuclear capability (for non-violators)
        develop_capability(world);

        // Pass 3: Apply binary violation consequences
        apply_violation_consequences(world);

        debug!("NuclearPhase: Complete");
    }
}

/// Check if nations are in compliance with their treaty obligations
fn check_treaty_compliance(world: &mut World) {
    let tick = 0u64; // TODO: Add tick tracking from world resource

    let mut nations_to_check: Vec<(Entity, NationId, Option<NuclearTreatyId>)> = {
        let mut query = world.query::<(Entity, &Nation, Option<&NuclearTreatyMembership>)>();
        query
            .iter(world)
            .map(|(entity, nation, membership)| (entity, nation.id, membership.map(|m| m.treaty_id)))
            .collect()
    };

    for (entity, nation_id, treaty_id) in nations_to_check {
        if let Some(_treaty_id) = treaty_id {
            // Check if nation is developing (violating treaty)
            if let Some(posture) = world.get::<NuclearPosture>(entity) {
                if *posture != NuclearPosture::Dormant {
                    // Mark as violator
                    if let Some(mut violation_record) = world.get_mut::<NuclearViolationRecord>(entity) {
                        if !violation_record.is_violator() {
                            violation_record.add_violation(
                                NuclearViolationType::DevelopmentWhileInTreaty,
                                tick,
                            );
                            
                            // Create global notification for treaty violation
                            create_nuclear_treaty_violation_notification(world, nation_id, tick);
                            
                            info!("Nuclear violation recorded: {:?} developing while in treaty", nation_id);
                        }
                    }
                }
            }
        }
    }
}

/// Develop nuclear capability for nations not violating treaties
fn develop_capability(world: &mut World) {
    let _tick = 0u64; // TODO: Add tick tracking from world resource

    let mut entities_to_develop: Vec<(Entity, bool)> = {
        let mut query = world.query::<(
            Entity,
            &NuclearCapability,
            &NuclearPosture,
            &NuclearViolationRecord,
        )>();
        query
            .iter(world)
            .filter(|(_, cap, postured, _)| cap.readiness < 100.0 && **postured == NuclearPosture::Developing)
            .map(|(entity, _, _, violations)| (entity, violations.is_violator()))
            .collect()
    };

    for (entity, is_violator) in entities_to_develop {
        if !is_violator {
            // Development rate: 0.5% per tick
            if let Some(mut cap) = world.get_mut::<NuclearCapability>(entity) {
                let old_readiness = cap.readiness;
                cap.develop(0.5);
                
                // Check if just crossed the deployment threshold (30%)
                if cap.readiness >= 30.0 && old_readiness < 30.0 {
                    // Get nation ID for notification
                    if let Some(nation) = world.get::<Nation>(entity) {
                        let nation_id = nation.id;
                        create_nuclear_capability_notification(world, nation_id, _tick);
                        debug!("Entity {:?} nuclear capability reached deployment readiness", entity);
                    }
                }
            }
        }
    }
}

/// Apply binary violation consequences: sanctions + expulsion + penalties
fn apply_violation_consequences(world: &mut World) {
    let tick = 0u64; // TODO: Add tick tracking from world resource

    let violators: Vec<(Entity, NationId, NuclearTreatyId)> = {
        let mut query = world.query::<(
            Entity,
            &Nation,
            &NuclearViolationRecord,
            &NuclearTreatyMembership,
        )>();
        query
            .iter(world)
            .filter(|(_, _, violations, _)| violations.is_violator())
            .map(|(entity, nation, _, membership)| (entity, nation.id, membership.treaty_id))
            .collect()
    };

    for (entity, nation_id, _treaty_id) in violators {
        // Apply instant sanctions: legitimacy loss, GDP penalty, reputation crash
        if let Some(mut legitimacy) = world.get_mut::<Legitimacy>(entity) {
            legitimacy.modify(-20.0);
            debug!("Legitimacy penalty applied to {:?} for nuclear treaty violation", nation_id);
        }

        if let Some(mut gdp) = world.get_mut::<GDP>(entity) {
            gdp.value *= 0.90;  // -10% GDP
            debug!("GDP penalty applied to {:?} for nuclear treaty violation", nation_id);
        }

        // Crash reputation with all other nations
        let mut all_nations: Vec<NationId> = {
            let mut query = world.query::<&Nation>();
            query.iter(world).map(|n| n.id).collect()
        };

        for other_nation_id in all_nations {
            if other_nation_id != nation_id {
                let mut relations_query = world.query::<&mut DiplomaticRelation>();
                for mut relation in relations_query.iter_mut(world) {
                    if (relation.nation_a == nation_id && relation.nation_b == other_nation_id)
                        || (relation.nation_a == other_nation_id && relation.nation_b == nation_id)
                    {
                        relation.modify_reputation(-30.0);
                    }
                }
            }
        }

        // Remove from treaty (expulsion)
        world.entity_mut(entity).remove::<NuclearTreatyMembership>();

        // Force posture to Dormant
        if let Some(mut posture) = world.get_mut::<NuclearPosture>(entity) {
            *posture = NuclearPosture::Dormant;
        }

        // Spawn event (WorldEvent) for diplomacy phase to process
        spawn_nuclear_violation_event(world, nation_id, tick);

        info!(
            "Nuclear treaty violation enforced: {:?} expelled and sanctioned",
            nation_id
        );
    }
}

/// Spawn a nuclear violation event for other subsystems to react to
fn spawn_nuclear_violation_event(world: &mut World, nation_id: NationId, _tick: Tick) {
    // Event spawning deferred to integration with event system in later phase
    // For now, just log the violation
    debug!(
        "Nuclear violation event: {:?} treaty violation recorded (tick={})",
        nation_id, _tick
    );
}

// ============================================================================
// BOT AI DECISION MAKING
// ============================================================================

/// Check if a bot nation meets desperation threshold for nuclear use
pub fn bot_should_use_nuclear(world: &mut World, nation_id: NationId) -> bool {
    // Step 1: Check if nation can use (capability + war + crisis)
    let can_use = player_can_use_nuclear(world, nation_id);
    if !can_use {
        return false;
    }

    // Step 2: Check if all 4 desperation conditions are met
    let is_desperate = bot_is_desperate(world, nation_id);
    if !is_desperate {
        return false;
    }

    // Step 3: Roll probability based on personality
    let entity = {
        let mut query = world.query::<(Entity, &Nation)>();
        query
            .iter(world)
            .find(|(_, n)| n.id == nation_id)
            .map(|(e, _)| e)
    };

    if let Some(entity) = entity {
        let personality = world
            .get::<AIPersonality>(entity)
            .copied()
            .unwrap_or(AIPersonality::Balanced);

        let use_threshold = match personality {
            AIPersonality::Aggressive => 65.0,    // 65% chance
            AIPersonality::Balanced => 45.0,      // 45% chance
            AIPersonality::Defensive => 20.0,     // 20% chance
        };

        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0.0..100.0);

        let should_use = roll < use_threshold;
        debug!(
            "Bot {:?} nuclear decision: personality={:?}, threshold={}, roll={:.1}, use={}",
            nation_id, personality, use_threshold, roll, should_use
        );

        return should_use;
    }

    false
}

/// Check if a bot nation is in desperation (all 4 conditions)
fn bot_is_desperate(world: &mut World, nation_id: NationId) -> bool {
    let entity = {
        let mut query = world.query::<(Entity, &Nation)>();
        query
            .iter(world)
            .find(|(_, n)| n.id == nation_id)
            .map(|(e, _)| e)
    };

    if let None = entity {
        return false;
    }

    let entity = entity.unwrap();

    // Condition 1: War exhaustion > 80%
    let war_exhaustion = world
        .get::<crate::core::types::WarExhaustion>(entity)
        .map(|we| we.value)
        .unwrap_or(0.0);

    if war_exhaustion <= 80.0 {
        return false;
    }

    // Condition 2: Legitimacy < 30%
    let legitimacy = world
        .get::<Legitimacy>(entity)
        .map(|l| l.value)
        .unwrap_or(50.0);

    if legitimacy >= 30.0 {
        return false;
    }

    // Condition 3: Lost 70% of territory
    // (requires tracking initial territory - simplified check for now)
    let current_territory = {
        let mut query = world.query::<&OwnedBy>();
        query.iter(world).filter(|ob| ob.nation_id == nation_id).count()
    };

    // Assume initial territory was ~50 provinces (scale as needed)
    // Lost 70% means < 15 provinces remaining
    if current_territory >= 15 {
        return false;
    }

    // Condition 4: Casualties ~75% of original army
    let current_military = world
        .get::<MilitaryCapacity>(entity)
        .map(|m| m.value)
        .unwrap_or(100.0);

    // Assume original was ~100 (scale as needed)
    // 75% casualties = < 25 remaining
    if current_military >= 25.0 {
        return false;
    }

    // Condition 5: Allies abandoned or destroyed
    let is_isolated = {
        let mut query = world.query::<&Alliance>();
        !query
            .iter(world)
            .any(|a| a.members.contains(&nation_id) && a.cohesion > 20.0)
    };

    if !is_isolated {
        return false;
    }

    debug!(
        "Bot {:?} DESPERATE: exhaustion={:.1}, legitimacy={:.1}, territory={}, military={:.1}",
        nation_id, war_exhaustion, legitimacy, current_territory, current_military
    );

    true
}

// ============================================================================
// PLAYER USE GATE (NO DESPERATION REQUIRED)
// ============================================================================

/// Check if player can use nuclear (only needs war + world-war crisis)
pub fn player_can_use_nuclear(world: &mut World, nation_id: NationId) -> bool {
    // Check 1: Nation has nuclear capability
    let entity = {
        let mut query = world.query::<(Entity, &Nation)>();
        query
            .iter(world)
            .find(|(_, n)| n.id == nation_id)
            .map(|(e, _)| e)
    };

    if let None = entity {
        return false;
    }

    let entity = entity.unwrap();

    let has_capability = world.get::<NuclearCapability>(entity).is_some();
    if !has_capability {
        return false;
    }

    // Check 2: Nation is at war
    let war_state = world
        .get::<WarState>(entity)
        .map(|ws| !ws.at_war_with.is_empty())
        .unwrap_or(false);

    if !war_state {
        return false;
    }

    // Check 3: World-war crisis is active
    // (Crisis = 3+ independent alliances + 25%+ of world population at war)
    let is_crisis = is_world_war_crisis(world);

    is_crisis
}

/// Check if a world-war crisis is active
fn is_world_war_crisis(world: &mut World) -> bool {
    // Count independent alliances (cohesion > 20)
    let alliance_count = {
        let mut query = world.query::<&Alliance>();
        query
            .iter(world)
            .filter(|a| a.cohesion > 20.0 && a.member_count() >= 2)
            .count()
    };

    if alliance_count < 3 {
        return false;
    }

    // Count nations at war
    let total_nations = {
        let mut query = world.query::<&Nation>();
        query.iter(world).count()
    };

    let nations_at_war = {
        let mut query = world.query::<&WarState>();
        query
            .iter(world)
            .filter(|ws| !ws.at_war_with.is_empty())
            .count()
    };

    let war_ratio = nations_at_war as f64 / total_nations.max(1) as f64;

    war_ratio >= 0.25  // 25% of world population at war
}

// ============================================================================
// USE EFFECTS (NATION + PROVINCE)
// ============================================================================

/// Apply nuclear use effects: legitimacy, GDP, reputation, province damage
pub fn apply_nuclear_use_effects(
    world: &mut World,
    attacker_id: NationId,
    target_id: NationId,
    target_provinces: Vec<ProvinceId>,
    _tick: Tick,
) {
    let attacker_entity = {
        let mut query = world.query::<(Entity, &Nation)>();
        query
            .iter(world)
            .find(|(_, n)| n.id == attacker_id)
            .map(|(e, _)| e)
    };

    let target_entity = {
        let mut query = world.query::<(Entity, &Nation)>();
        query
            .iter(world)
            .find(|(_, n)| n.id == target_id)
            .map(|(e, _)| e)
    };

    if let (Some(attacker), Some(target)) = (attacker_entity, target_entity) {
        // Attacker: legitimacy loss + GDP loss + reputation crash
        if let Some(mut leg) = world.get_mut::<Legitimacy>(attacker) {
            leg.modify(-15.0);
        }

        if let Some(mut gdp) = world.get_mut::<GDP>(attacker) {
            gdp.value *= 0.95;  // -5% GDP
        }

        // Crash reputation with all non-allied nations
        {
            let mut relations_query = world.query::<&mut DiplomaticRelation>();
            for mut relation in relations_query.iter_mut(world) {
                if relation.nation_a == attacker_id && relation.nation_b != target_id {
                    relation.modify_reputation(-20.0);
                } else if relation.nation_b == attacker_id && relation.nation_a != target_id {
                    relation.modify_reputation(-20.0);
                }
            }
        }

        // Target: legitimacy loss + GDP loss + province damage
        if let Some(mut leg) = world.get_mut::<Legitimacy>(target) {
            leg.modify(-25.0);
        }

        if let Some(mut gdp) = world.get_mut::<GDP>(target) {
            gdp.value *= 0.90;  // -10% GDP
        }

        // Province damage: population -20%, infrastructure -15%, resources -40%
        for province_id in &target_provinces {
            let mut provinces_query = world.query::<(Entity, &Province)>();
            if let Some((province_entity, _)) = provinces_query
                .iter(world)
                .find(|(_, p)| p.id == *province_id)
            {
                // Population damage
                if let Some(mut pop) = world.get_mut::<Population>(province_entity) {
                    pop.total = (pop.total as f64 * 0.80) as u64;  // -20%
                }

                // Infrastructure damage
                if let Some(mut infra) = world.get_mut::<Infrastructure>(province_entity) {
                    infra.level = ((infra.level as f64 * 0.85) as u32).min(infra.max_level);  // -15%
                }

                // Resource damage
                if let Some(mut resources) = world.get_mut::<Resources>(province_entity) {
                    resources.food *= 0.60;          // -40%
                    resources.water *= 0.60;         // -40%
                    resources.oil *= 0.60;           // -40%
                    resources.rare_earths *= 0.60;   // -40%
                }
            }
        }

        // Record use in both nations' records
        if let Some(mut use_record) = world.get_mut::<NuclearUseRecord>(attacker) {
            use_record.add_use(attacker_id, target_id, target_provinces.clone(), None, _tick);
        }

        // Create global notification for nuclear weapon use (all nations should know)
        create_nuclear_use_notification(world, attacker_id, target_id, target_provinces.clone(), _tick);

        // Spawn nuclear use event for diplomacy hooks
        spawn_nuclear_use_event(world, attacker_id, target_id, _tick);

        info!(
            "Nuclear weapon used: {:?} attacked {:?}, {} provinces damaged",
            attacker_id,
            target_id,
            target_provinces.len()
        );
    }
}

/// Spawn a nuclear use event for diplomatic reactions
fn spawn_nuclear_use_event(world: &mut World, attacker_id: NationId, target_id: NationId, _tick: Tick) {
    // Event spawning deferred to integration with event system
    debug!(
        "Nuclear use event: {:?} deployed against {:?}",
        attacker_id, target_id
    );
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nuclear_phase_creation() {
        let _phase = NuclearPhase::new();
    }

    #[test]
    fn test_player_can_use_requires_capability() {
        let mut world = World::new();
        let nation_id = NationId::new();
        let nation = Nation {
            id: nation_id,
            name: "Test Nation".to_string(),
            color: [255, 0, 0],
        };
        let entity = world.spawn(nation).id();

        // No nuclear capability - should fail
        assert!(!player_can_use_nuclear(&mut world, nation_id));

        // Add capability
        world.entity_mut(entity).insert(NuclearCapability::new(50.0));

        // Still no war - should fail
        assert!(!player_can_use_nuclear(&mut world, nation_id));
    }

    #[test]
    fn test_bot_desperation_all_conditions() {
        let mut world = World::new();
        let nation_id = NationId::new();
        let nation = Nation {
            id: nation_id,
            name: "Desperate Bot".to_string(),
            color: [255, 0, 0],
        };
        let entity = world.spawn(nation).id();

        // Add all bad-state components
        world
            .entity_mut(entity)
            .insert(crate::core::types::WarExhaustion { value: 85.0 })
            .insert(Legitimacy::new(20.0))
            .insert(MilitaryCapacity { value: 20.0 })
            .insert(AIPersonality::Aggressive);

        // Need at least 1 province to form basis, but we'll simulate less than 15
        // (in real test with province setup)

        // Without full setup, this is a partial test
        // but shows the logic structure
    }

    #[test]
    fn test_nuclear_capability_development() {
        let mut cap = NuclearCapability::new(10.0);
        assert_eq!(cap.readiness, 10.0);

        cap.develop(0.5);
        assert!((cap.readiness - 10.5).abs() < 0.01);

        // Test max cap
        cap.readiness = 99.5;
        cap.develop(1.0);
        assert_eq!(cap.readiness, 100.0);
    }

    #[test]
    fn test_violation_record_tracking() {
        let mut record = NuclearViolationRecord::default();
        assert!(!record.is_violator());

        record.add_violation(NuclearViolationType::DevelopmentWhileInTreaty, 100);
        assert!(record.is_violator());
        assert_eq!(record.violation_count(), 1);
    }

    #[test]
    fn test_use_record_tracking() {
        let mut record = NuclearUseRecord::default();
        assert_eq!(record.total_uses(), 0);

        let attacker = NationId::new();
        let target = NationId::new();
        let province = ProvinceId::new();

        record.add_use(attacker, target, vec![province], None, 100);
        assert_eq!(record.total_uses(), 1);
        assert_eq!(record.uses_against(target), 1);
    }
}
