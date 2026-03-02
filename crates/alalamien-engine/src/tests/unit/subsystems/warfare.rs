// Warfare subsystem unit tests
// Tests for military units, combat resolution, and war mechanics

use crate::subsystems::warfare::*;
use crate::core::types::{NationId, WarState};

#[test]
fn test_warfare_phase_creation() {
    let phase = WarfarePhase::new();
    assert_eq!(std::mem::size_of_val(&phase), 0);
}

#[test]
fn test_simple_war_state_tracking() {
    let nation_a = NationId::new();
    let nation_b = NationId::new();
    
    let war_state = WarState {
        at_war_with: vec![nation_b].into_iter().collect(),
    };
    
    assert!(war_state.at_war_with.contains(&nation_b));
}
// 4. Adapt assertions to use test fixture helpers
