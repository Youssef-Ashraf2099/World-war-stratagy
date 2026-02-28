//! Warfare subsystem
//!
//! Handles war declarations, peace negotiations, and war state management.

use bevy_ecs::prelude::*;
use tracing::{debug, info};

use crate::core::tick::TickPhase;
use crate::core::types::{
    CasusBelli, NationId, PeaceTreaty, Tick, WarDeclaration, WarExhaustion, WarGoal,
    WarId, WarState,
};

/// Warfare phase - manages war declarations and peace
pub struct WarfarePhase;

impl WarfarePhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for WarfarePhase {
    fn name(&self) -> &str {
        "Warfare"
    }
    
    fn execute(&mut self, world: &mut World) {
        debug!("WarfarePhase: Starting");
        
        // 1. Process new war declarations
        process_war_declarations(world);
        
        // 2. Check for peace treaty conditions
        check_peace_conditions(world);
        
        // 3. Update war exhaustion
        update_war_exhaustion(world);
        
        debug!("WarfarePhase: Complete");
    }
}

/// Process pending war declarations
fn process_war_declarations(_world: &mut World) {
    // TODO: Implement war declaration processing
    // 1. Find pending WarDeclaration components
    // 2. Update WarState for both nations
    // 3. Apply legitimacy penalties
    // 4. Cancel trade agreements
}

/// Check if any wars should end in peace
fn check_peace_conditions(_world: &mut World) {
    // TODO: Implement peace condition checking
    // 1. Check war exhaustion levels
    // 2. Check for capital capture
    // 3. Check for stalemate duration
    // 4. Generate peace treaties
}

/// Update war exhaustion for all warring nations
fn update_war_exhaustion(world: &mut World) {
    let mut query = world.query::<(&WarState, &mut WarExhaustion)>();
    
    for (war_state, mut exhaustion) in query.iter_mut(world) {
        if !war_state.at_war_with.is_empty() {
            // Increase exhaustion each tick at war
            exhaustion.value = (exhaustion.value + 0.1).min(100.0);
        } else {
            // Decrease exhaustion slowly when at peace
            exhaustion.value = (exhaustion.value - 0.05).max(0.0);
        }
    }
}

/// Declare war between two nations
pub fn declare_war(
    world: &mut World,
    aggressor: NationId,
    defender: NationId,
    casus_belli: CasusBelli,
    war_goal: WarGoal,
    current_tick: Tick,
) {
    let war_id = WarId::new();
    
    // Create war declaration
    world.spawn(WarDeclaration {
        war_id,
        aggressor,
        defender,
        casus_belli,
        war_goal,
        declared_tick: current_tick,
    });
    
    info!(
        "War declared: {:?} vs {:?} (War ID: {:?})",
        aggressor, defender, war_id
    );
}

/// Sign peace treaty between warring nations
pub fn sign_peace(
    world: &mut World,
    war_id: WarId,
    victor: Option<NationId>,
    terms: crate::core::types::PeaceTerms,
    current_tick: Tick,
) {
    world.spawn(PeaceTreaty {
        war_id,
        victor,
        terms,
        signed_tick: current_tick,
    });
    
    info!("Peace treaty signed for war: {:?}", war_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::WorldState;

    #[test]
    fn test_warfare_phase_creation() {
        let _phase = WarfarePhase::new();
        // Ensure it compiles
    }

    #[test]
    fn test_war_exhaustion_increases_at_war() {
        let mut world = WorldState::new(42);
        let mut phase = WarfarePhase::new();
        
        // Create a nation at war
        let nation = world.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        
        // Add war exhaustion component
        world.world.entity_mut(nation).insert(WarExhaustion { value: 10.0 });
        world.world.entity_mut(nation).insert(WarState {
            at_war_with: vec![NationId::new()],
        });
        
        let initial_exhaustion = world.world.get::<WarExhaustion>(nation).unwrap().value;
        
        // Run phase
        phase.execute(&mut world.world);
        
        let final_exhaustion = world.world.get::<WarExhaustion>(nation).unwrap().value;
        assert!(final_exhaustion > initial_exhaustion);
    }
}
