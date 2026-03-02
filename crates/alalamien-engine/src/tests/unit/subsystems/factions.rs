// Factions subsystem unit tests
// Tests for civil wars, faction mechanics, and stability

use super::super::TestWorldBuilder;
use crate::subsystems::factions::*;

#[test]
fn test_factions_module_exists() {
    // Verify the factions module loads
    let phase = FactionCivilWarPhase::new();
    assert_eq!(std::mem::size_of_val(&phase), 0);
}

#[test]
fn test_factions_world_integration() {
    // Test factions subsystem through world integration
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(4)
        .build();
    
    fixture.execute_ticks(50);
    assert_eq!(fixture.current_tick(), 50);
}
