//! State serialization and snapshot management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

use super::world::WorldState;

/// Serializable snapshot of world state
#[derive(Debug, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tick: u64,
    pub seed: u64,
    pub metadata: super::world::WorldMetadata,
    // ECS data will be added as we build out components
}

impl WorldState {
    /// Save current state to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let snapshot = StateSnapshot {
            tick: self.tick,
            seed: self.seed,
            metadata: self.metadata.clone(),
        };

        let json = serde_json::to_string_pretty(&snapshot)
            .context("Failed to serialize state")?;
        
        fs::write(path, json)
            .context("Failed to write state file")?;

        Ok(())
    }

    /// Load state from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = fs::read_to_string(path)
            .context("Failed to read state file")?;
        
        let snapshot: StateSnapshot = serde_json::from_str(&json)
            .context("Failed to deserialize state")?;

        let mut world_state = WorldState::new(snapshot.seed);
        world_state.tick = snapshot.tick;
        world_state.metadata = snapshot.metadata;

        Ok(world_state)
    }

    /// Generate state hash for determinism verification
    pub fn state_hash(&mut self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use crate::core::types::{
            GDP, Legitimacy, Logistics, MilitaryCapacity, Nation, OwnedBy, Population, Province,
            Resources, WarState,
        };

        let mut hasher = DefaultHasher::new();
        self.tick.hash(&mut hasher);
        self.seed.hash(&mut hasher);

        let nation_name_by_id = {
            let mut query = self.world.query::<&Nation>();
            query
                .iter(&self.world)
                .map(|n| (n.id, n.name.clone()))
                .collect::<std::collections::HashMap<_, _>>()
        };

        let mut nation_rows = {
            let mut query = self.world.query::<(
                &Nation,
                &Legitimacy,
                &GDP,
                &MilitaryCapacity,
                &Logistics,
                &WarState,
            )>();

            query
                .iter(&self.world)
                .map(|(n, l, g, m, lo, w)| {
                    let mut enemies: Vec<String> = w
                        .at_war_with
                        .iter()
                        .map(|id| {
                            nation_name_by_id
                                .get(id)
                                .cloned()
                                .unwrap_or_else(|| "UNKNOWN".to_string())
                        })
                        .collect();
                    enemies.sort();
                    (
                        n.name.clone(),
                        l.value.to_bits(),
                        g.value.to_bits(),
                        m.value.to_bits(),
                        lo.value.to_bits(),
                        enemies,
                    )
                })
                .collect::<Vec<_>>()
        };
            nation_rows.sort_by(|a, b| a.0.cmp(&b.0));
        nation_rows.hash(&mut hasher);

        let mut province_rows = {
            let mut query = self
                .world
                .query::<(&Province, &OwnedBy, &Population, &Resources)>();

            query
                .iter(&self.world)
                .map(|(p, o, pop, res)| {
                    let owner_name = nation_name_by_id
                        .get(&o.nation_id)
                        .cloned()
                        .unwrap_or_else(|| "UNKNOWN".to_string());
                    (
                        p.name.clone(),
                        owner_name,
                        pop.total,
                        pop.growth_rate.to_bits(),
                        res.food.to_bits(),
                        res.iron.to_bits(),
                        res.oil.to_bits(),
                        res.rare_earths.to_bits(),
                        res.water.to_bits(),
                        res.trade_ports,
                    )
                })
                .collect::<Vec<_>>()
        };
            province_rows.sort_by(|a, b| a.0.cmp(&b.0));
        province_rows.hash(&mut hasher);
        
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_and_load() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        // Create and save a world
        let mut world = WorldState::new(42);
        world.advance_tick();
        world.advance_tick();
        
        world.save_to_file(path)?;

        // Load it back
        let loaded = WorldState::load_from_file(path)?;
        
        assert_eq!(loaded.tick, 2);
        assert_eq!(loaded.seed, 42);

        Ok(())
    }

    #[test]
    fn test_state_hash_determinism() {
        let mut world1 = WorldState::new(42);
        let mut world2 = WorldState::new(42);

        assert_eq!(world1.state_hash(), world2.state_hash());
    }
}
