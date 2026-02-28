//! Vassalage subsystem
//!
//! Handles vassal relationships, tribute, and peaceful annexation.

use bevy_ecs::prelude::*;
use tracing::{debug, info, warn};

use crate::core::tick::TickPhase;
use crate::core::types::{
    AutonomyLevel, NationId, Tick, VassalRelation,
};

const LOYALTY_DECAY_RATE: f64 = 0.1;  // per tick if mistreated
const INDEPENDENCE_THRESHOLD: f64 = 20.0;

/// Vassalage phase - manages vassal relationships
pub struct VassalagePhase;

impl VassalagePhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for VassalagePhase {
    fn name(&self) -> &str {
        "Vassalage"
    }
    
    fn execute(&mut self, world: &mut World) {
        debug!("VassalagePhase: Starting");
        
        // 1. Transfer tribute from vassals to overlords
        transfer_tribute(world);
        
        // 2. Update loyalty levels
        update_loyalty(world);
        
        // 3. Check for independence wars
        check_independence_triggers(world);
        
        debug!("VassalagePhase: Complete");
    }
}

/// Transfer tribute from vassals to overlords
fn transfer_tribute(_world: &mut World) {
    // TODO: Implement tribute transfer
    // 1. Query all VassalRelation components
    // 2. Calculate tribute amount (vassal GDP * percentage)
    // 3. Transfer resources
}

/// Update loyalty for all vassal relationships
fn update_loyalty(world: &mut World) {
    let mut query = world.query::<&mut VassalRelation>();
    
    for mut vassal in query.iter_mut(world) {
        // High tribute reduces loyalty
        if vassal.tribute_percentage > 30.0 {
            vassal.loyalty = (vassal.loyalty - LOYALTY_DECAY_RATE).max(0.0);
        } else if vassal.tribute_percentage < 15.0 {
            // Low tribute increases loyalty
            vassal.loyalty = (vassal.loyalty + 0.05).min(100.0);
        }
        
        if vassal.loyalty < INDEPENDENCE_THRESHOLD {
            warn!(
                "Vassal {:?} has low loyalty: {:.1}",
                vassal.vassal, vassal.loyalty
            );
        }
    }
}

/// Check if any vassals should declare independence
fn check_independence_triggers(_world: &mut World) {
    // TODO: Implement independence war triggering
    // Triggered by:
    // - Loyalty < 20
    // - Overlord weakened in war
    // - Vassal becomes stronger than overlord
}

/// Offer vassalization to a target nation
pub fn offer_vassalization(
    world: &mut World,
    overlord: NationId,
    target: NationId,
    tribute_percentage: f64,
    _autonomy: AutonomyLevel,
    current_tick: Tick,
) -> bool {
    let accepted = evaluate_vassal_offer(world, overlord, target, tribute_percentage);
    
    if accepted {
        world.spawn(VassalRelation {
            overlord,
            vassal: target,
            tribute_percentage,
            established_tick: current_tick,
            loyalty: 50.0,
        });
        
        info!(
            "Vassalization accepted: {:?} → {:?}",
            target, overlord
        );
    }
    
    accepted
}

/// Evaluate whether a nation accepts vassalage
fn evaluate_vassal_offer(
    _world: &World,
    _overlord: NationId,
    _target: NationId,
    _tribute: f64,
) -> bool {
    // TODO: Implement full evaluation logic
    // Accept if:
    // - Power ratio > 5:1
    // - Under attack AND power ratio > 2:1
    // - Generous terms (< 10% tribute)
    
    false  // Placeholder
}

/// Attempt to annex a vassal or weak nation
pub fn attempt_annexation(
    world: &mut World,
    annexer: NationId,
    target: NationId,
    compensation: f64,
) -> bool {
    let accepted = evaluate_annexation(world, target, compensation);
    
    if accepted {
        execute_annexation(world, annexer, target);
        info!("Annexation successful: {:?} → {:?}", target, annexer);
    }
    
    accepted
}

/// Evaluate annexation offer
fn evaluate_annexation(_world: &World, _target: NationId, _compensation: f64) -> bool {
    // TODO: Implement evaluation
    // Only accept if:
    // - Legitimacy < 15 (collapsing)
    // - Compensation > GDP * 5
    
    false  // Placeholder
}

/// Execute annexation (transfer provinces, merge)
fn execute_annexation(_world: &mut World, _annexer: NationId, _target: NationId) {
    // TODO: Implement annexation execution
    // 1. Transfer all provinces
    // 2. Merge populations
    // 3. Despawn target nation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vassalage_phase_creation() {
        let _phase = VassalagePhase::new();
        // Ensure it compiles
    }

    #[test]
    fn test_high_tribute_reduces_loyalty() {
        let mut vassal = VassalRelation {
            overlord: NationId::new(),
            vassal: NationId::new(),
            tribute_percentage: 40.0,
            established_tick: 0,
            loyalty: 50.0,
        };
        
        let initial_loyalty = vassal.loyalty;
        
        // Simulate loyalty decay
        if vassal.tribute_percentage > 30.0 {
            vassal.loyalty = (vassal.loyalty - LOYALTY_DECAY_RATE).max(0.0);
        }
        
        assert!(vassal.loyalty < initial_loyalty);
    }
}
