//! Advanced AI subsystem (V0.35)
//!
//! Adds strategic target selection, persistent memory, and intelligence-driven decisions.

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::core::tick::TickPhase;
use crate::core::types::{
    AIControlled, AIMemory, AIPersonality, CasusBelli, GDP, IntelligenceProfile, Legitimacy,
    MilitaryCapacity, Nation, NationId, ThreatRecord, WarDeclaration, WarGoal, WarId, WarState,
};

const AGGRESSION_WAR_THRESHOLD: f64 = 1.35;
const PEACE_EXHAUSTION_LEGITIMACY: f64 = 25.0;
const MAX_MEMORY_ENEMIES: usize = 12;
const MAX_KNOWN_THREATS: usize = 8;

/// V0.35 advanced AI phase.
pub struct AdvancedAIDecisionPhase;

impl AdvancedAIDecisionPhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for AdvancedAIDecisionPhase {
    fn name(&self) -> &str {
        "Advanced AI"
    }

    fn execute(&mut self, world: &mut World) {
        debug!("AdvancedAIDecisionPhase: Starting");

        let (entities_by_nation, ai_nations) = collect_ai_nations(world);
        let snapshots = collect_snapshots(world);

        for nation_id in ai_nations {
            let Some(snapshot) = snapshots.get(&nation_id) else {
                continue;
            };

            refresh_intelligence_and_memory(world, &entities_by_nation, &snapshots, snapshot);

            let personality = get_personality(world, nation_id);
            let decision = choose_decision(snapshot, personality, &snapshots);
            apply_decision(world, &entities_by_nation, &snapshots, nation_id, decision);
        }

        debug!("AdvancedAIDecisionPhase: Complete");
    }
}

#[derive(Debug, Clone)]
struct NationSnapshot {
    nation_id: NationId,
    legitimacy: f64,
    gdp: f64,
    military: f64,
    enemies: Vec<NationId>,
    memory_aggression_bias: f64,
    memory_peace_bias: f64,
    intel_quality: f64,
    known_threats: Vec<ThreatRecord>,
}

#[derive(Debug, Clone)]
enum AdvancedDecision {
    SeekPeace { enemy: NationId },
    DeclareWar { target: NationId },
    BuildMilitary,
    FocusEconomy,
}

fn collect_ai_nations(world: &mut World) -> (HashMap<NationId, Entity>, Vec<NationId>) {
    let mut entities_by_nation = HashMap::new();
    let mut ai_nations = Vec::new();

    let mut query = world.query::<(Entity, &Nation, &AIControlled)>();
    for (entity, nation, _) in query.iter(world) {
        entities_by_nation.insert(nation.id, entity);
        ai_nations.push(nation.id);
    }

    ai_nations.sort_by_key(|id| id.0);
    (entities_by_nation, ai_nations)
}

fn collect_snapshots(world: &mut World) -> HashMap<NationId, NationSnapshot> {
    let mut snapshots = HashMap::new();

    let mut query = world.query::<(
        &Nation,
        &Legitimacy,
        &GDP,
        &MilitaryCapacity,
        &WarState,
        Option<&AIMemory>,
        Option<&IntelligenceProfile>,
    )>();

    for (nation, legitimacy, gdp, military, war_state, memory, intel) in query.iter(world) {
        let memory = memory.cloned().unwrap_or_default();
        let intel = intel.cloned().unwrap_or_default();

        let aggression_bias = ((memory.successful_wars as i32 - memory.failed_wars as i32) as f64)
            * 0.05;
        let peace_bias = (memory.failed_wars as f64) * 0.05;

        snapshots.insert(
            nation.id,
            NationSnapshot {
                nation_id: nation.id,
                legitimacy: legitimacy.value,
                gdp: gdp.value,
                military: military.value,
                enemies: war_state.at_war_with.clone(),
                memory_aggression_bias: aggression_bias.clamp(-0.3, 0.3),
                memory_peace_bias: peace_bias.clamp(0.0, 0.4),
                intel_quality: intel.intel_quality.clamp(0.0, 1.0),
                known_threats: intel.known_threats,
            },
        );
    }

    snapshots
}

fn get_personality(world: &mut World, nation_id: NationId) -> AIPersonality {
    let mut query = world.query::<(&Nation, Option<&AIPersonality>)>();
    for (nation, personality) in query.iter(world) {
        if nation.id == nation_id {
            return personality.copied().unwrap_or(AIPersonality::Balanced);
        }
    }

    AIPersonality::Balanced
}

fn choose_decision(
    own: &NationSnapshot,
    personality: AIPersonality,
    snapshots: &HashMap<NationId, NationSnapshot>,
) -> AdvancedDecision {
    let peace_limit = PEACE_EXHAUSTION_LEGITIMACY + own.memory_peace_bias * 20.0;
    if own.legitimacy < peace_limit && !own.enemies.is_empty() {
        return AdvancedDecision::SeekPeace {
            enemy: own.enemies[0],
        };
    }

    if let Some(target) = choose_war_target(own, personality, snapshots) {
        return AdvancedDecision::DeclareWar { target };
    }

    if own.military < (own.gdp / 25_000.0) {
        return AdvancedDecision::BuildMilitary;
    }

    AdvancedDecision::FocusEconomy
}

fn choose_war_target(
    own: &NationSnapshot,
    personality: AIPersonality,
    snapshots: &HashMap<NationId, NationSnapshot>,
) -> Option<NationId> {
    let personality_bonus = match personality {
        AIPersonality::Defensive => 0.15,
        AIPersonality::Balanced => 0.0,
        AIPersonality::Aggressive => -0.15,
    };

    let intel_uncertainty = (1.0 - own.intel_quality) * 0.1;
    let threshold = AGGRESSION_WAR_THRESHOLD + personality_bonus - own.memory_aggression_bias + intel_uncertainty;

    let mut known_threat_score = HashMap::<NationId, f64>::new();
    for threat in &own.known_threats {
        known_threat_score.insert(threat.nation_id, threat.score);
    }

    let mut candidates: Vec<&NationSnapshot> = snapshots
        .values()
        .filter(|other| {
            other.nation_id != own.nation_id
                && !own.enemies.contains(&other.nation_id)
                && other.military > 0.0
        })
        .collect();

    candidates.sort_by(|a, b| {
        let ta = known_threat_score.get(&a.nation_id).copied().unwrap_or(0.0);
        let tb = known_threat_score.get(&b.nation_id).copied().unwrap_or(0.0);
        tb.partial_cmp(&ta).unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates
        .into_iter()
        .find(|target| {
            let ratio = own.military / target.military.max(1.0);
            let pressure = known_threat_score
                .get(&target.nation_id)
                .copied()
                .unwrap_or(0.0);
            ratio + pressure * 0.2 >= threshold && target.legitimacy < 60.0
        })
        .map(|target| target.nation_id)
}

fn apply_decision(
    world: &mut World,
    entities_by_nation: &HashMap<NationId, Entity>,
    snapshots: &HashMap<NationId, NationSnapshot>,
    nation_id: NationId,
    decision: AdvancedDecision,
) {
    match decision {
        AdvancedDecision::SeekPeace { enemy } => {
            remove_war_relation(world, entities_by_nation, nation_id, enemy);
            add_failed_war_memory(world, entities_by_nation, nation_id, enemy);
            info!("Advanced AI {:?} seeks peace with {:?}", nation_id, enemy);
        }
        AdvancedDecision::DeclareWar { target } => {
            declare_war(world, entities_by_nation, snapshots, nation_id, target);
        }
        AdvancedDecision::BuildMilitary => {
            if let Some(&entity) = entities_by_nation.get(&nation_id) {
                if let Some(mut military) = world.get_mut::<MilitaryCapacity>(entity) {
                    military.value += 5.0;
                }
                if let Some(mut legitimacy) = world.get_mut::<Legitimacy>(entity) {
                    legitimacy.value = (legitimacy.value - 0.1).max(0.0);
                }
            }
            debug!("Advanced AI {:?} builds military", nation_id);
        }
        AdvancedDecision::FocusEconomy => {
            if let Some(&entity) = entities_by_nation.get(&nation_id) {
                if let Some(mut gdp) = world.get_mut::<GDP>(entity) {
                    gdp.growth_rate = (gdp.growth_rate + 0.001).min(0.08);
                }
                if let Some(mut legitimacy) = world.get_mut::<Legitimacy>(entity) {
                    legitimacy.value = (legitimacy.value + 0.2).min(100.0);
                }
            }
            debug!("Advanced AI {:?} focuses economy", nation_id);
        }
    }
}

fn declare_war(
    world: &mut World,
    entities_by_nation: &HashMap<NationId, Entity>,
    snapshots: &HashMap<NationId, NationSnapshot>,
    aggressor: NationId,
    defender: NationId,
) {
    let Some(&aggressor_entity) = entities_by_nation.get(&aggressor) else {
        return;
    };
    let Some(&defender_entity) = entities_by_nation.get(&defender) else {
        return;
    };

    if let Some(mut war_state) = world.get_mut::<WarState>(aggressor_entity) {
        if !war_state.at_war_with.contains(&defender) {
            war_state.at_war_with.push(defender);
        }
    }
    if let Some(mut war_state) = world.get_mut::<WarState>(defender_entity) {
        if !war_state.at_war_with.contains(&aggressor) {
            war_state.at_war_with.push(aggressor);
        }
    }

    if let Some(mut legitimacy) = world.get_mut::<Legitimacy>(aggressor_entity) {
        legitimacy.value = (legitimacy.value - 1.5).max(0.0);
    }

    if let Some(mut memory) = world.get_mut::<AIMemory>(aggressor_entity) {
        if !memory.recent_enemies.contains(&defender) {
            memory.recent_enemies.push(defender);
        }
        if memory.recent_enemies.len() > MAX_MEMORY_ENEMIES {
            memory.recent_enemies.remove(0);
        }
    }

    let casus_belli = snapshots
        .get(&defender)
        .filter(|d| d.legitimacy < 30.0)
        .map(|_| CasusBelli::PreemptiveStrike)
        .unwrap_or(CasusBelli::ResourceConflict);

    world.spawn(WarDeclaration {
        war_id: WarId::new(),
        aggressor,
        defender,
        casus_belli,
        war_goal: WarGoal::Humiliate,
        declared_tick: 0,
    });

    info!("Advanced AI {:?} declares war on {:?}", aggressor, defender);
}

fn remove_war_relation(
    world: &mut World,
    entities_by_nation: &HashMap<NationId, Entity>,
    nation_id: NationId,
    enemy: NationId,
) {
    if let Some(&entity) = entities_by_nation.get(&nation_id) {
        if let Some(mut war_state) = world.get_mut::<WarState>(entity) {
            war_state.at_war_with.retain(|id| *id != enemy);
        }
    }

    if let Some(&enemy_entity) = entities_by_nation.get(&enemy) {
        if let Some(mut war_state) = world.get_mut::<WarState>(enemy_entity) {
            war_state.at_war_with.retain(|id| *id != nation_id);
        }
    }
}

fn refresh_intelligence_and_memory(
    world: &mut World,
    entities_by_nation: &HashMap<NationId, Entity>,
    snapshots: &HashMap<NationId, NationSnapshot>,
    own: &NationSnapshot,
) {
    let Some(&entity) = entities_by_nation.get(&own.nation_id) else {
        return;
    };

    if let Some(mut intel) = world.get_mut::<IntelligenceProfile>(entity) {
        let mut threats: Vec<ThreatRecord> = snapshots
            .values()
            .filter(|other| other.nation_id != own.nation_id)
            .map(|other| {
                let military_pressure = (other.military / own.military.max(1.0)).min(5.0);
                let legitimacy_pressure = ((60.0 - other.legitimacy).max(0.0)) / 60.0;
                ThreatRecord {
                    nation_id: other.nation_id,
                    score: military_pressure * 0.7 + legitimacy_pressure * 0.3,
                    last_updated: 0,
                }
            })
            .collect();

        threats.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        threats.truncate(MAX_KNOWN_THREATS);

        intel.known_threats = threats;
        intel.intel_quality = (intel.intel_quality + 0.002).min(1.0);
        intel.fog_strength = (1.0 - intel.intel_quality).clamp(0.0, 1.0);
    }

    if let Some(mut memory) = world.get_mut::<AIMemory>(entity) {
        memory.last_decision_tick = memory.last_decision_tick.saturating_add(1);
        for enemy in &own.enemies {
            if !memory.recent_enemies.contains(enemy) {
                memory.recent_enemies.push(*enemy);
            }
        }
        if memory.recent_enemies.len() > MAX_MEMORY_ENEMIES {
            let overflow = memory.recent_enemies.len() - MAX_MEMORY_ENEMIES;
            memory.recent_enemies.drain(0..overflow);
        }
    }
}

fn add_failed_war_memory(
    world: &mut World,
    entities_by_nation: &HashMap<NationId, Entity>,
    nation_id: NationId,
    enemy: NationId,
) {
    if let Some(&entity) = entities_by_nation.get(&nation_id) {
        if let Some(mut memory) = world.get_mut::<AIMemory>(entity) {
            memory.failed_wars = memory.failed_wars.saturating_add(1);
            if !memory.recent_enemies.contains(&enemy) {
                memory.recent_enemies.push(enemy);
            }
            if memory.recent_enemies.len() > MAX_MEMORY_ENEMIES {
                memory.recent_enemies.remove(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_ai_phase_creation() {
        let _phase = AdvancedAIDecisionPhase::new();
    }

    #[test]
    fn test_choose_decision_prefers_peace_in_crisis() {
        let own = NationSnapshot {
            nation_id: NationId::new(),
            legitimacy: 20.0,
            gdp: 1_000_000.0,
            military: 80.0,
            enemies: vec![NationId::new()],
            memory_aggression_bias: 0.0,
            memory_peace_bias: 0.2,
            intel_quality: 0.8,
            known_threats: Vec::new(),
        };

        let snapshots = HashMap::new();
        let decision = choose_decision(&own, AIPersonality::Aggressive, &snapshots);

        assert!(matches!(decision, AdvancedDecision::SeekPeace { .. }));
    }

    #[test]
    fn test_memory_aggression_bias_lowers_war_threshold() {
        let target_id = NationId::new();
        let own_id = NationId::new();

        let own = NationSnapshot {
            nation_id: own_id,
            legitimacy: 70.0,
            gdp: 2_000_000.0,
            military: 130.0,
            enemies: Vec::new(),
            memory_aggression_bias: 0.25,
            memory_peace_bias: 0.0,
            intel_quality: 1.0,
            known_threats: vec![ThreatRecord {
                nation_id: target_id,
                score: 1.0,
                last_updated: 0,
            }],
        };

        let target = NationSnapshot {
            nation_id: target_id,
            legitimacy: 50.0,
            gdp: 1_500_000.0,
            military: 100.0,
            enemies: Vec::new(),
            memory_aggression_bias: 0.0,
            memory_peace_bias: 0.0,
            intel_quality: 0.5,
            known_threats: Vec::new(),
        };

        let mut snapshots = HashMap::new();
        snapshots.insert(own_id, own.clone());
        snapshots.insert(target_id, target);

        let decision = choose_decision(&own, AIPersonality::Balanced, &snapshots);
        assert!(matches!(decision, AdvancedDecision::DeclareWar { .. }));
    }
}
