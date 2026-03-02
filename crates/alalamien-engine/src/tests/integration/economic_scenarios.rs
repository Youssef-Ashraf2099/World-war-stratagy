// Expanded integration tests: Complex economic scenarios
// Tests interaction between economic subsystem and other systems

use crate::tests::fixtures::{TestWorldBuilder, assertions};

#[test]
fn test_economic_crisis_triggers_unrest() {
    // Verify that severe economic problems affect legitimacy
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(3)
        .build();
    
    fixture.execute_ticks(100);
    
    // After 100 ticks with stress, should show effects
    assert_eq!(fixture.current_tick(), 100);
    assertions::assert_world_stable(&mut fixture, "economic_crisis_scenario");
}

#[test]
fn test_economic_recovery_after_war() {
    // Nations rebuild economy after war
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(4)
        .build();
    
    // Run through conflict and recovery
    fixture.execute_ticks(150);
    
    assert_eq!(fixture.current_tick(), 150);
    assertions::assert_world_stable(&mut fixture, "economic_recovery_scenario");
}

#[test]
fn test_resource_bottleneck_scenarios() {
    // Nations with different resource distributions
    let mut fixture = TestWorldBuilder::new()
        .with_seed(123)
        .with_nations(5)
        .build();
    
    // Extended session to show resource dynamics
    fixture.execute_ticks(200);
    
    assert_eq!(fixture.current_tick(), 200);
    assertions::assert_world_stable(&mut fixture, "resource_bottleneck");
}

#[test]
fn test_trade_network_formation() {
    // Verify trading relationships develop
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(6)
        .build();
    
    // Trade networks take time to form
    fixture.execute_ticks(180);
    
    assert_eq!(fixture.current_tick(), 180);
    assertions::assert_world_stable(&mut fixture, "trade_network_formation");
}
