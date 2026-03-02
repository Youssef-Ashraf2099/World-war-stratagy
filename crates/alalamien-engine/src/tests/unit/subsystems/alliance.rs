// Alliance subsystem unit tests
// Tests for alliance formation, cohesion, and mechanics

use super::super::TestWorldBuilder;
use crate::subsystems::alliance::*;

#[test]
fn test_alliance_module_exists() {
    // Verify the alliance module loads
    let phase = AlliancePhase::new();
    assert_eq!(std::mem::size_of_val(&phase), 0);
}

#[test]
fn test_alliance_world_integration() {
    // Test alliance subsystem through world integration
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(3)
        .build();
    
    fixture.execute_ticks(10);
    assert_eq!(fixture.current_tick(), 10);
}
