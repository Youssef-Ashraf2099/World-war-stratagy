//! Trade subsystem
//!
//! Handles resource distribution across trade routes, abstract pricing, and deficit resolution.

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use tracing::{trace, debug};

use crate::core::tick::TickPhase;
use crate::core::types::{Resources, OwnedBy, ProvinceId};
use crate::core::province_graph::ProvinceGraph;

/// Trade simulation phase
pub struct TradePhase;

impl TradePhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for TradePhase {
    fn name(&self) -> &str {
        "Trade"
    }

    fn execute(&mut self, world: &mut World) {
        // For simplicity in V0.2, if a province has a deficit (food < 0),
        // it attempts to pull from neighboring provinces of the same owner
        // that have a surplus.
        
        // This requires multi-province manipulation, which we do by cloning
        // the resources snapshot, resolving trades, and applying back.

        let graph = world.resource::<ProvinceGraph>().clone();
        
        // Step 1: Pull current province states
        let mut province_states: HashMap<ProvinceId, (crate::core::types::NationId, Resources)> = HashMap::new();
        
        {
            let mut query = world.query::<(&crate::core::types::Province, &OwnedBy, &Resources)>();
            for (prov, owner, res) in query.iter(world) {
                province_states.insert(prov.id, (owner.nation_id, res.clone()));
            }
        }

        // Step 2: Resolve Local Trade (Simulated scalar price abstract via free flow within same nation for now)
        let mut trade_updates: HashMap<ProvinceId, Resources> = province_states.iter().map(|(id, (_, r))| (*id, r.clone())).collect();

        // 5 passes of trade diffusion
        for _ in 0..5 {
            let current_state = trade_updates.clone();
            
            for (prov_id, (owner, _)) in &province_states {
                let neighbors = graph.get_neighbors(*prov_id);
                
                // If we are starving (food < 0), ask neighbors
                let my_food = current_state.get(prov_id).unwrap().food;
                if my_food < 0.0 {
                    let mut needed = -my_food;
                    for neighbor_id in neighbors {
                        if let Some((n_owner, _)) = province_states.get(&neighbor_id) {
                            // Only trade within same nation or allied (for V0.2 just same nation)
                            if owner == n_owner {
                                if let Some(n_res) = current_state.get(&neighbor_id) {
                                    if n_res.food > 0.0 {
                                        let take_amount = n_res.food.min(needed);
                                        
                                        // Update local tracking
                                        let my_res_mut = trade_updates.get_mut(prov_id).unwrap();
                                        my_res_mut.food += take_amount;
                                        
                                        let n_res_mut = trade_updates.get_mut(&neighbor_id).unwrap();
                                        n_res_mut.food -= take_amount;
                                        
                                        needed -= take_amount;
                                        if needed <= 0.001 {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Step 3: Apply back to world and process unmitigated starvation
        let mut query = world.query::<(&crate::core::types::Province, &mut Resources, &mut crate::core::types::Population)>();
        for (prov, mut res, mut pop) in query.iter_mut(world) {
            if let Some(updated_res) = trade_updates.get(&prov.id) {
                res.food = updated_res.food;
                res.iron = updated_res.iron;
                res.oil = updated_res.oil;

                // Simple starvation effect: if food is STILL negative after trade, people die off
                if res.food < 0.0 {
                    let starvation = res.food.abs();
                    let deaths = (starvation * 10.0) as u64; // Tunable constant
                    pop.total = pop.total.saturating_sub(deaths);
                }
            }
        }
        
        debug!("Trade phase executed spreading resources across immediate graph edges.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::world::WorldState;
    use crate::core::types::{Nation, Legitimacy, Population, ResourceType, Infrastructure};
    use glam::Vec2;

    #[test]
    fn test_trade_blockade() {
        let mut world_state = WorldState::new(42);
        let nation_id = world_state.spawn_nation("TestNation".to_string(), [0, 0, 0], false);
        let owner_nation_id = world_state.world.get::<Nation>(nation_id).unwrap().id;

        // Spawn Capital (High Population, No Food)
        let capital = world_state.spawn_province("Capital".to_string(), Vec2::new(0.0, 0.0), ResourceType::Iron, owner_nation_id);
        if let Some(mut pop) = world_state.world.get_mut::<Population>(capital) { pop.total = 5_000_000; }
        if let Some(mut res) = world_state.world.get_mut::<Resources>(capital) { res.food = 0.0; }

        // Spawn Farm (Low Population, Lots of Food)
        let farm = world_state.spawn_province("Farm".to_string(), Vec2::new(1.0, 0.0), ResourceType::Food, owner_nation_id);
        if let Some(mut pop) = world_state.world.get_mut::<Population>(farm) { pop.total = 100_000; }
        if let Some(mut res) = world_state.world.get_mut::<Resources>(farm) { res.food = 2000.0; }

        let capital_id = world_state.world.get::<crate::core::types::Province>(capital).unwrap().id;
        let farm_id = world_state.world.get::<crate::core::types::Province>(farm).unwrap().id;

        // 1. Unconnected state (Blockaded) -> Capital should starve
        let init_pop = world_state.world.get::<Population>(capital).unwrap().total;
        
        let mut economic_phase = crate::subsystems::economic::EconomicPhase::new();
        economic_phase.execute(&mut world_state.world);

        let mut trade_phase = TradePhase::new();
        trade_phase.execute(&mut world_state.world);

        let pop_after_blockade = world_state.world.get::<Population>(capital).unwrap().total;
        assert!(pop_after_blockade < init_pop, "Capital should lose population from starvation");
        
        // 2. Connected state
        world_state.add_province_border(capital_id, farm_id);
        
        // Give back food
        if let Some(mut res) = world_state.world.get_mut::<Resources>(farm) { res.food = 2000.0; }
        if let Some(mut res) = world_state.world.get_mut::<Resources>(capital) { res.food = 0.0; }
        
        let init_pop_connected = world_state.world.get::<Population>(capital).unwrap().total;
        economic_phase.execute(&mut world_state.world);
        trade_phase.execute(&mut world_state.world);
        
        let pop_after_connected = world_state.world.get::<Population>(capital).unwrap().total;
        // Population might still decrease slightly if deficit > surplus, but trade definitely mitigated it.
        // Actually since we trade 2000 food to Capital and it consumed 500, it should have > 0 food and no deaths.
        assert_eq!(pop_after_connected, init_pop_connected, "Capital should not starve when connected to farm");
    }
}
