//! Logistics subsystem for V0.3
//!
//! Handles supply line pathfinding, teeth-to-tail ratio degradation,
//! infrastructure bottlenecks, and army attrition.

use bevy_ecs::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::debug;

use crate::core::tick::TickPhase;
use crate::core::types::{
    Army, Capital, Infrastructure, Logistics as NationLogistics, Nation, NationId,
    Province, ProvinceId, OwnedBy,
};
use crate::core::province_graph::ProvinceGraph;

/// Configuration constants for logistics math
const TAIL_BASE_RATE: f64 = 0.10;        // 10% of personnel needed per hop as base
const TAIL_EXPONENT_BASE: f64 = 1.1;     // exponential scale per hop (10%)
const INFRA_FLOW_PER_LEVEL: u64 = 10_000; // each infra level allows 10k men supplied
const ATTRITION_BOTTLENECK: f64 = 0.05;  // 5% of deficit dies from supply shortage
const ATTRITION_ISOLATED: f64 = 0.10;   // 10% of army dies per tick when isolated

pub struct LogisticsPhase;

impl LogisticsPhase {
    pub fn new() -> Self {
        Self
    }
}

impl TickPhase for LogisticsPhase {
    fn name(&self) -> &str {
        "Logistics"
    }

    fn execute(&mut self, world: &mut World) {
        let graph = world.resource::<ProvinceGraph>().clone();

        // 1. Collect province data (ownership + infrastructure level)
        let mut prov_meta: HashMap<ProvinceId, (NationId, u32)> = HashMap::new();
        {
            let mut query = world.query::<(&Province, &OwnedBy, &Infrastructure)>();
            for (prov, owned, infra) in query.iter(world) {
                prov_meta.insert(prov.id, (owned.nation_id, infra.level));
            }
        }

        // 2. Identify Capital province for each nation
        let mut capitals: HashMap<NationId, ProvinceId> = HashMap::new();
        {
            let mut query = world.query::<(&Province, &OwnedBy, &Capital)>();
            for (prov, owned, _) in query.iter(world) {
                capitals.insert(owned.nation_id, prov.id);
            }
        }

        // 3. Compute per-army supply cost and attrition
        // Collect (entity, personnel, owner, prov_id, forced_march) to avoid world borrow overlap
        let army_snapshot: Vec<(Entity, u64, NationId, ProvinceId, bool)> = {
            let mut query = world.query::<(Entity, &Army, &Province)>();
            query
                .iter(world)
                .map(|(e, a, p)| (e, a.personnel, a.owner, p.id, a.forced_march))
                .collect()
        };

        let mut attrition_list: Vec<(Entity, u64)> = Vec::new();
        let mut nation_logistics_cost: HashMap<NationId, f64> = HashMap::new();

        for (entity, personnel, owner, army_prov, forced_march) in &army_snapshot {
            let owner = *owner;
            let army_prov = *army_prov;
            let personnel = *personnel;
            let forced_march = *forced_march;

            match capitals.get(&owner) {
                None => {
                    // No capital: fully isolated
                    let deaths = (personnel as f64 * ATTRITION_ISOLATED) as u64;
                    attrition_list.push((*entity, deaths));
                }
                Some(&cap_id) => {
                    // BFS from army province to capital, through friendly territory only
                    let mut queue: VecDeque<(ProvinceId, usize, u32)> = VecDeque::new();
                    queue.push_back((army_prov, 0, u32::MAX));
                    let mut visited: HashSet<ProvinceId> = HashSet::new();
                    visited.insert(army_prov);

                    let mut found = false;
                    let mut route_len: usize = 0;
                    let mut min_flow: u32 = 0;

                    // Seed starting infra
                    let start_infra = prov_meta.get(&army_prov).map(|(_, i)| *i).unwrap_or(1);
                    queue.pop_front(); // already added, re-push with infra seeded
                    queue.push_back((army_prov, 0, start_infra));

                    while let Some((curr, dist, running_min)) = queue.pop_front() {
                        if curr == cap_id {
                            found = true;
                            route_len = dist;
                            min_flow = running_min;
                            break;
                        }

                        for neighbor in graph.get_neighbors(curr) {
                            if visited.contains(&neighbor) {
                                continue;
                            }
                            if let Some((n_owner, n_infra)) = prov_meta.get(&neighbor) {
                                if *n_owner == owner {
                                    visited.insert(neighbor);
                                    let new_min = running_min.min(*n_infra);
                                    queue.push_back((neighbor, dist + 1, new_min));
                                }
                            }
                        }
                    }

                    if !found {
                        // Supply line broken: heavy attrition
                        let deaths = (personnel as f64 * ATTRITION_ISOLATED) as u64;
                        attrition_list.push((*entity, deaths));
                        debug!(owner=?owner, "Army supply line broken, heavy attrition");
                    } else {
                        // Teeth-to-Tail ratio: tail = personnel * (TAIL_BASE ^ dist - 1)
                        let distance_factor = TAIL_EXPONENT_BASE.powi(route_len as i32) - 1.0;
                        let mut tail_needed = (personnel as f64 * distance_factor) as u64;

                        if forced_march {
                            tail_needed = tail_needed.saturating_mul(3);
                        }

                        // Infrastructure flow cap: min_infra_level * INFRA_FLOW_PER_LEVEL
                        let flow_limit = (min_flow as u64).saturating_mul(INFRA_FLOW_PER_LEVEL);
                        let total_supply_demand = personnel.saturating_add(tail_needed);

                        if total_supply_demand > flow_limit {
                            let deficit = total_supply_demand - flow_limit;
                            let deaths = (deficit as f64 * ATTRITION_BOTTLENECK) as u64;
                            if deaths > 0 {
                                attrition_list.push((*entity, deaths));
                                debug!(owner=?owner, deficit, deaths, "Infra bottleneck attrition");
                            }
                        }

                        // Accumulate logistics cost for the nation
                        *nation_logistics_cost.entry(owner).or_insert(0.0) +=
                            tail_needed as f64 * 0.001;
                    }
                }
            }
        }

        // 4. Apply attrition to armies
        for (entity, deaths) in attrition_list {
            if let Some(mut army) = world.get_mut::<Army>(entity) {
                army.personnel = army.personnel.saturating_sub(deaths);
            }
        }

        // 5. Drain national logistics pool by army upkeep
        let mut nation_query = world.query::<(&Nation, &mut NationLogistics)>();
        for (nation, mut log) in nation_query.iter_mut(world) {
            if let Some(cost) = nation_logistics_cost.get(&nation.id) {
                log.value -= cost;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::world::WorldState;
    use crate::core::types::ResourceType;
    use glam::Vec2;

    fn setup_two_province_nation(world: &mut WorldState) -> (ProvinceId, ProvinceId, NationId) {
        let nation_entity = world.spawn_nation("TestNation".to_string(), [255, 0, 0], false);
        let nation_id = world.world.get::<Nation>(nation_entity).unwrap().id;

        let cap_entity = world.spawn_province(
            "Capital".to_string(),
            Vec2::new(0.0, 0.0),
            ResourceType::Iron,
            nation_id,
        );
        // Mark as Capital and set infrastructure to level 3 to avoid bottleneck
        world.world.entity_mut(cap_entity).insert(Capital);
        if let Some(mut infra) = world.world.get_mut::<Infrastructure>(cap_entity) {
            infra.level = 3;
        }
        let cap_id = world.world.get::<Province>(cap_entity).unwrap().id;

        let front_entity = world.spawn_province(
            "Frontline".to_string(),
            Vec2::new(1.0, 0.0),
            ResourceType::Iron,
            nation_id,
        );
        // Raise infra on frontline to level 3
        if let Some(mut infra) = world.world.get_mut::<Infrastructure>(front_entity) {
            infra.level = 3;
        }
        let front_id = world.world.get::<Province>(front_entity).unwrap().id;

        world.add_province_border(cap_id, front_id);
        (cap_id, front_id, nation_id)
    }

    #[test]
    fn test_teeth_to_tail_ratio() {
        let mut world_state = WorldState::new(42);
        let (_, front_id, nation_id) = setup_two_province_nation(&mut world_state);

        // Find frontline province entity to add army
        let mut front_entity = None;
        {
            let mut q = world_state.world.query::<(Entity, &Province)>();
            for (e, p) in q.iter(&world_state.world) {
                if p.id == front_id {
                    front_entity = Some(e);
                }
            }
        }
        let front_entity = front_entity.expect("Frontline entity");

        // Spawn army of 10,000 at frontline (1 hop from capital)
        world_state.world.entity_mut(front_entity).insert(Army {
            owner: nation_id,
            personnel: 10_000,
            forced_march: false,
        });

        let init_personnel = 10_000u64;
        // Infra level 3 -> flow limit = 30_000. 10k + ~1k tail = 11k, well within limit.
        // No attrition expected.
        let mut phase = LogisticsPhase::new();
        phase.execute(&mut world_state.world);

        let army = world_state.world.get::<Army>(front_entity).unwrap();
        assert_eq!(army.personnel, init_personnel, "No attrition should occur when supply is sufficient");
    }

    #[test]
    fn test_infrastructure_bottleneck() {
        let mut world_state = WorldState::new(42);
        let (_, front_id, nation_id) = setup_two_province_nation(&mut world_state);

        let mut front_entity = None;
        {
            let mut q = world_state.world.query::<(Entity, &Province)>();
            for (e, p) in q.iter(&world_state.world) {
                if p.id == front_id {
                    front_entity = Some(e);
                }
            }
        }
        let front_entity = front_entity.expect("Frontline entity");

        // Throttle infra to level 1 (max flow = 10_000)
        if let Some(mut infra) = world_state.world.get_mut::<Infrastructure>(front_entity) {
            infra.level = 1;
        }

        // Spawn massive army: 50,000 men — well above infra limit
        world_state.world.entity_mut(front_entity).insert(Army {
            owner: nation_id,
            personnel: 50_000,
            forced_march: false,
        });

        let mut phase = LogisticsPhase::new();
        phase.execute(&mut world_state.world);

        let army = world_state.world.get::<Army>(front_entity).unwrap();
        assert!(army.personnel < 50_000, "Bottleneck should cause attrition on oversized army");
    }

    #[test]
    fn test_isolated_army_attrition() {
        let mut world_state = WorldState::new(42);
        let nation_entity = world_state.spawn_nation("Isolated".to_string(), [0, 0, 255], false);
        let nation_id = world_state.world.get::<Nation>(nation_entity).unwrap().id;

        // Province with NO capital connection
        let front = world_state.spawn_province(
            "Lonely".to_string(), Vec2::new(5.0, 5.0), ResourceType::Iron, nation_id,
        );
        world_state.world.entity_mut(front).insert(Army {
            owner: nation_id,
            personnel: 10_000,
            forced_march: false,
        });

        let mut phase = LogisticsPhase::new();
        phase.execute(&mut world_state.world);

        let army = world_state.world.get::<Army>(front).unwrap();
        assert!(army.personnel < 10_000, "Isolated army must suffer heavy attrition");
    }
}
