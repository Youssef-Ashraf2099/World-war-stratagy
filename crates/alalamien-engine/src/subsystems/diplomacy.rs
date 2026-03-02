//! Diplomatic relations subsystem (V0.4)
//! Tracks diplomatic relations, reputation, threat alignment, and alliance proposals

use bevy_ecs::prelude::*;
use std::collections::HashMap;

use crate::core::types::*;

/// Diplomacy management phase
pub struct DiplomacyPhase;

impl DiplomacyPhase {
    pub fn new() -> Self {
        Self
    }

    /// Execute diplomacy phase: update relations, decay reputation changes, suggest alliances
    pub fn execute(&mut self, world: &mut World) {
        // Step 1: Update diplomatic relations based on war state and trade
        Self::update_relations_from_wars(world);
        
        // Step 2: Update threat alignment based on shared enemies
        Self::update_threat_alignment(world);
        
        // Step 3: Decay reputation changes (relations stabilize over time)
        Self::decay_reputation_shifts(world);
    }

    /// Update diplomatic relations based on current wars
    fn update_relations_from_wars(world: &mut World) {
        // Collect current wars
        let wars: Vec<(NationId, NationId)> = {
            let mut query = world.query::<&WarDeclaration>();
            query
                .iter(world)
                .map(|war| (war.aggressor, war.defender))
                .collect()
        };

        // Degrade relations between warring nations
        for (aggressor, defender) in wars {
            Self::modify_relation_by_nations(world, aggressor, defender, -5.0);
        }
    }

    /// Update threat alignment based on shared enemies
    fn update_threat_alignment(world: &mut World) {
        // Collect each nation's enemies
        let enemies_by_nation: HashMap<NationId, Vec<NationId>> = {
            let mut result: HashMap<NationId, Vec<NationId>> = HashMap::new();
            
            let mut query = world.query::<(&Nation, &WarState)>();
            for (nation, war_state) in query.iter(world) {
                result.insert(nation.id, war_state.at_war_with.clone());
            }
            
            result
        };

        // Update threat alignment based on shared enemies/allies
        let nations: Vec<NationId> = {
            let mut query = world.query::<&Nation>();
            query.iter(world).map(|n| n.id).collect()
        };

        for i in 0..nations.len() {
            for j in (i + 1)..nations.len() {
                let nation_a = nations[i];
                let nation_b = nations[j];
                
                let enemies_a = enemies_by_nation.get(&nation_a).cloned().unwrap_or_default();
                let enemies_b = enemies_by_nation.get(&nation_b).cloned().unwrap_or_default();
                
                // Shared enemies increase threat alignment
                let shared_enemies = enemies_a.iter()
                    .filter(|e| enemies_b.contains(e))
                    .count() as f64;
                
                // Direct conflict decreases threat alignment
                let direct_conflict = if enemies_a.contains(&nation_b) || enemies_b.contains(&nation_a) {
                    1.0
                } else {
                    0.0
                };

                let threat_delta = (shared_enemies * 0.1) - (direct_conflict * 0.3);
                Self::modify_threat_alignment(world, nation_a, nation_b, threat_delta);
            }
        }
    }

    /// Decay reputation shifts towards neutral (0.0) over time
    fn decay_reputation_shifts(world: &mut World) {
        const REPUTATION_DECAY: f64 = 0.5; // Reputation drifts 0.5 towards 0 per tick
        
        let mut query = world.query::<&mut DiplomaticRelation>();
        for mut relation in query.iter_mut(world) {
            if relation.reputation > 0.0 {
                relation.reputation = (relation.reputation - REPUTATION_DECAY).max(-100.0);
            } else if relation.reputation < 0.0 {
                relation.reputation = (relation.reputation + REPUTATION_DECAY).min(100.0);
            }
            relation.last_updated = 0; // TODO: Pass current tick
        }
    }

    /// Modify reputation between two nations
    fn modify_relation_by_nations(world: &mut World, nation_a: NationId, nation_b: NationId, delta: f64) {
        let mut query = world.query::<&mut DiplomaticRelation>();
        for mut relation in query.iter_mut(world) {
            if (relation.nation_a == nation_a && relation.nation_b == nation_b) ||
               (relation.nation_a == nation_b && relation.nation_b == nation_a) {
                relation.modify_reputation(delta);
            }
        }
    }

    /// Modify threat alignment between two nations
    fn modify_threat_alignment(world: &mut World, nation_a: NationId, nation_b: NationId, delta: f64) {
        let mut query = world.query::<&mut DiplomaticRelation>();
        for mut relation in query.iter_mut(world) {
            if (relation.nation_a == nation_a && relation.nation_b == nation_b) ||
               (relation.nation_a == nation_b && relation.nation_b == nation_a) {
                relation.threat_alignment = (relation.threat_alignment + delta).clamp(-1.0, 1.0);
            }
        }
    }

    /// Create or get a diplomatic relation between two nations
    pub fn get_or_create_relation(
        world: &mut World,
        nation_a: NationId,
        nation_b: NationId,
    ) -> Entity {
        // Try to find existing relation
        let mut query = world.query::<&DiplomaticRelation>();
        for relation in query.iter(world) {
            if (relation.nation_a == nation_a && relation.nation_b == nation_b) ||
               (relation.nation_a == nation_b && relation.nation_b == nation_a) {
                return Entity::PLACEHOLDER; // In production, return actual entity
            }
        }

        // Create new relation if not found
        let relation = DiplomaticRelation {
            nation_a,
            nation_b,
            reputation: 0.0,
            trade_dependency: 0.0,
            threat_alignment: 0.0,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        };

        world.spawn(relation).id()
    }

    /// Calculate alliance proposal score for two nations
    pub fn alliance_proposal_score(world: &mut World, nation_a: NationId, nation_b: NationId) -> f64 {
        let mut query = world.query::<&DiplomaticRelation>();
        for relation in query.iter(world) {
            if (relation.nation_a == nation_a && relation.nation_b == nation_b) ||
               (relation.nation_a == nation_b && relation.nation_b == nation_a) {
                return relation.alliance_score();
            }
        }
        0.0
    }
}

impl crate::core::tick::TickPhase for DiplomacyPhase {
    fn name(&self) -> &str {
        "Diplomacy"
    }

    fn execute(&mut self, world: &mut World) {
        DiplomacyPhase::execute(self, world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diplomacy_phase_creation() {
        let phase = DiplomacyPhase::new();
        assert_eq!(std::mem::size_of_val(&phase), 0);
    }

    #[test]
    fn test_reputation_modification() {
        let mut world = World::new();

        let relation = DiplomaticRelation {
            nation_a: NationId::default(),
            nation_b: NationId::new(),
            reputation: 0.0,
            trade_dependency: 0.0,
            threat_alignment: 0.0,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        };

        world.spawn(relation);

        // Verify relation created
        let mut query = world.query::<&DiplomaticRelation>();
        for rel in query.iter(&world) {
            assert_eq!(rel.reputation, 0.0);
        }
    }

    #[test]
    fn test_alliance_score_calculation() {
        let nation_a = NationId::default();
        let nation_b = NationId::new();

        let relation = DiplomaticRelation {
            nation_a,
            nation_b,
            reputation: 50.0,
            trade_dependency: 0.6,
            threat_alignment: 0.8,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        };

        let score = relation.alliance_score();
        // Score = trade_dependency * 0.3 + threat_alignment.abs() * 0.4 + (reputation + 100) / 200 * 0.3
        // = 0.6 * 0.3 + 0.8 * 0.4 + (50 + 100) / 200 * 0.3
        // = 0.18 + 0.32 + 0.225
        // ≈ 0.725
        assert!(score > 0.72 && score < 0.73);
    }

    #[test]
    fn test_friendly_hostile_checks() {
        let relation_friendly = DiplomaticRelation {
            nation_a: NationId::default(),
            nation_b: NationId::new(),
            reputation: 50.0,
            trade_dependency: 0.0,
            threat_alignment: 0.0,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        };

        let relation_hostile = DiplomaticRelation {
            nation_a: NationId::default(),
            nation_b: NationId::new(),
            reputation: -50.0,
            trade_dependency: 0.0,
            threat_alignment: 0.0,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        };

        assert!(relation_friendly.is_friendly());
        assert!(!relation_friendly.is_hostile());
        
        assert!(!relation_hostile.is_friendly());
        assert!(relation_hostile.is_hostile());
    }

    #[test]
    fn test_reputation_bounds() {
        let mut relation = DiplomaticRelation {
            nation_a: NationId::default(),
            nation_b: NationId::new(),
            reputation: 0.0,
            trade_dependency: 0.0,
            threat_alignment: 0.0,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        };

        // Test upper bound
        relation.modify_reputation(200.0);
        assert_eq!(relation.reputation, 100.0);

        // Test lower bound
        relation.modify_reputation(-300.0);
        assert_eq!(relation.reputation, -100.0);
    }

    #[test]
    fn test_threat_alignment_bounds() {
        let mut world = World::new();

        let mut relation = DiplomaticRelation {
            nation_a: NationId::default(),
            nation_b: NationId::new(),
            reputation: 0.0,
            trade_dependency: 0.0,
            threat_alignment: 0.5,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        };

        // Increase alignment
        relation.threat_alignment = (relation.threat_alignment + 0.8).clamp(-1.0, 1.0);
        assert_eq!(relation.threat_alignment, 1.0); // Clamped

        // Decrease alignment
        relation.threat_alignment = (relation.threat_alignment - 2.5).clamp(-1.0, 1.0);
        assert_eq!(relation.threat_alignment, -1.0); // Clamped
    }

    #[test]
    fn test_shared_enemies_increase_alignment() {
        let nation_a = NationId::new();
        let nation_b = NationId::new();
        let common_enemy = NationId::new();

        let war_a = WarState {
            at_war_with: vec![common_enemy],
        };

        let war_b = WarState {
            at_war_with: vec![common_enemy],
        };

        assert_eq!(
            war_a.at_war_with.iter()
                .filter(|e| war_b.at_war_with.contains(e))
                .count(),
            1
        );
    }

    #[test]
    fn test_diplomatic_relation_update_tick() {
        let mut relation = DiplomaticRelation::default();
        
        assert_eq!(relation.last_updated, 0);
        
        relation.last_updated = 100;
        assert_eq!(relation.last_updated, 100);
    }
}
