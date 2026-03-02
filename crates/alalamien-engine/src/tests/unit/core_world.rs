/// Unit tests for WorldState system
/// 
/// Tests cover:
/// - World initialization
/// - Nation and province spawning
/// - State persistence and loading
/// - Entity relationships

use super::TestWorldBuilder;
use crate::core::WorldState;

#[test]
fn test_world_creation() {
    let world = WorldState::new(42);
    assert_eq!(world.current_tick(), 0);
}

#[test]
fn test_nation_spawning() {
    let mut world = WorldState::new(42);
    let nation = world.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
    
    // Verify nation was created
    let nation_component = world.world.get::<crate::core::types::Nation>(nation);
    assert!(nation_component.is_some());
}

// TODO: Migrate existing world tests from core/world.rs
