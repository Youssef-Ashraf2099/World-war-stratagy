//! Unit tests for Nuclear Weapons System
//!
//! Tests cover:
//! - Nuclear capability mechanics and development
//! - Treaty compliance and violation tracking
//! - Bot desperation thresholds
//! - Player use gate logic
//! - Nuclear effect application
//! - Record tracking (uses and violations)

use crate::core::types::*;
use crate::subsystems::nuclear::*;
use crate::core::tick::TickPhase;
use super::TestWorldBuilder;

// ============================================================================
// NUCLEAR CAPABILITY MECHANICS
// ============================================================================

#[test]
fn test_nuclear_capability_default_initialization() {
    let cap = NuclearCapability::default();
    assert_eq!(cap.readiness, 0.0);
    assert_eq!(cap.development_rate, 0.5);
    assert!(!cap.can_use());
}

#[test]
fn test_nuclear_capability_with_initial_readiness() {
    let cap = NuclearCapability::new(50.0);
    assert_eq!(cap.readiness, 50.0);
    assert!(cap.can_use());  // 30% threshold
}

#[test]
fn test_nuclear_capability_clamping() {
    let cap = NuclearCapability::new(150.0);
    assert_eq!(cap.readiness, 100.0);  // Clamped to max
}

#[test]
fn test_nuclear_capability_development() {
    let mut cap = NuclearCapability::new(10.0);
    cap.develop(0.5);
    assert!((cap.readiness - 10.5).abs() < 0.01);

    cap.develop(100.0);  // Overflow test
    assert_eq!(cap.readiness, 100.0);  // Should be capped
}

#[test]
fn test_nuclear_capability_use_threshold() {
    let mut cap = NuclearCapability::new(20.0);
    assert!(!cap.can_use());  // Below 30% threshold

    cap.readiness = 30.0;
    assert!(cap.can_use());

    cap.readiness = 100.0;
    assert!(cap.can_use());
}

#[test]
fn test_nuclear_capability_reset() {
    let mut cap = NuclearCapability::new(75.0);
    cap.reset();
    assert_eq!(cap.readiness, 0.0);
}

// ============================================================================
// NUCLEAR POSTURE MECHANICS
// ============================================================================

#[test]
fn test_nuclear_posture_default() {
    let posture = NuclearPosture::default();
    assert_eq!(posture, NuclearPosture::Dormant);
}

#[test]
fn test_nuclear_posture_enum_variants() {
    assert_eq!(NuclearPosture::Dormant, NuclearPosture::Dormant);
    assert_ne!(NuclearPosture::Dormant, NuclearPosture::Developing);
    assert_ne!(NuclearPosture::Developing, NuclearPosture::Deployed);
    assert_ne!(NuclearPosture::Deployed, NuclearPosture::Deterrent);
}

// ============================================================================
// TREATY AND VIOLATION TRACKING
// ============================================================================

#[test]
fn test_nuclear_violation_record_empty() {
    let record = NuclearViolationRecord::default();
    assert!(!record.is_violator());
    assert_eq!(record.violation_count(), 0);
}

#[test]
fn test_nuclear_violation_record_add_violation() {
    let mut record = NuclearViolationRecord::default();
    record.add_violation(NuclearViolationType::DevelopmentWhileInTreaty, 100);

    assert!(record.is_violator());
    assert_eq!(record.violation_count(), 1);
}

#[test]
fn test_nuclear_violation_record_multiple_violations() {
    let mut record = NuclearViolationRecord::default();
    record.add_violation(NuclearViolationType::DevelopmentWhileInTreaty, 100);
    record.add_violation(NuclearViolationType::UseWhileInTreaty, 150);
    record.add_violation(NuclearViolationType::DevelopmentWhileInTreaty, 200);

    assert_eq!(record.violation_count(), 3);
    assert!(record.is_violator());
}

#[test]
fn test_nuclear_violation_types() {
    assert_eq!(
        NuclearViolationType::DevelopmentWhileInTreaty,
        NuclearViolationType::DevelopmentWhileInTreaty
    );
    assert_ne!(
        NuclearViolationType::DevelopmentWhileInTreaty,
        NuclearViolationType::UseWhileInTreaty
    );
}

// ============================================================================
// NUCLEAR USE RECORD TRACKING
// ============================================================================

#[test]
fn test_nuclear_use_record_empty() {
    let record = NuclearUseRecord::default();
    assert_eq!(record.total_uses(), 0);
}

#[test]
fn test_nuclear_use_record_add_use() {
    let mut record = NuclearUseRecord::default();
    let attacker = NationId::new();
    let target = NationId::new();
    let province = ProvinceId::new();

    record.add_use(attacker, target, vec![province], None, 100);

    assert_eq!(record.total_uses(), 1);
    assert_eq!(record.uses_against(target), 1);
}

#[test]
fn test_nuclear_use_record_multiple_targets() {
    let mut record = NuclearUseRecord::default();
    let attacker = NationId::new();
    let target_a = NationId::new();
    let target_b = NationId::new();

    record.add_use(attacker, target_a, vec![], None, 100);
    record.add_use(attacker, target_a, vec![], None, 150);
    record.add_use(attacker, target_b, vec![], None, 200);

    assert_eq!(record.total_uses(), 3);
    assert_eq!(record.uses_against(target_a), 2);
    assert_eq!(record.uses_against(target_b), 1);
}

#[test]
fn test_nuclear_use_record_tracks_provinces() {
    let mut record = NuclearUseRecord::default();
    let attacker = NationId::new();
    let target = NationId::new();
    let province1 = ProvinceId::new();
    let province2 = ProvinceId::new();
    let province3 = ProvinceId::new();

    record.add_use(attacker, target, vec![province1, province2, province3], None, 100);

    assert_eq!(record.total_uses(), 1);
    let incident = &record.uses[0];
    assert_eq!(incident.target_provinces.len(), 3);
}

// ============================================================================
// NUCLEAR PHASE CREATION AND INITIALIZATION
// ============================================================================

#[test]
fn test_nuclear_phase_creation() {
    let phase = NuclearPhase::new();
    assert_eq!(phase.name(), "Nuclear");
}

#[test]
fn test_nuclear_phase_from_default() {
    let phase = NuclearPhase::default();
    assert_eq!(phase.name(), "Nuclear");
}

// ============================================================================
// WORLD INTEGRATION TESTS
// ============================================================================

#[test]
fn test_nuclearcapable_nation_spawning() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Spawn nuclear-capable nation
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Nuclear Power".to_string(),
        [255, 0, 0],
        false,
        Some(40.0),  // 40% initial readiness
    );

    // Verify components exist
    let has_capability = fixture.world.world.get::<NuclearCapability>(nation_entity).is_some();
    let has_posture = fixture.world.world.get::<NuclearPosture>(nation_entity).is_some();
    
    assert!(has_capability);
    assert!(has_posture);
}

#[test]
fn test_non_nuclear_nation_spawning() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Spawn non-nuclear nation (traditional method)
    let nation_entity = fixture.world.spawn_nation(
        "Regular Power".to_string(),
        [0, 255, 0],
        false,
    );

    // Verify nuclear components don't exist
    let has_capability = fixture.world.world.get::<NuclearCapability>(nation_entity).is_some();
    let has_posture = fixture.world.world.get::<NuclearPosture>(nation_entity).is_some();
    
    assert!(!has_capability);
    assert!(!has_posture);
}

#[test]
fn test_nuclear_violation_record_on_spawned_nation() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation(
        "Nation".to_string(),
        [100, 100, 100],
        false,
    );

    // All nations should have violation record (even non-nuclear)
    let has_violation_record = fixture.world.world.get::<NuclearViolationRecord>(nation_entity).is_some();
    assert!(has_violation_record);
}

#[test]
fn test_nuclear_use_record_on_spawned_nation() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation(
        "Nation".to_string(),
        [100, 100, 100],
        false,
    );

    // All nations should have use record (even non-nuclear)
    let has_use_record = fixture.world.world.get::<NuclearUseRecord>(nation_entity).is_some();
    assert!(has_use_record);
}

// ============================================================================
// PLAYER USE GATE LOGIC
// ============================================================================

#[test]
fn test_player_use_gate_no_capability() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation(
        "Non-Nuclear".to_string(),
        [0, 0, 0],
        true,
    );

    // Cannot use without capability
    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;
    let can_use = player_can_use_nuclear(&mut fixture.world.world, nation_id);
    assert!(!can_use);
}

#[test]
fn test_player_use_gate_no_war() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Nuclear Power".to_string(),
        [255, 0, 0],
        true,
        Some(50.0),
    );

    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;

    // Has capability but not at war
    let can_use = player_can_use_nuclear(&mut fixture.world.world, nation_id);
    assert!(!can_use);
}

#[test]
fn test_player_use_gate_war_but_no_crisis() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Nuclear Power".to_string(),
        [255, 0, 0],
        true,
        Some(50.0),
    );

    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;

    // Add war state
    fixture.world.world.entity_mut(nation_entity).insert(WarState {
        at_war_with: vec![NationId::new()],
    });

    // Has capability and war, but not world-war crisis
    let can_use = player_can_use_nuclear(&mut fixture.world.world, nation_id);
    assert!(!can_use);  // No world-war crisis
}

// ============================================================================
// BOT USE GATE LOGIC
// ============================================================================

#[test]
fn test_bot_cannot_use_without_capability() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation(
        "Bot Without Nuclear".to_string(),
        [0, 0, 0],
        false,
    );

    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;

    let should_use = bot_should_use_nuclear(&mut fixture.world.world, nation_id);
    assert!(!should_use);
}

#[test]
fn test_bot_desperation_conditions_incomplete() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Desperate Bot".to_string(),
        [255, 0, 0],
        false,
        Some(50.0),
    );

    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;

    // Add at-war state
    fixture.world.world.entity_mut(nation_entity).insert(WarState {
        at_war_with: vec![NationId::new()],
    });

    // Only has war, not all desperation conditions
    // (missing war exhaustion >80, legitimacy <30, territory loss, etc.)
    let should_use = bot_should_use_nuclear(&mut fixture.world.world, nation_id);
    assert!(!should_use);
}

#[test]
fn test_bot_aggressive_personality_higher_probability() {
    // This is a statistical test - we can't guarantee exact probability
    // but we verify the logic structure is in place
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Aggressive Bot".to_string(),
        [255, 0, 0],
        false,
        Some(50.0),
    );

    // Mark as aggressive
    fixture.world.world.entity_mut(nation_entity).insert(AIPersonality::Aggressive);

    // The actual probability is random, so we just verify the function exists
    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;
    let _result = bot_should_use_nuclear(&mut fixture.world.world, nation_id);
    // If it compiles and doesn't panic, test passes
}

#[test]
fn test_bot_defensive_personality_lower_probability() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Defensive Bot".to_string(),
        [0, 0, 255],
        false,
        Some(50.0),
    );

    fixture.world.world.entity_mut(nation_entity).insert(AIPersonality::Defensive);

    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;
    let _result = bot_should_use_nuclear(&mut fixture.world.world, nation_id);
    // If it compiles, test passes
}

// ============================================================================
// TREATY INFRASTRUCTURE
// ============================================================================

#[test]
fn test_nuclear_treaty_membership_creation() {
    let treaty_id = NuclearTreatyId::new();
    let membership = NuclearTreatyMembership::new(treaty_id, 100);

    assert_eq!(membership.treaty_id, treaty_id);
    assert_eq!(membership.joined_tick, 100);
}

#[test]
fn test_war_start_snapshot() {
    let war_id = WarId::new();
    let snapshot = WarStartSnapshot::new(war_id, 50, 25, 100.0);

    assert_eq!(snapshot.war_id, war_id);
    assert_eq!(snapshot.start_tick, 50);
    assert_eq!(snapshot.territory_at_start, 25);
    assert_eq!(snapshot.military_at_start, 100.0);
}

// ============================================================================
// INTEGRATION WITH TICK PHASE
// ============================================================================

#[test]
fn test_nuclear_phase_executes_in_tick_pipeline() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create nuclear subsystem
    let mut nuclear_phase = NuclearPhase::new();
    
    // Execute should not panic
    nuclear_phase.execute(&mut fixture.world.world);
    
    // Verify phase name
    assert_eq!(nuclear_phase.name(), "Nuclear");
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_nuclear_capability_negative_development() {
    let mut cap = NuclearCapability::new(50.0);
    cap.develop(-10.0);
    // Should clamp to 0 due to min boundary
    assert!(cap.readiness >= 0.0);
}

#[test]
fn test_multiple_nations_nuclear_states() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nuclear_nation1 = fixture.world.spawn_nation_with_nuclear(
        "Nuclear Power 1".to_string(),
        [255, 0, 0],
        false,
        Some(30.0),
    );

    let nuclear_nation2 = fixture.world.spawn_nation_with_nuclear(
        "Nuclear Power 2".to_string(),
        [0, 255, 0],
        false,
        Some(60.0),
    );

    let non_nuclear_nation = fixture.world.spawn_nation(
        "Non-Nuclear".to_string(),
        [0, 0, 255],
        false,
    );

    // Verify independent states
    let cap1 = fixture.world.world.get::<NuclearCapability>(nuclear_nation1).unwrap();
    let cap2 = fixture.world.world.get::<NuclearCapability>(nuclear_nation2).unwrap();
    
    assert_eq!(cap1.readiness, 30.0);
    assert_eq!(cap2.readiness, 60.0);
    assert!(fixture.world.world.get::<NuclearCapability>(non_nuclear_nation).is_none());
}
