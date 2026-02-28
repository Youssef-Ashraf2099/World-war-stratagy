//! World state management using bevy_ecs

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use super::types::*;
use super::province_graph::ProvinceGraph;

/// Central world state container
#[derive(Default)]
pub struct WorldState {
    /// ECS world instance
    pub world: World,
    /// Current game tick
    pub tick: Tick,
    /// Simulation seed for determinism
    pub seed: u64,
    /// Metadata
    pub metadata: WorldMetadata,
}

/// World metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    pub name: String,
    pub created_at: String,
    pub version: String,
}

impl Default for WorldMetadata {
    fn default() -> Self {
        Self {
            name: "New World".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            version: crate::VERSION.to_string(),
        }
    }
}

impl WorldState {
    /// Create a new world state with a seed
    pub fn new(seed: u64) -> Self {
        let mut world = World::default();
        
        // Initialize province graph resource
        world.insert_resource(ProvinceGraph::new());
        
        Self {
            world,
            tick: 0,
            seed,
            metadata: WorldMetadata::default(),
        }
    }

    /// Initialize world from Natural Earth nations data
    pub fn from_geodata(seed: u64, nations_json_path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        use std::collections::HashMap;

        use crate::game::{NationData, borders};
        
        let mut world = Self::new(seed);
        let nations = NationData::load_all(nations_json_path)?;
        let mut nation_name_to_id: HashMap<String, NationId> = HashMap::new();
        let mut nation_id_to_province: HashMap<NationId, ProvinceId> = HashMap::new();
        
        // Spawn a nation for each loaded record
        for (idx, nation_data) in nations.iter().enumerate() {
            // Generate a unique color based on continent hash
            let color = Self::color_from_continent(&nation_data.continent, idx);
            
            // Spawn the nation
            let nation_entity = world.spawn_nation(
                nation_data.name.clone(),
                color,
                false, // AI controlled
            );
            
            // Set GDP based on loaded data
            if let Some(mut gdp) = world.world.get_mut::<GDP>(nation_entity) {
                gdp.value = (nation_data.gdp as f64) / 1_000_000_000.0; // Convert to billions
            }

            let nation_id = world
                .world
                .get::<Nation>(nation_entity)
                .map(|n| n.id)
                .ok_or("Failed to read spawned nation")?;

            nation_name_to_id.insert(nation_data.name.clone(), nation_id);
            if let Some(formal_name) = &nation_data.formal_name {
                nation_name_to_id.insert(formal_name.clone(), nation_id);
            }

            // Spawn one bootstrap province per nation for adjacency/trade/logistics wiring.
            let dominant_resource = match idx % 3 {
                0 => ResourceType::Food,
                1 => ResourceType::Iron,
                _ => ResourceType::Oil,
            };

            let province_entity = world.spawn_province(
                format!("{} Core", nation_data.name),
                glam::Vec2::new(idx as f32, 0.0),
                dominant_resource,
                nation_id,
            );

            world.world.entity_mut(province_entity).insert(Capital);

            let province_id = world
                .world
                .get::<Province>(province_entity)
                .map(|p| p.id)
                .ok_or("Failed to read spawned province")?;

            nation_id_to_province.insert(nation_id, province_id);
        }

        // Alias common Natural Earth naming differences
        if let Some(id) = nation_name_to_id.get("United States of America").copied() {
            nation_name_to_id.insert("United States".to_string(), id);
        }

        // Build province borders from precomputed nation borders
        for (country_a, country_b) in borders::get_nation_borders() {
            let a_id = nation_name_to_id.get(country_a).copied();
            let b_id = nation_name_to_id.get(country_b).copied();
            if let (Some(nation_a), Some(nation_b)) = (a_id, b_id) {
                if let (Some(province_a), Some(province_b)) = (
                    nation_id_to_province.get(&nation_a),
                    nation_id_to_province.get(&nation_b),
                ) {
                    world.add_province_border(*province_a, *province_b);
                }
            }
        }
        
        world.metadata.name = format!("World ({} nations)", nations.len());
        Ok(world)
    }
    
    /// Generate a color from continent name and index
    fn color_from_continent(continent: &str, idx: usize) -> [u8; 3] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        continent.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Create deterministic colors per continent
        match continent {
            "Africa" => {
                let offset = idx.wrapping_mul(13) % 256;
                [200 ^ (offset as u8), 100 ^ ((offset >> 1) as u8), 50]
            }
            "Asia" => {
                let offset = idx.wrapping_mul(17) % 256;
                [100 ^ (offset as u8), 150 ^ ((offset >> 1) as u8), 200]
            }
            "Europe" => {
                let offset = idx.wrapping_mul(19) % 256;
                [200 ^ (offset as u8), 150 ^ ((offset >> 1) as u8), 100]
            }
            "North America" => {
                let offset = idx.wrapping_mul(23) % 256;
                [50, 100 ^ (offset as u8), 200 ^ ((offset >> 1) as u8)]
            }
            "South America" => {
                let offset = idx.wrapping_mul(29) % 256;
                [100 ^ (offset as u8), 200 ^ ((offset >> 1) as u8), 50]
            }
            "Oceania" => {
                let offset = idx.wrapping_mul(31) % 256;
                [200, 100 ^ (offset as u8), 100 ^ ((offset >> 1) as u8)]
            }
            _ => {
                let offset = ((hash as usize).wrapping_mul(idx)) % 256;
                [(hash as u8) ^ (offset as u8), ((hash >> 8) as u8) ^ ((offset >> 1) as u8), ((hash >> 16) as u8)]
            }
        }
    }

    /// Create a nation entity
    pub fn spawn_nation(
        &mut self,
        name: String,
        color: [u8; 3],
        player_controlled: bool,
    ) -> Entity {
        let nation_id = NationId::new();
        let nation = Nation {
            id: nation_id,
            name,
            color,
        };

        let mut entity = self.world.spawn((
            nation,
            Legitimacy::default(),
            GDP::default(),
            Resources::default(),
            crate::core::types::MilitaryCapacity::default(),
            crate::core::types::Logistics::default(),
            crate::core::types::WarState::default(),
        ));

        if player_controlled {
            entity.insert(PlayerControlled);
        } else {
            entity.insert(AIControlled);
        }

        entity.id()
    }

    /// Create a province entity
    pub fn spawn_province(
        &mut self,
        name: String,
        position: glam::Vec2,
        dominant_resource: ResourceType,
        owner: NationId,
    ) -> Entity {
        let province_id = ProvinceId::new();
        let province = Province {
            id: province_id,
            name,
            position,
            dominant_resource,
        };

        self.world
            .spawn((
                province,
                Population::default(),
                Infrastructure::default(),
                Resources::default(),
                OwnedBy { nation_id: owner },
            ))
            .id()
    }

    /// Get total number of nations
    pub fn nation_count(&mut self) -> usize {
        let mut query = self.world.query::<&Nation>();
        query.iter(&self.world).count()
    }

    /// Get total number of provinces
    pub fn province_count(&mut self) -> usize {
        let mut query = self.world.query::<&Province>();
        query.iter(&self.world).count()
    }

    /// Advance the tick counter
    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Get current tick
    pub fn current_tick(&self) -> Tick {
        self.tick
    }

    /// Add a border between two provinces
    pub fn add_province_border(&mut self, province_a: ProvinceId, province_b: ProvinceId) {
        if let Some(mut graph) = self.world.get_resource_mut::<ProvinceGraph>() {
            graph.add_border(province_a, province_b);
        }
    }

    /// Get neighbors of a province
    pub fn get_province_neighbors(&self, province_id: ProvinceId) -> Vec<ProvinceId> {
        self.world
            .get_resource::<ProvinceGraph>()
            .map(|graph| graph.get_neighbors(province_id))
            .unwrap_or_default()
    }

    /// Check if two provinces are neighbors
    pub fn are_provinces_neighbors(&self, province_a: ProvinceId, province_b: ProvinceId) -> bool {
        self.world
            .get_resource::<ProvinceGraph>()
            .map(|graph| graph.are_neighbors(province_a, province_b))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = WorldState::new(42);
        assert_eq!(world.tick, 0);
        assert_eq!(world.seed, 42);
    }

    #[test]
    fn test_spawn_nation() {
        let mut world = WorldState::new(42);
        world.spawn_nation("TestNation".to_string(), [255, 0, 0], true);
        
        assert_eq!(world.nation_count(), 1);
    }

    #[test]
    fn test_spawn_province() {
        let mut world = WorldState::new(42);
        let nation_entity = world.spawn_nation("TestNation".to_string(), [255, 0, 0], true);
        
        // Get the NationId from the spawned entity
        let nation_id = world.world
            .get::<Nation>(nation_entity)
            .unwrap()
            .id;
        
        world.spawn_province(
            "TestProvince".to_string(),
            glam::Vec2::new(0.0, 0.0),
            ResourceType::Food,
            nation_id,
        );
        
        assert_eq!(world.province_count(), 1);
    }

    #[test]
    fn test_tick_advancement() {
        let mut world = WorldState::new(42);
        assert_eq!(world.current_tick(), 0);
        
        world.advance_tick();
        assert_eq!(world.current_tick(), 1);
        
        world.advance_tick();
        assert_eq!(world.current_tick(), 2);
    }
}
