// Diplomacy subsystem unit tests
// Tests for diplomatic relations, reputation, and alliance mechanics

use super::super::TestWorldBuilder;
use crate::subsystems::diplomacy::*;
use crate::core::types::{NationId, DiplomaticRelation};

#[test]
fn test_diplomacy_phase_creation() {
    let phase = DiplomacyPhase::new();
    assert_eq!(std::mem::size_of_val(&phase), 0);
}

#[test]
fn test_diplomatic_relation_default() {
    let relation = DiplomaticRelation::default();
    assert_eq!(relation.reputation, 0.0);
    assert_eq!(relation.trade_dependency, 0.0);
    assert_eq!(relation.threat_alignment, 0.0);
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
fn test_diplomacy_world_integration() {
    // Test diplomacy subsystem through world integration
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(3)
        .build();
    
    fixture.execute_ticks(100);
    assert_eq!(fixture.current_tick(), 100);
}
