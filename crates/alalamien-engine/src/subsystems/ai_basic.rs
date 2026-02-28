//! Basic AI subsystem
//!
//! Simple rule-based AI for V0.3. Does not include advanced planning or memory.

use bevy_ecs::prelude::*;
use tracing::{debug, info};

use crate::core::tick::TickPhase;
use crate::core::types::{
    AIControlled, AIPersonality, NationId,
};

/// AI decision phase - makes strategic decisions for AI nations
pub struct AIDecisionPhase;

impl AIDecisionPhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for AIDecisionPhase {
    fn name(&self) -> &str {
        "AI Decision"
    }
    
    fn execute(&mut self, world: &mut World) {
        debug!("AIDecisionPhase: Starting");
        
        // Query all AI-controlled nations
        let ai_nations = identify_ai_nations(world);
        
        for nation_id in ai_nations {
            make_ai_decision(world, nation_id);
        }
        
        debug!("AIDecisionPhase: Complete");
    }
}

/// Identify all AI-controlled nations
fn identify_ai_nations(world: &mut World) -> Vec<NationId> {
    let mut ai_nations = Vec::new();
    
    let mut query = world.query::<(&crate::core::types::Nation, &AIControlled)>();
    for (nation, _) in query.iter(world) {
        ai_nations.push(nation.id);
    }
    
    ai_nations
}

/// Make a decision for one AI nation
fn make_ai_decision(world: &mut World, nation_id: NationId) {
    let context = assess_situation(world, nation_id);
    let personality = get_personality(world, nation_id);
    
    let action = decide_action(personality, &context);
    
    execute_action(world, nation_id, action);
}

/// Strategic context for AI decision-making
#[derive(Debug)]
struct StrategyContext {
    legitimacy: f64,
    military_capacity: f64,
    economic_health: f64,
    active_wars: usize,
    military_advantage: f64,
    being_attacked: bool,
}

/// Assess current strategic situation
fn assess_situation(_world: &World, _nation_id: NationId) -> StrategyContext {
    // TODO: Implement full situation assessment
    // For now, return default context
    StrategyContext {
        legitimacy: 50.0,
        military_capacity: 100.0,
        economic_health: 0.7,
        active_wars: 0,
        military_advantage: 1.0,
        being_attacked: false,
    }
}

/// Get AI personality for a nation
fn get_personality(_world: &World, _nation_id: NationId) -> AIPersonality {
    // TODO: Query actual personality component
    // For now, return balanced
    AIPersonality::Balanced
}

/// AI action types
#[derive(Debug)]
enum AIAction {
    DeclareWar { target: NationId },
    SeekPeace { target: NationId },
    BuildArmy,
    FocusEconomy,
    DefendBorders,
    DoNothing,
}

/// Decide what action to take based on personality and context
fn decide_action(personality: AIPersonality, context: &StrategyContext) -> AIAction {
    // Emergency: internal crisis
    if context.legitimacy < 30.0 {
        return AIAction::FocusEconomy;
    }
    
    // Being attacked: defend
    if context.being_attacked {
        return AIAction::DefendBorders;
    }
    
    match personality {
        AIPersonality::Aggressive => {
            if context.military_advantage > 1.5 && context.economic_health > 0.6 {
                // TODO: Actually find a weak target
                // AIAction::DeclareWar { target }
                AIAction::BuildArmy
            } else {
                AIAction::BuildArmy
            }
        }
        AIPersonality::Defensive => {
            if context.military_capacity < 50.0 {
                AIAction::BuildArmy
            } else {
                AIAction::FocusEconomy
            }
        }
        AIPersonality::Balanced => {
            if context.military_advantage > 2.0 {
                // Opportunistic expansion
                AIAction::BuildArmy
            } else if context.legitimacy < 40.0 {
                AIAction::FocusEconomy
            } else {
                AIAction::BuildArmy
            }
        }
    }
}

/// Execute the chosen AI action
fn execute_action(_world: &mut World, nation_id: NationId, action: AIAction) {
    debug!("AI {:?} taking action: {:?}", nation_id, action);
    
    match action {
        AIAction::DeclareWar { target } => {
            // TODO: Call warfare system to declare war
            info!("AI {:?} declaring war on {:?}", nation_id, target);
        }
        AIAction::SeekPeace { target } => {
            // TODO: Initiate peace negotiations
            info!("AI {:?} seeking peace with {:?}", nation_id, target);
        }
        AIAction::BuildArmy => {
            // TODO: Allocate resources to military
            debug!("AI {:?} building army", nation_id);
        }
        AIAction::FocusEconomy => {
            // TODO: Prioritize economic development
            debug!("AI {:?} focusing on economy", nation_id);
        }
        AIAction::DefendBorders => {
            // TODO: Position armies defensively
            debug!("AI {:?} defending borders", nation_id);
        }
        AIAction::DoNothing => {
            // Passive observation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_phase_creation() {
        let _phase = AIDecisionPhase::new();
        // Ensure it compiles
    }

    #[test]
    fn test_defensive_ai_prioritizes_safety() {
        let context = StrategyContext {
            legitimacy: 50.0,
            military_capacity: 30.0,
            economic_health: 0.7,
            active_wars: 0,
            military_advantage: 1.0,
            being_attacked: false,
        };
        
        let action = decide_action(AIPersonality::Defensive, &context);
        // Defensive AI should build army when weak
        matches!(action, AIAction::BuildArmy);
    }

    #[test]
    fn test_emergency_overrides_personality() {
        let context = StrategyContext {
            legitimacy: 20.0,  // Critical
            military_capacity: 100.0,
            economic_health: 0.3,
            active_wars: 0,
            military_advantage: 2.0,
            being_attacked: false,
        };
        
        // Even aggressive AI should focus economy in crisis
        let action = decide_action(AIPersonality::Aggressive, &context);
        matches!(action, AIAction::FocusEconomy);
    }
}
