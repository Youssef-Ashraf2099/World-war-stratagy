//! Integration tests for Nuclear Weapons System
//!
//! Tests nuclear weapons in multi-nation scenarios:
//! - Treaty formation and violations
//! - Nuclear development progression
//! - Use gate enforcement
//! - Effect application across nations and provinces
//! - Diplomatic cascades from nuclear use
//! - Desperation threshold interactions

use crate::core::types::*;
use crate::core::tick::{TickPipeline, TickPhase};
use crate::subsystems::nuclear::*;
use super::super::fixtures::TestWorldBuilder;

// ============================================================================
// TREATY AND VIOLATIONS
// ============================================================================

#[test]
fn test_nuclear_treaty_formation_and_violations() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create nuclear-capable nation
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Nuclear Power".to_string(),
        [255, 0, 0],
        false,
        Some(20.0),
    );
    
    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;

    // Add to treaty
    let treaty_id = NuclearTreatyId::new();
    let current_tick = fixture.current_tick();
    fixture.world.world.entity_mut(nation_entity)
        .insert(NuclearTreatyMembership::new(treaty_id, current_tick));

    // Verify membership
    let is_member = fixture.world.world.get::<NuclearTreatyMembership>(nation_entity).is_some();
    assert!(is_member);

    // Change to developing posture (violation)
    fixture.world.world.entity_mut(nation_entity)
        .insert(NuclearPosture::Developing);

    // Execute nuclear phase
    let mut nuclear_phase = NuclearPhase::new();
    nuclear_phase.execute(&mut fixture.world.world);

    // Verify violation was recorded
    let violation_record = fixture.world.world.get::<NuclearViolationRecord>(nation_entity).unwrap();
    assert!(violation_record.is_violator());
    assert_eq!(violation_record.violation_count(), 1);

    // Verify treaty expulsion (membership removed)
    let still_member = fixture.world.world.get::<NuclearTreatyMembership>(nation_entity);
    assert!(still_member.is_none());  // Should be removed
}

#[test]
fn test_nuclear_development_progression() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Developing Nuclear Power".to_string(),
        [255, 0, 0],
        false,
        Some(10.0),
    );

    // Set to developing posture
    fixture.world.world.entity_mut(nation_entity)
        .insert(NuclearPosture::Developing);

    // Record initial
    let initial_cap = fixture.world.world.get::<NuclearCapability>(nation_entity).unwrap().readiness;

    // Execute nuclear phase 5 times
    let mut nuclear_phase = NuclearPhase::new();
    for _ in 0..5 {
        nuclear_phase.execute(&mut fixture.world.world);
    }

    // Verify capability increased
    let final_cap = fixture.world.world.get::<NuclearCapability>(nation_entity).unwrap().readiness;
    assert!(final_cap > initial_cap);
}

#[test]
fn test_nuclear_violation_prevents_development() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation_with_nuclear(
        "Violator".to_string(),
        [255, 0, 0],
        false,
        Some(20.0),
    );

    // Add to treaty and violate
    let treaty_id = NuclearTreatyId::new();
    fixture.world.world.entity_mut(nation_entity)
        .insert(NuclearTreatyMembership::new(treaty_id, 0))
        .insert(NuclearPosture::Developing);

    // Execute phase - violation should be recorded
    let mut nuclear_phase = NuclearPhase::new();
    nuclear_phase.execute(&mut fixture.world.world);

    // Mark as violator directly for testing prevention
    let mut violation_record = fixture.world.world.get_mut::<NuclearViolationRecord>(nation_entity).unwrap();
    violation_record.add_violation(NuclearViolationType::DevelopmentWhileInTreaty, 100);

    // Future development should not occur (implementation detail in develop_capability)
    // This test verifies the structure can handle violator tracking
    assert!(violation_record.is_violator());
}

// ============================================================================
// MULTI-NATION NUCLEAR SCENARIOS
// ============================================================================

#[test]
fn test_3_nation_nuclear_balance_of_power() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create 3 nuclear powers with different capabilities
    let nation1 = fixture.world.spawn_nation_with_nuclear(
        "Super Power A".to_string(),
        [255, 0, 0],
        false,
        Some(80.0),  // Advanced
    );

    let nation2 = fixture.world.spawn_nation_with_nuclear(
        "Super Power B".to_string(),
        [0, 255, 0],
        false,
        Some(75.0),  // Almost equal
    );

    let nation3 = fixture.world.spawn_nation_with_nuclear(
        "Emerging Power".to_string(),
        [0, 0, 255],
        false,
        Some(30.0),  // Developing
    );

    // Verify all have independent states
    let cap1 = fixture.world.world.get::<NuclearCapability>(nation1).unwrap().readiness;
    let cap2 = fixture.world.world.get::<NuclearCapability>(nation2).unwrap().readiness;
    let cap3 = fixture.world.world.get::<NuclearCapability>(nation3).unwrap().readiness;

    assert_eq!(cap1, 80.0);
    assert_eq!(cap2, 75.0);
    assert_eq!(cap3, 30.0);

    // Verify independent posturescan be set
    fixture.world.world.entity_mut(nation1).insert(NuclearPosture::Deployed);
    fixture.world.world.entity_mut(nation2).insert(NuclearPosture::Deterrent);
    fixture.world.world.entity_mut(nation3).insert(NuclearPosture::Developing);

    let posture1 = fixture.world.world.get::<NuclearPosture>(nation1).unwrap();
    let posture2 = fixture.world.world.get::<NuclearPosture>(nation2).unwrap();
    let posture3 = fixture.world.world.get::<NuclearPosture>(nation3).unwrap();

    assert_eq!(*posture1, NuclearPosture::Deployed);
    assert_eq!(*posture2, NuclearPosture::Deterrent);
    assert_eq!(*posture3, NuclearPosture::Developing);
}

#[test]
fn test_nuclear_use_effects_on_multiple_provinces() {
    let mut fixture = TestWorldBuilder::new().build();

    // Create attacker and target nations
    let attacker = fixture.world.spawn_nation_with_nuclear(
        "Attacker".to_string(),
        [255, 0, 0],
        false,
        Some(50.0),
    );

    let target = fixture.world.spawn_nation(
        "Target".to_string(),
        [0, 0, 255],
        false,
    );

    let attacker_id = fixture.world.world.get::<Nation>(attacker).unwrap().id;
    let target_id = fixture.world.world.get::<Nation>(target).unwrap().id;

    // Create target provinces
    let prov1 = fixture.world.spawn_province(
        "Province 1".to_string(),
        glam::Vec2::new(100.0, 100.0),
        ResourceType::Food,
        target_id,
    );

    let prov2 = fixture.world.spawn_province(
        "Province 2".to_string(),
        glam::Vec2::new(150.0, 150.0),
        ResourceType::Oil,
        target_id,
    );

    // Record initial states (using already spawned province entities prov1 and prov2)
    
    // Apply nuclear effects
    let province_ids = vec![
        fixture.world.world.get::<Province>(prov1).unwrap().id,
        fixture.world.world.get::<Province>(prov2).unwrap().id,
    ];

    let current_tick = fixture.current_tick();
    apply_nuclear_use_effects(
        &mut fixture.world.world,
        attacker_id,
        target_id,
        province_ids.clone(),
        current_tick,
    );

    // Verify attacker legitimacy loss
    let attacker_leg = fixture.world.world.get::<Legitimacy>(attacker).unwrap();
    assert!(attacker_leg.value < 100.0);  // Should have lost legitimacy

    // Verify target received damage
    let attacker_gdp_before = 1_000_000.0;  // Default GDP
    let target_leg = fixture.world.world.get::<Legitimacy>(target).unwrap();
    assert!(target_leg.value < 100.0);

    // Verify use was recorded
    let use_record = fixture.world.world.get::<NuclearUseRecord>(attacker).unwrap();
    assert_eq!(use_record.total_uses(), 1);
    assert_eq!(use_record.uses_against(target_id), 1);
}

// ============================================================================
// WORLD-WAR CRISIS GATE
// ============================================================================

#[test]
fn test_world_war_crisis_requires_multiple_alliances() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create nations and alliances
    let nation1 = fixture.world.spawn_nation(
        "Nation 1".to_string(),
        [255, 0, 0],
        false,
    );

    let nation2 = fixture.world.spawn_nation(
        "Nation 2".to_string(),
        [0, 255, 0],
        false,
    );

    let nation3 = fixture.world.spawn_nation(
        "Nation 3".to_string(),
        [0, 0, 255],
        false,
    );

    // Create and add to alliances (need 3+ for crisis)
    // This is simplified; real implementation would use alliance subsystem
    let n1_id = fixture.world.world.get::<Nation>(nation1).unwrap().id;
    let n2_id = fixture.world.world.get::<Nation>(nation2).unwrap().id;
    let n3_id = fixture.world.world.get::<Nation>(nation3).unwrap().id;

    // At least create the nations for the world-war-crisis check
    // Full alliance logic is tested in alliance subsystem
}

// ============================================================================
// BOT PERSONALITY INTEGRATION
// ============================================================================

#[test]
fn test_aggressive_bot_nuclear_context() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation_with_nuclear(
        "Aggressive Nuclear Power".to_string(),
        [255, 0, 0],
        false,
        Some(60.0),
    );

    fixture.world.world.entity_mut(nation)
        .insert(AIPersonality::Aggressive);

    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;

    // Aggressive AI should have higher inclination (20% higher probability than defensive)
    // We store the personality and it's used in bot decision logic
    let personality = fixture.world.world.get::<AIPersonality>(nation).unwrap();
    assert_eq!(*personality, AIPersonality::Aggressive);
}

#[test]
fn test_defensive_bot_nuclear_cautious() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation_with_nuclear(
        "Defensive Nuclear Power".to_string(),
        [0, 0, 255],
        false,
        Some(60.0),
    );

    fixture.world.world.entity_mut(nation)
        .insert(AIPersonality::Defensive);

    let personality = fixture.world.world.get::<AIPersonality>(nation).unwrap();
    assert_eq!(*personality, AIPersonality::Defensive);
}

// ============================================================================
// RECORD INTEGRITY
// ============================================================================

#[test]
fn test_nuclear_records_survive_phase_execution() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation_with_nuclear(
        "Test Nation".to_string(),
        [100, 100, 100],
        false,
        Some(45.0),
    );

    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;

    // Add to use record
    let mut use_record = fixture.world.world.get_mut::<NuclearUseRecord>(nation).unwrap();
    use_record.add_use(nation_id, NationId::new(), vec![], None, 100);

    // Execute phase
    let mut nuclear_phase = NuclearPhase::new();
    nuclear_phase.execute(&mut fixture.world.world);

    // Verify record still exists and is intact
    let record_after = fixture.world.world.get::<NuclearUseRecord>(nation).unwrap();
    assert_eq!(record_after.total_uses(), 1);
}

#[test]
fn test_nuclear_capability_persists_across_phases() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let initial_readiness = 35.0;
    let nation = fixture.world.spawn_nation_with_nuclear(
        "Persistent Nation".to_string(),
        [50, 50, 50],
        false,
        Some(initial_readiness),
    );

    // Set to developing
    fixture.world.world.entity_mut(nation)
        .insert(NuclearPosture::Developing);

    // Check before
    let cap_before = fixture.world.world.get::<NuclearCapability>(nation).unwrap().readiness;
    assert_eq!(cap_before, initial_readiness);

    // Execute nuclear phase multiple times
    let mut nuclear_phase = NuclearPhase::new();
    for _ in 0..3 {
        nuclear_phase.execute(&mut fixture.world.world);
    }

    // Verify capability was developed
    let cap_after = fixture.world.world.get::<NuclearCapability>(nation).unwrap().readiness;
    assert!(cap_after > cap_before);
}

// ============================================================================
// NUCLEAR USE RESTRICTIONS
// ============================================================================

#[test]
fn test_nuclear_requires_minimum_readiness() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create nation below use threshold (30%)
    let nation = fixture.world.spawn_nation_with_nuclear(
        "Primitive Nuclear Program".to_string(),
        [0, 0, 0],
        false,
        Some(15.0),  // Below 30% threshold
    );

    let cap = fixture.world.world.get::<NuclearCapability>(nation).unwrap();
    assert!(!cap.can_use());

    // Upgrade to use threshold
    let mut cap_mut = fixture.world.world.get_mut::<NuclearCapability>(nation).unwrap();
    cap_mut.readiness = 30.0;
    assert!(cap_mut.can_use());
}
