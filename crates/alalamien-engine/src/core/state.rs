//! State serialization and snapshot management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

use super::world::WorldState;

/// Serializable snapshot of an alliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceSnapshot {
    pub alliance_id: String,  // UUID as string
    pub alliance_name: String,
    pub members: Vec<String>,            // Nation IDs as strings
    pub cohesion: f64,
    pub doctrine: String,                // AllianceDoctrine serialized
    pub founded_tick: u64,
    pub threat_reduction: f64,
    pub cohesion_decay_rate: f64,
}

/// Serializable snapshot of diplomatic relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticSnapshot {
    pub nation_a: String,
    pub nation_b: String,
    pub reputation: f64,
    pub trade_dependency: f64,
    pub threat_alignment: f64,
    pub last_war: Option<u64>,
    pub allied_since: Option<u64>,
    pub last_updated: u64,
}

/// Serializable snapshot of world state
#[derive(Debug, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tick: u64,
    pub seed: u64,
    pub game_clock: super::world::GameClock,
    pub metadata: super::world::WorldMetadata,
    // ECS data will be added as we build out components
    #[serde(default)]
    pub alliances: Vec<AllianceSnapshot>,
    #[serde(default)]
    pub diplomatic_relations: Vec<DiplomaticSnapshot>,
}

impl WorldState {
    /// Save current state to JSON file
    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        use crate::core::types::Alliance;

        // Serialize alliances
        let alliances = {
            let mut query = self.world.query::<&Alliance>();
            query
                .iter(&self.world)
                .map(|a| AllianceSnapshot {
                    alliance_id: a.alliance_id.0.to_string(),
                    alliance_name: a.alliance_name.clone(),
                    members: a.members.iter().map(|n| n.0.to_string()).collect(),
                    cohesion: a.cohesion,
                    doctrine: a.doctrine.as_str().to_string(),
                    founded_tick: a.founded_tick,
                    threat_reduction: a.threat_reduction,
                    cohesion_decay_rate: a.cohesion_decay_rate,
                })
                .collect()
        };

        let snapshot = StateSnapshot {
            tick: self.tick,
            seed: self.seed,
            game_clock: self.game_clock.clone(),
            metadata: self.metadata.clone(),
            alliances,
            diplomatic_relations: Vec::new(), // TODO: Serialize DiplomaticRelation when integrated
        };

        let json = serde_json::to_string_pretty(&snapshot)
            .context("Failed to serialize state")?;
        
        fs::write(path, json)
            .context("Failed to write state file")?;

        Ok(())
    }

    /// Load state from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        use crate::core::types::{Alliance, AllianceId, NationId, AllianceDoctrine};
        
        let json = fs::read_to_string(path)
            .context("Failed to read state file")?;
        
        let snapshot: StateSnapshot = serde_json::from_str(&json)
            .context("Failed to deserialize state")?;

        let mut world_state = WorldState::new(snapshot.seed);
        world_state.tick = snapshot.tick;
        world_state.game_clock = snapshot.game_clock;
        world_state.metadata = snapshot.metadata;

        // Restore alliances
        for alliance_snap in snapshot.alliances {
            let doctrine = match alliance_snap.doctrine.as_str() {
                "DefensiveAgreement" => AllianceDoctrine::DefensiveAgreement,
                "OffensivePact" => AllianceDoctrine::OffensivePact,
                "EconomicBloc" => AllianceDoctrine::EconomicBloc,
                "ResearchConsortium" => AllianceDoctrine::ResearchConsortium,
                "BalanceOfPower" => AllianceDoctrine::BalanceOfPower,
                _ => AllianceDoctrine::DefensiveAgreement,
            };

            let members = alliance_snap
                .members
                .iter()
                .filter_map(|m| {
                    uuid::Uuid::parse_str(m)
                        .ok()
                        .map(|u| NationId(u))
                })
                .collect();

            let alliance = Alliance {
                alliance_id: uuid::Uuid::parse_str(&alliance_snap.alliance_id)
                    .map(AllianceId)
                    .unwrap_or_else(|_| AllianceId::new()),
                alliance_name: alliance_snap.alliance_name,
                founding_nation: NationId::default(),
                members,
                cohesion: alliance_snap.cohesion,
                doctrine,
                founded_tick: alliance_snap.founded_tick,
                threat_reduction: alliance_snap.threat_reduction,
                cohesion_decay_rate: alliance_snap.cohesion_decay_rate,
            };

            world_state.world.spawn(alliance);
        }

        Ok(world_state)
    }

    /// Generate state hash for determinism verification
    pub fn state_hash(&mut self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use crate::core::types::{
            GDP, Legitimacy, Logistics, MilitaryCapacity, Nation, OwnedBy, Population, Province,
            Resources, WarState, Alliance,
        };

        let mut hasher = DefaultHasher::new();
        self.tick.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        self.game_clock.start_year.hash(&mut hasher);
        self.game_clock.start_month.hash(&mut hasher);
        self.game_clock.start_day.hash(&mut hasher);
        self.game_clock.hours_per_tick.hash(&mut hasher);
        self.game_clock.speed.as_str().hash(&mut hasher);

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

        // Hash alliance data for determinism
        let mut alliance_rows = {
            let mut query = self.world.query::<&Alliance>();
            query
                .iter(&self.world)
                .map(|a| {
                    let member_names: Vec<String> = a
                        .members
                        .iter()
                        .map(|m| {
                            nation_name_by_id
                                .get(m)
                                .cloned()
                                .unwrap_or_else(|| "UNKNOWN".to_string())
                        })
                        .collect();
                    (
                        a.alliance_name.clone(),
                        a.cohesion.to_bits(),
                        a.doctrine.as_str().to_string(),
                        member_names,
                    )
                })
                .collect::<Vec<_>>()
        };
            alliance_rows.sort_by(|a, b| a.0.cmp(&b.0));
        alliance_rows.hash(&mut hasher);
        
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
        world.set_start_date(2015, 6, 10).unwrap();
        world.set_hours_per_tick(6);
        world.set_game_speed(super::super::world::GameSpeed::Fast);
        world.advance_tick();
        world.advance_tick();
        
        world.save_to_file(path)?;

        // Load it back
        let loaded = WorldState::load_from_file(path)?;
        
        assert_eq!(loaded.tick, 2);
        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.game_clock.start_year, 2015);
        assert_eq!(loaded.game_clock.hours_per_tick, 6);
        assert_eq!(loaded.game_clock.speed, super::super::world::GameSpeed::Fast);

        Ok(())
    }

    #[test]
    fn test_state_hash_determinism() {
        let mut world1 = WorldState::new(42);
        let mut world2 = WorldState::new(42);

        assert_eq!(world1.state_hash(), world2.state_hash());
    }
}
