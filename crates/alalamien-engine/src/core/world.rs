//! World state management using bevy_ecs

use bevy_ecs::prelude::*;
use chrono::{Datelike, Days, NaiveDate};
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
    /// Game clock configuration/state
    pub game_clock: GameClock,
    /// Metadata
    pub metadata: WorldMetadata,
}

/// Game speed settings used by API/UI control
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameSpeed {
    Paused,
    Slow,
    Normal,
    Fast,
    VeryFast,
}

impl GameSpeed {
    pub fn ticks_per_step(self) -> u64 {
        match self {
            Self::Paused => 0,
            Self::Slow => 1,
            Self::Normal => 3,
            Self::Fast => 7,
            Self::VeryFast => 15,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Paused => "paused",
            Self::Slow => "slow",
            Self::Normal => "normal",
            Self::Fast => "fast",
            Self::VeryFast => "very_fast",
        }
    }
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self::Normal
    }
}

/// Calendar and pacing configuration for simulation time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameClock {
    pub start_year: i32,
    pub start_month: u32,
    pub start_day: u32,
    pub hours_per_tick: u32,
    pub speed: GameSpeed,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            start_year: 2010,
            start_month: 1,
            start_day: 1,
            hours_per_tick: 4,
            speed: GameSpeed::Normal,
        }
    }
}

impl GameClock {
    pub fn set_start_date(&mut self, year: i32, month: u32, day: u32) -> Result<(), &'static str> {
        if NaiveDate::from_ymd_opt(year, month, day).is_none() {
            return Err("invalid start date");
        }

        self.start_year = year;
        self.start_month = month;
        self.start_day = day;
        Ok(())
    }

    pub fn set_hours_per_tick(&mut self, hours_per_tick: u32) {
        self.hours_per_tick = hours_per_tick.max(1).min(24);
    }

    pub fn datetime_for_tick(&self, tick: Tick) -> (i32, u32, u32, u32, u32) {
        let start = NaiveDate::from_ymd_opt(self.start_year, self.start_month, self.start_day)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2010, 1, 1).expect("valid fallback date"));

        let elapsed_hours = tick.saturating_mul(self.hours_per_tick as u64);
        let elapsed_days = elapsed_hours / 24;
        let hour = (elapsed_hours % 24) as u32;

        let date = start
            .checked_add_days(Days::new(elapsed_days))
            .unwrap_or(start);

        (date.year(), date.month(), date.day(), hour, 0)
    }

    pub fn formatted_datetime_for_tick(&self, tick: Tick) -> String {
        let (year, month, day, hour, minute) = self.datetime_for_tick(tick);
        let month_name = match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        };

        format!("{:02}:{:02}, {} {}, {}", hour, minute, day, month_name, year)
    }
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
            game_clock: GameClock::default(),
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
            crate::core::types::EconomicStress::default(),
            crate::core::types::CasualtyLog::default(),
            crate::core::types::AllianceCrisisLog::default(),
            crate::core::types::DiplomaticIsolationLog::default(),
            Resources::default(),
            crate::core::types::MilitaryCapacity::default(),
            crate::core::types::Logistics::default(),
            crate::core::types::WarState::default(),
            crate::core::types::AIMemory::default(),
            crate::core::types::IntelligenceProfile::default(),
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

    pub fn current_datetime_string(&self) -> String {
        self.game_clock.formatted_datetime_for_tick(self.tick)
    }

    pub fn current_datetime(&self) -> (i32, u32, u32, u32, u32) {
        self.game_clock.datetime_for_tick(self.tick)
    }

    pub fn set_game_speed(&mut self, speed: GameSpeed) {
        self.game_clock.speed = speed;
    }

    pub fn speed_ticks_per_step(&self) -> u64 {
        self.game_clock.speed.ticks_per_step()
    }

    pub fn set_start_date(&mut self, year: i32, month: u32, day: u32) -> Result<(), &'static str> {
        self.game_clock.set_start_date(year, month, day)
    }

    pub fn set_hours_per_tick(&mut self, hours_per_tick: u32) {
        self.game_clock.set_hours_per_tick(hours_per_tick);
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

    // ========================================================================
    // V0.4 ALLIANCE MANAGEMENT
    // ========================================================================

    /// Spawn a new alliance entity
    pub fn spawn_alliance(
        &mut self,
        name: String,
        founding_nation: NationId,
        doctrine: crate::core::types::AllianceDoctrine,
        threat_reduction: f64,
        cohesion_decay_rate: f64,
    ) -> Entity {
        let alliance = Alliance {
            alliance_id: AllianceId::new(),
            alliance_name: name,
            founding_nation,
            members: vec![founding_nation],
            cohesion: 100.0,
            doctrine,
            founded_tick: self.tick,
            threat_reduction,
            cohesion_decay_rate,
        };

        self.world.spawn(alliance).id()
    }

    /// Get an alliance by entity
    pub fn get_alliance(&self, alliance_entity: Entity) -> Option<&Alliance> {
        self.world.get::<Alliance>(alliance_entity)
    }

    /// Get a mutable reference to an alliance
    pub fn get_alliance_mut(&mut self, alliance_entity: Entity) -> Option<bevy_ecs::world::Mut<Alliance>> {
        self.world.get_mut::<Alliance>(alliance_entity)
    }

    /// Get all alliances a nation is a member of
    pub fn get_nation_alliances(&mut self, nation_id: NationId) -> Vec<(Entity, &Alliance)> {
        let mut query = self.world.query::<&Alliance>();
        query
            .iter(&self.world)
            .filter_map(|alliance| {
                if alliance.members.contains(&nation_id) {
                    Some((Entity::PLACEHOLDER, alliance))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Add a member to an existing alliance
    pub fn add_alliance_member(&mut self, alliance_entity: Entity, nation_id: NationId) -> bool {
        if let Some(mut alliance) = self.world.get_mut::<Alliance>(alliance_entity) {
            if !alliance.members.contains(&nation_id) {
                alliance.add_member(nation_id);
                return true;
            }
        }
        false
    }

    /// Remove a member from an alliance
    pub fn remove_alliance_member(&mut self, alliance_entity: Entity, nation_id: NationId) -> bool {
        if let Some(mut alliance) = self.world.get_mut::<Alliance>(alliance_entity) {
            let original_len = alliance.member_count();
            alliance.remove_member(nation_id);
            return alliance.member_count() < original_len;
        }
        false
    }

    /// Get all active alliances
    pub fn get_all_alliances(&mut self) -> Vec<&Alliance> {
        let mut query = self.world.query::<&Alliance>();
        query.iter(&self.world).collect()
    }

    /// Decay cohesion for all alliances (call once per tick)
    pub fn update_alliance_cohesion(&mut self) {
        let mut query = self.world.query::<&mut Alliance>();
        for mut alliance in query.iter_mut(&mut self.world) {
            alliance.decay_cohesion();
        }
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
        assert_eq!(world.game_clock.start_year, 2010);
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

    #[test]
    fn test_clock_progression() {
        let mut world = WorldState::new(42);
        assert_eq!(world.current_datetime_string(), "00:00, 1 January, 2010");

        world.advance_tick();
        assert_eq!(world.current_datetime_string(), "04:00, 1 January, 2010");
    }

    #[test]
    fn test_alliance_spawn() {
        let mut world = WorldState::new(42);
        let nation = world.spawn_nation("TestNation".to_string(), [255, 0, 0], true);
        let nation_id = world.world.get::<Nation>(nation).unwrap().id;

        let alliance_entity = world.spawn_alliance(
            "Test Alliance".to_string(),
            nation_id,
            crate::core::types::AllianceDoctrine::DefensiveAgreement,
            0.25,
            1.0,
        );

        let alliance = world.get_alliance(alliance_entity).unwrap();
        assert_eq!(alliance.alliance_name, "Test Alliance");
        assert_eq!(alliance.member_count(), 1);
        assert!(alliance.members.contains(&nation_id));
    }

    #[test]
    fn test_alliance_member_add_remove() {
        let mut world = WorldState::new(42);
        let nation1 = world.spawn_nation("Nation1".to_string(), [255, 0, 0], true);
        let nation2 = world.spawn_nation("Nation2".to_string(), [0, 255, 0], true);
        
        let nation_id1 = world.world.get::<Nation>(nation1).unwrap().id;
        let nation_id2 = world.world.get::<Nation>(nation2).unwrap().id;

        let alliance_entity = world.spawn_alliance(
            "Test Alliance".to_string(),
            nation_id1,
            crate::core::types::AllianceDoctrine::EconomicBloc,
            0.20,
            0.8,
        );

        // Add member
        assert!(world.add_alliance_member(alliance_entity, nation_id2));
        let alliance = world.get_alliance(alliance_entity).unwrap();
        assert_eq!(alliance.member_count(), 2);

        // Remove member
        assert!(world.remove_alliance_member(alliance_entity, nation_id2));
        let alliance = world.get_alliance(alliance_entity).unwrap();
        assert_eq!(alliance.member_count(), 1);
    }

    #[test]
    fn test_alliance_cohesion_decay() {
        let mut world = WorldState::new(42);
        let nation = world.spawn_nation("TestNation".to_string(), [255, 0, 0], true);
        let nation_id = world.world.get::<Nation>(nation).unwrap().id;

        let alliance_entity = world.spawn_alliance(
            "Test Alliance".to_string(),
            nation_id,
            crate::core::types::AllianceDoctrine::DefensiveAgreement,
            0.25,
            2.0, // High decay rate
        );

        let initial_cohesion = world.get_alliance(alliance_entity).unwrap().cohesion;
        world.update_alliance_cohesion();
        let new_cohesion = world.get_alliance(alliance_entity).unwrap().cohesion;

        assert!(new_cohesion < initial_cohesion);
        assert!((initial_cohesion - new_cohesion - 2.0).abs() < 0.01);
    }
}
