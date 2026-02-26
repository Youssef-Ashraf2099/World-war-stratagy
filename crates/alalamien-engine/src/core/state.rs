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
    pub fn state_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.tick.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        
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
        let world1 = WorldState::new(42);
        let world2 = WorldState::new(42);

        assert_eq!(world1.state_hash(), world2.state_hash());
    }
}
