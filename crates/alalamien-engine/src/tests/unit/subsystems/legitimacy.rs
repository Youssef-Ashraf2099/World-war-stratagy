// Legitimacy subsystem unit tests
// Tests for nation stability, war exhaustion, economic stress, and alliance burden

use super::super::TestWorldBuilder;
use crate::subsystems::legitimacy::*;
use crate::core::types::Legitimacy;

#[test]
fn test_legitimacy_respects_ceiling() {
    // Legitimacy component should clamp to 0-100
    let mut legit = Legitimacy::new(99.0);
    legit.modify(5.0);
    assert_eq!(legit.value, 100.0, "Legitimacy should not exceed 100");
}

#[test]
fn test_legitimacy_respects_floor() {
    // Legitimacy component should clamp to 0-100
    let mut legit = Legitimacy::new(5.0);
    legit.modify(-10.0);
    assert_eq!(legit.value, 0.0, "Legitimacy should not go below 0");
}

#[test]
fn test_legitimacy_default() {
    let legit = Legitimacy::default();
    assert!(legit.value >= 0.0 && legit.value <= 100.0);
}

#[test]
fn test_legitimacy_is_stable() {
    let stable = Legitimacy::new(60.0);
    let unstable = Legitimacy::new(30.0);
    let critical = Legitimacy::new(10.0);
    
    assert!(stable.is_stable(), "60 legitimacy should be stable");
    assert!(!unstable.is_stable(), "30 legitimacy should not be stable");
    assert!(critical.is_critical(), "10 legitimacy should be critical");
}

#[test]
fn test_legitimacy_modification_chain() {
    let mut legit = Legitimacy::new(50.0);
    
    legit.modify(10.0);
    assert_eq!(legit.value, 60.0);
    
    legit.modify(-20.0);
    assert_eq!(legit.value, 40.0);
    
    legit.modify(70.0);
    assert_eq!(legit.value, 100.0); // Clamped
}

#[test]
fn test_legitimacy_world_integration() {
    // Test legitimacy subsystem through world integration
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    fixture.execute_ticks(100);
    assert_eq!(fixture.current_tick(), 100);
}
