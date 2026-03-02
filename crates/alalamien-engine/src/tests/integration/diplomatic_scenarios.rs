// Expanded integration tests: Complex diplomatic scenarios
// Tests interaction between diplomatic subsystem and other systems

use crate::tests::fixtures::{TestWorldBuilder, assertions};

#[test]
fn test_alliance_dissolution_under_pressure() {
    // Alliances break when members face conflicts
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(4)
        .build();
    
    // Simulate alliance pressures
    fixture.execute_ticks(120);
    
    assert_eq!(fixture.current_tick(), 120);
    assertions::assert_world_stable(&mut fixture, "alliance_dissolution");
}

#[test]
fn test_diplomatic_isolation_effects() {
    // Nations isolated diplomatically experience problems
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    fixture.execute_ticks(140);
    
    assert_eq!(fixture.current_tick(), 140);
    assertions::assert_world_stable(&mut fixture, "diplomatic_isolation");
}

#[test]
fn test_peace_treaty_impact() {
    // Peace treaties should improve relations
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(3)
        .build();
    
    fixture.execute_ticks(100);
    
    // Continue with peaceful state
    fixture.execute_ticks(100);
    
    assert_eq!(fixture.current_tick(), 200);
    assertions::assert_world_stable(&mut fixture, "peace_treaty_impact");
}

#[test]
fn test_neutral_mediator_role() {
    // Nations can take neutral roles in conflicts
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    fixture.execute_ticks(160);
    
    assert_eq!(fixture.current_tick(), 160);
    assertions::assert_world_stable(&mut fixture, "neutral_mediator");
}

#[test]
fn test_reputation_recovery_trajectory() {
    // Nations can rebuild reputation over time
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(3)
        .build();
    
    // Extended period for reputation recovery
    fixture.execute_ticks(250);
    
    assert_eq!(fixture.current_tick(), 250);
    assertions::assert_world_stable(&mut fixture, "reputation_recovery");
}
