//! Occupation subsystem
//!
//! Handles occupied province mechanics, resistance, and garrison requirements.

use bevy_ecs::prelude::*;
use tracing::debug;

use crate::core::tick::TickPhase;
use crate::core::types::{OccupiedProvince};

const RESISTANCE_GROWTH_RATE: f64 = 0.01;  // per tick
const PRODUCTION_PENALTY_FACTOR: f64 = 0.5;
const HIGH_RESISTANCE_THRESHOLD: f64 = 0.7;

/// Occupation phase - manages occupied provinces
pub struct OccupationPhase;

impl OccupationPhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for OccupationPhase {
    fn name(&self) -> &str {
        "Occupation"
    }
    
    fn execute(&mut self, world: &mut World) {
        debug!("OccupationPhase: Starting");
        
        // 1. Update resistance levels
        update_resistance(world);
        
        // 2. Apply production penalties
        apply_production_penalties(world);
        
        // 3. Apply garrison costs
        apply_garrison_costs(world);
        
        // 4. Check for independence movements
        check_independence_triggers(world);
        
        debug!("OccupationPhase: Complete");
    }
}

/// Update resistance in all occupied provinces
fn update_resistance(world: &mut World) {
    let mut query = world.query::<&mut OccupiedProvince>();
    
    for mut occupied in query.iter_mut(world) {
        // Resistance grows over time
        occupied.resistance = (occupied.resistance + RESISTANCE_GROWTH_RATE).min(1.0);
        
        if occupied.resistance > HIGH_RESISTANCE_THRESHOLD {
            debug!(
                "High resistance in province {:?}: {:.2}",
                occupied.province_id, occupied.resistance
            );
        }
    }
}

/// Apply production penalties to occupied provinces
fn apply_production_penalties(_world: &mut World) {
    // TODO: Implement production penalty application
    // 1. Query occupied provinces
    // 2. Reduce their resource production based on resistance
}

/// Apply garrison costs to occupying nations
fn apply_garrison_costs(_world: &mut World) {
    // TODO: Implement garrison cost calculation
    // 1. Calculate required garrison per province
    // 2. Drain resources from occupier
}

/// Check if any occupied provinces should trigger independence
fn check_independence_triggers(_world: &mut World) {
    // TODO: Implement independence checking
    // Triggered by:
    // - Very high resistance (> 0.9)
    // - Weak occupier
    // - Long occupation duration
}

/// Calculate production penalty for an occupied province
pub fn production_penalty(resistance: f64) -> f64 {
    1.0 - (resistance * PRODUCTION_PENALTY_FACTOR)
}

/// Calculate garrison requirement for occupation
pub fn garrison_requirement(resistance: f64) -> f64 {
    // Higher resistance = more troops needed
    1000.0 + (resistance * 10000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_penalty() {
        assert_eq!(production_penalty(0.0), 1.0);
        assert_eq!(production_penalty(1.0), 0.5);
        assert!(production_penalty(0.5) > 0.5 && production_penalty(0.5) < 1.0);
    }

    #[test]
    fn test_garrison_requirement() {
        let low_resistance = garrison_requirement(0.1);
        let high_resistance = garrison_requirement(0.9);
        assert!(high_resistance > low_resistance);
    }

    #[test]
    fn test_occupation_phase_creation() {
        let _phase = OccupationPhase::new();
        // Ensure it compiles
    }
}
