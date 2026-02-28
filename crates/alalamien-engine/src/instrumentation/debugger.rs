//! Dev tool debugger phase for diagnosing simulation health

use bevy_ecs::prelude::*;
use tracing::{debug, warn};

use crate::core::tick::TickPhase;
use crate::core::types::{Nation, Legitimacy, Population, Resources, Province};
use crate::EngineConfig;

/// A diagnostic phase added to the end of the tick pipeline
/// to trace out warnings and invariants. Runs only if debug_mode = true.
pub struct DebuggerPhase {
    config: EngineConfig,
}

impl DebuggerPhase {
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }
}

impl TickPhase for DebuggerPhase {
    fn name(&self) -> &str {
        "Debugger"
    }

    fn execute(&mut self, world: &mut World) {
        if !self.config.debug_mode {
            return;
        }

        // Trace and check invariants on nations
        let mut nation_query = world.query::<(&Nation, &Legitimacy)>();
        for (nation, legitimacy) in nation_query.iter(world) {
            if legitimacy.is_critical() {
                warn!(
                    nation = %nation.name,
                    legitimacy = %legitimacy.value,
                    "Nation legitimacy is critical. Imminent collapse risk."
                );
            }
        }

        // Trace and check invariants on provinces
        let mut prov_query = world.query::<(&Province, &Population, &Resources)>();
        for (province, population, resources) in prov_query.iter(world) {
            // Check for negative or runaway values
            if population.total == 0 {
                warn!(province = %province.name, "Province population reached exactly 0.");
            }
            if resources.food == 0.0 {
                warn!(province = %province.name, "Province is completely out of food.");
            }
            if resources.food < 0.0 {
                warn!(province = %province.name, food = resources.food, "INVARIANT BROKEN: Negative food.");
            }
            if resources.iron < 0.0 {
                warn!(province = %province.name, iron = resources.iron, "INVARIANT BROKEN: Negative iron.");
            }
            if resources.oil < 0.0 {
                warn!(province = %province.name, oil = resources.oil, "INVARIANT BROKEN: Negative oil.");
            }
        }
        
        debug!("Debugger phase execution completed.");
    }
}
