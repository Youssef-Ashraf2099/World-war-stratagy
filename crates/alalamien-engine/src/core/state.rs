//! State serialization and snapshot management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

use super::world::WorldState;

/// Auto-save configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    /// Enable auto-save
    pub enabled: bool,
    /// Save every N ticks
    pub interval_ticks: u64,
    /// Maximum number of auto-save slots to maintain
    pub max_slots: usize,
    /// Directory for auto-saves
    pub save_directory: PathBuf,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ticks: 100,  // Auto-save every 100 ticks
            max_slots: 3,          // Keep last 3 auto-saves
            save_directory: PathBuf::from("saves/auto"),
        }
    }
}

/// Serializable snapshot of a nation and all its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationSnapshot {
    pub entity_id: String,  // Bevy Entity ID for restoration
    pub nation_id: String,  // Nation UUID
    pub name: String,
    pub color: [u8; 3],
    pub legitimacy: f64,
    pub gdp: f64,
    pub gdp_growth_rate: f64,
    pub military_capacity: f64,
    pub logistics: f64,
    pub war_exhaustion: f64,
    pub at_war_with: Vec<String>,  // NationId UUIDs
    pub ai_personality: Option<String>,
    pub ai_memory: Option<AIMemorySnapshot>,
    pub economic_stress: Option<EconomicStressSnapshot>,
    pub casualty_log: Option<CasualtyLogSnapshot>,
    pub alliance_crisis_log: Option<AllianceCrisisSnapshot>,
    pub diplomatic_isolation_log: Option<DiplomaticIsolationSnapshot>,
    pub is_player: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMemorySnapshot {
    pub recent_enemies: Vec<String>,
    pub successful_wars: u32,
    pub failed_wars: u32,
    pub last_decision_tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicStressSnapshot {
    pub current_deficit: f64,
    pub accumulated_deficit: f64,
    pub gdp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasualtyLogSnapshot {
    pub personnel_lost: u64,
    pub total_personnel: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceCrisisSnapshot {
    pub alliances_in_crisis: u32,
    pub total_alliances: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiplomaticIsolationSnapshot {
    pub hostile_relations: u32,
    pub friendly_relations: u32,
    pub total_relations: u32,
}

/// Serializable snapshot of a province and all its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceSnapshot {
    pub entity_id: String,
    pub province_id: String,
    pub name: String,
    pub position_x: f32,
    pub position_y: f32,
    pub dominant_resource: String,
    pub owner: String,  // NationId UUID
    pub population: u64,
    pub growth_rate: f64,
    pub food: f64,
    pub iron: f64,
    pub oil: f64,
    pub rare_earths: f64,
    pub water: f64,
    pub trade_ports: u32,
    pub is_capital: bool,
    pub is_occupied: bool,
    pub occupied_by: Option<String>,
    pub original_owner: Option<String>,
    pub occupation_tick: Option<u64>,
    pub resistance: Option<f64>,
}

/// Serializable snapshot of an army
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmySnapshot {
    pub entity_id: String,
    pub army_id: String,
    pub owner: String,
    pub location: String,  // ProvinceId UUID
    pub infantry: u64,
    pub armor: u64,
    pub artillery: u64,
    pub morale: f64,
    pub organization: f64,
    pub supply_state: f64,
    pub entrenchment: f64,
    pub movement_points: f64,
    pub destination: Option<String>,
}

/// Serializable snapshot of a war declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarDeclarationSnapshot {
    pub entity_id: String,
    pub war_id: String,
    pub aggressor: String,
    pub defender: String,
    pub casus_belli: String,
    pub war_goal: String,
    pub declared_tick: u64,
}

/// Serializable snapshot of a peace treaty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceTreatySnapshot {
    pub entity_id: String,
    pub war_id: String,
    pub victor: Option<String>,
    pub provinces_transferred: Vec<(String, String)>,
    pub war_reparations: f64,
    pub cannot_redeclare_until: u64,
    pub signed_tick: u64,
}

/// Serializable snapshot of an alliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllianceSnapshot {
    pub entity_id: String,
    pub alliance_id: String,  // UUID as string
    pub alliance_name: String,
    pub members: Vec<String>,  // Nation IDs as strings
    pub cohesion: f64,
    pub doctrine: String,      // AllianceDoctrine serialized
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

/// Serializable snapshot of complete world state
#[derive(Debug, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tick: u64,
    pub seed: u64,
    pub game_clock: super::world::GameClock,
    pub metadata: super::world::WorldMetadata,
    #[serde(default)]
    pub nations: Vec<NationSnapshot>,
    #[serde(default)]
    pub provinces: Vec<ProvinceSnapshot>,
    #[serde(default)]
    pub armies: Vec<ArmySnapshot>,
    #[serde(default)]
    pub war_declarations: Vec<WarDeclarationSnapshot>,
    #[serde(default)]
    pub peace_treaties: Vec<PeaceTreatySnapshot>,
    #[serde(default)]
    pub alliances: Vec<AllianceSnapshot>,
    #[serde(default)]
    pub diplomatic_relations: Vec<DiplomaticSnapshot>,
}

impl WorldState {
    /// Save current state to JSON file with complete ECS data
    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        use crate::core::types::*;
        use bevy_ecs::entity::Entity;

        // Serialize nations with all components
        let nations = {
            let mut query = self.world.query::<(
                Entity,
                &Nation,
                &Legitimacy,
                &GDP,
                &MilitaryCapacity,
                &Logistics,
                &WarState,
                Option<&WarExhaustion>,
                Option<&AIPersonality>,
                Option<&AIMemory>,
                Option<&EconomicStress>,
                Option<&CasualtyLog>,
                Option<&AllianceCrisisLog>,
                Option<&DiplomaticIsolationLog>,
                Option<&PlayerControlled>,
            )>();

            query
                .iter(&self.world)
                .map(
                    |(entity, nation, legit, gdp, mil, log, war, war_ex, ai_pers, ai_mem, econ, cas, alliance_crisis, diplo_iso, player)| {
                        NationSnapshot {
                            entity_id: format!("{:?}", entity),
                            nation_id: nation.id.0.to_string(),
                            name: nation.name.clone(),
                            color: nation.color,
                            legitimacy: legit.value,
                            gdp: gdp.value,
                            gdp_growth_rate: gdp.growth_rate,
                            military_capacity: mil.value,
                            logistics: log.value,
                            war_exhaustion: war_ex.map(|w| w.value).unwrap_or(0.0),
                            at_war_with: war.at_war_with.iter().map(|n| n.0.to_string()).collect(),
                            ai_personality: ai_pers.map(|p| match p {
                                AIPersonality::Defensive => "Defensive",
                                AIPersonality::Balanced => "Balanced",
                                AIPersonality::Aggressive => "Aggressive",
                            }.to_string()),
                            ai_memory: ai_mem.map(|m| AIMemorySnapshot {
                                recent_enemies: m.recent_enemies.iter().map(|n| n.0.to_string()).collect(),
                                successful_wars: m.successful_wars,
                                failed_wars: m.failed_wars,
                                last_decision_tick: m.last_decision_tick,
                            }),
                            economic_stress: econ.map(|e| EconomicStressSnapshot {
                                current_deficit: e.current_deficit,
                                accumulated_deficit: e.accumulated_deficit,
                                gdp: e.gdp,
                            }),
                            casualty_log: cas.map(|c| CasualtyLogSnapshot {
                                personnel_lost: c.personnel_lost,
                                total_personnel: c.total_personnel,
                            }),
                            alliance_crisis_log: alliance_crisis.map(|a| AllianceCrisisSnapshot {
                                alliances_in_crisis: a.alliances_in_crisis,
                                total_alliances: a.total_alliances,
                            }),
                            diplomatic_isolation_log: diplo_iso.map(|d| DiplomaticIsolationSnapshot {
                                hostile_relations: d.hostile_relations,
                                friendly_relations: d.friendly_relations,
                                total_relations: d.total_relations,
                            }),
                            is_player: player.is_some(),
                        }
                    },
                )
                .collect()
        };

        // Serialize provinces with all components
        let provinces = {
            let mut query = self.world.query::<(
                Entity,
                &Province,
                &OwnedBy,
                &Population,
                &Resources,
                Option<&Capital>,
                Option<&OccupiedProvince>,
            )>();

            query
                .iter(&self.world)
                .map(|(entity, prov, owner, pop, res, capital, occupied)| {
                    let resource_type_str = match prov.dominant_resource {
                        ResourceType::Food => "Food",
                        ResourceType::Iron => "Iron",
                        ResourceType::Oil => "Oil",
                        ResourceType::RareEarths => "RareEarths",
                        ResourceType::Water => "Water",
                        ResourceType::TradePorts => "TradePorts",
                    };
                    ProvinceSnapshot {
                        entity_id: format!("{:?}", entity),
                        province_id: prov.id.0.to_string(),
                        name: prov.name.clone(),
                        position_x: prov.position.x,
                        position_y: prov.position.y,
                        dominant_resource: resource_type_str.to_string(),
                        owner: owner.nation_id.0.to_string(),
                        population: pop.total,
                        growth_rate: pop.growth_rate,
                        food: res.food,
                        iron: res.iron,
                        oil: res.oil,
                        rare_earths: res.rare_earths,
                        water: res.water,
                        trade_ports: res.trade_ports,
                        is_capital: capital.is_some(),
                        is_occupied: occupied.is_some(),
                        occupied_by: occupied.map(|o| o.occupier.0.to_string()),
                        original_owner: occupied.map(|o| o.original_owner.0.to_string()),
                        occupation_tick: occupied.map(|o| o.occupation_tick),
                        resistance: occupied.map(|o| o.resistance),
                    }
                })
                .collect()
        };

        // Serialize armies
        let armies = {
            let mut query = self.world.query::<(Entity, &Army)>();
            query
                .iter(&self.world)
                .map(|(entity, army)| ArmySnapshot {
                    entity_id: format!("{:?}", entity),
                    army_id: army.army_id.0.to_string(),
                    owner: army.owner.0.to_string(),
                    location: army.location.0.to_string(),
                    infantry: army.infantry,
                    armor: army.armor,
                    artillery: army.artillery,
                    morale: army.morale,
                    organization: army.organization,
                    supply_state: army.supply_state,
                    entrenchment: army.entrenchment,
                    movement_points: army.movement_points,
                    destination: army.destination.map(|d| d.0.to_string()),
                })
                .collect()
        };

        // Serialize war declarations
        let war_declarations = {
            let mut query = self.world.query::<(Entity, &WarDeclaration)>();
            query
                .iter(&self.world)
                .map(|(entity, war)| {
                    let casus_belli_str = match &war.casus_belli {
                        CasusBelli::TerritorialDispute(p) => format!("TerritorialDispute:{}", p.0),
                        CasusBelli::ResourceConflict => "ResourceConflict".to_string(),
                        CasusBelli::PreemptiveStrike => "PreemptiveStrike".to_string(),
                        CasusBelli::Liberation(p) => format!("Liberation:{}", p.0),
                    };
                    let war_goal_str = match &war.war_goal {
                        WarGoal::ConquerProvince(p) => format!("ConquerProvince:{}", p.0),
                        WarGoal::Humiliate => "Humiliate".to_string(),
                        WarGoal::Total => "Total".to_string(),
                    };
                    WarDeclarationSnapshot {
                        entity_id: format!("{:?}", entity),
                        war_id: war.war_id.0.to_string(),
                        aggressor: war.aggressor.0.to_string(),
                        defender: war.defender.0.to_string(),
                        casus_belli: casus_belli_str,
                        war_goal: war_goal_str,
                        declared_tick: war.declared_tick,
                    }
                })
                .collect()
        };

        // Serialize peace treaties
        let peace_treaties = {
            let mut query = self.world.query::<(Entity, &PeaceTreaty)>();
            query
                .iter(&self.world)
                .map(|(entity, peace)| PeaceTreatySnapshot {
                    entity_id: format!("{:?}", entity),
                    war_id: peace.war_id.0.to_string(),
                    victor: peace.victor.map(|v| v.0.to_string()),
                    provinces_transferred: peace
                        .terms
                        .provinces_transferred
                        .iter()
                        .map(|(p, n)| (p.0.to_string(), n.0.to_string()))
                        .collect(),
                    war_reparations: peace.terms.war_reparations,
                    cannot_redeclare_until: peace.terms.cannot_redeclare_until,
                    signed_tick: peace.signed_tick,
                })
                .collect()
        };

        // Serialize alliances
        let alliances = {
            let mut query = self.world.query::<(Entity, &Alliance)>();
            query
                .iter(&self.world)
                .map(|(entity, a)| AllianceSnapshot {
                    entity_id: format!("{:?}", entity),
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
            nations,
            provinces,
            armies,
            war_declarations,
            peace_treaties,
            alliances,
            diplomatic_relations: Vec::new(), // TODO: Serialize DiplomaticRelation when integrated
        };

        let json = serde_json::to_string_pretty(&snapshot)
            .context("Failed to serialize state")?;
        
        // Ensure directory exists
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).context("Failed to create save directory")?;
        }
        
        fs::write(path, json)
            .context("Failed to write state file")?;

        Ok(())
    }

    /// Load state from JSON file with complete ECS restoration
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        use crate::core::types::*;
        use std::collections::HashMap;
        
        let json = fs::read_to_string(path)
            .context("Failed to read state file")?;
        
        let snapshot: StateSnapshot = serde_json::from_str(&json)
            .context("Failed to deserialize state")?;

        let mut world_state = WorldState::new(snapshot.seed);
        world_state.tick = snapshot.tick;
        world_state.game_clock = snapshot.game_clock;
        world_state.metadata = snapshot.metadata;

        // Track entity ID mappings for cross-references
        let mut nation_id_to_entity = HashMap::new();
        let mut province_id_to_entity = HashMap::new();

        // Restore nations
        for nation_snap in snapshot.nations {
            let nation_id = uuid::Uuid::parse_str(&nation_snap.nation_id)
                .ok()
                .map(NationId)
                .unwrap_or_else(|| NationId::new());

            let mut entity_builder = world_state.world.spawn((
                Nation {
                    id: nation_id,
                    name: nation_snap.name.clone(),
                    color: nation_snap.color,
                },
                Legitimacy {
                    value: nation_snap.legitimacy,
                },
                GDP {
                    value: nation_snap.gdp,
                    growth_rate: nation_snap.gdp_growth_rate,
                },
                MilitaryCapacity {
                    value: nation_snap.military_capacity,
                },
                Logistics {
                    value: nation_snap.logistics,
                },
                WarState {
                    at_war_with: nation_snap
                        .at_war_with
                        .iter()
                        .filter_map(|s| uuid::Uuid::parse_str(s).ok().map(NationId))
                        .collect(),
                },
            ));

            // Add optional components
            if nation_snap.war_exhaustion > 0.0 {
                entity_builder.insert(WarExhaustion {
                    value: nation_snap.war_exhaustion,
                });
            }

            if let Some(ai_pers_str) = nation_snap.ai_personality {
                let personality = match ai_pers_str.as_str() {
                    "Defensive" => AIPersonality::Defensive,
                    "Aggressive" => AIPersonality::Aggressive,
                    _ => AIPersonality::Balanced,
                };
                entity_builder.insert(personality);
            }

            if let Some(ai_mem) = nation_snap.ai_memory {
                entity_builder.insert(AIMemory {
                    recent_enemies: ai_mem
                        .recent_enemies
                        .iter()
                        .filter_map(|s| uuid::Uuid::parse_str(s).ok().map(NationId))
                        .collect(),
                    successful_wars: ai_mem.successful_wars,
                    failed_wars: ai_mem.failed_wars,
                    last_decision_tick: ai_mem.last_decision_tick,
                });
            }

            if let Some(econ) = nation_snap.economic_stress {
                entity_builder.insert(EconomicStress {
                    current_deficit: econ.current_deficit,
                    accumulated_deficit: econ.accumulated_deficit,
                    gdp: econ.gdp,
                });
            }

            if let Some(cas) = nation_snap.casualty_log {
                entity_builder.insert(CasualtyLog {
                    personnel_lost: cas.personnel_lost,
                    total_personnel: cas.total_personnel,
                });
            }

            if let Some(alliance_crisis) = nation_snap.alliance_crisis_log {
                entity_builder.insert(AllianceCrisisLog {
                    alliances_in_crisis: alliance_crisis.alliances_in_crisis,
                    total_alliances: alliance_crisis.total_alliances,
                });
            }

            if let Some(diplo_iso) = nation_snap.diplomatic_isolation_log {
                entity_builder.insert(DiplomaticIsolationLog {
                    hostile_relations: diplo_iso.hostile_relations,
                    friendly_relations: diplo_iso.friendly_relations,
                    total_relations: diplo_iso.total_relations,
                });
            }

            if nation_snap.is_player {
                entity_builder.insert(PlayerControlled);
            }

            let entity = entity_builder.id();
            nation_id_to_entity.insert(nation_snap.nation_id.clone(), entity);
        }

        // Restore provinces
        for prov_snap in snapshot.provinces {
            let province_id = uuid::Uuid::parse_str(&prov_snap.province_id)
                .ok()
                .map(ProvinceId)
                .unwrap_or_else(|| ProvinceId::new());

            let owner_id = uuid::Uuid::parse_str(&prov_snap.owner)
                .ok()
                .map(NationId)
                .unwrap_or_else(|| NationId::new());

            let dominant_resource = match prov_snap.dominant_resource.as_str() {
                "Food" => ResourceType::Food,
                "Iron" => ResourceType::Iron,
                "Oil" => ResourceType::Oil,
                "RareEarths" => ResourceType::RareEarths,
                "Water" => ResourceType::Water,
                "TradePorts" => ResourceType::TradePorts,
                _ => ResourceType::Food,
            };

            let mut entity_builder = world_state.world.spawn((
                Province {
                    id: province_id,
                    name: prov_snap.name.clone(),
                    position: glam::Vec2::new(prov_snap.position_x, prov_snap.position_y),
                    dominant_resource,
                },
                OwnedBy {
                    nation_id: owner_id,
                },
                Population {
                    total: prov_snap.population,
                    growth_rate: prov_snap.growth_rate,
                },
                Resources {
                    food: prov_snap.food,
                    iron: prov_snap.iron,
                    oil: prov_snap.oil,
                    rare_earths: prov_snap.rare_earths,
                    water: prov_snap.water,
                    trade_ports: prov_snap.trade_ports,
                },
            ));

            if prov_snap.is_capital {
                entity_builder.insert(Capital);
            }

            if prov_snap.is_occupied {
                if let (Some(occupier_str), Some(orig_owner_str), Some(occ_tick), Some(resist)) = (
                    prov_snap.occupied_by,
                    prov_snap.original_owner,
                    prov_snap.occupation_tick,
                    prov_snap.resistance,
                ) {
                    if let (Ok(occupier), Ok(orig_owner)) = (
                        uuid::Uuid::parse_str(&occupier_str).map(NationId),
                        uuid::Uuid::parse_str(&orig_owner_str).map(NationId),
                    ) {
                        entity_builder.insert(OccupiedProvince {
                            province_id,
                            occupier,
                            original_owner: orig_owner,
                            occupation_tick: occ_tick,
                            resistance: resist,
                        });
                    }
                }
            }

            let entity = entity_builder.id();
            province_id_to_entity.insert(prov_snap.province_id.clone(), entity);
        }

        // Restore armies
        for army_snap in snapshot.armies {
            let army_id = uuid::Uuid::parse_str(&army_snap.army_id)
                .ok()
                .map(ArmyId)
                .unwrap_or_else(|| ArmyId::new());

            let owner = uuid::Uuid::parse_str(&army_snap.owner)
                .ok()
                .map(NationId)
                .unwrap_or_else(|| NationId::new());

            let location = uuid::Uuid::parse_str(&army_snap.location)
                .ok()
                .map(ProvinceId)
                .unwrap_or_else(|| ProvinceId::new());

            let destination = army_snap
                .destination
                .and_then(|d| uuid::Uuid::parse_str(&d).ok().map(ProvinceId));

            world_state.world.spawn(Army {
                army_id,
                owner,
                location,
                infantry: army_snap.infantry,
                armor: army_snap.armor,
                artillery: army_snap.artillery,
                morale: army_snap.morale,
                organization: army_snap.organization,
                supply_state: army_snap.supply_state,
                entrenchment: army_snap.entrenchment,
                movement_points: army_snap.movement_points,
                destination,
            });
        }

        // Restore war declarations
        for war_snap in snapshot.war_declarations {
            let war_id = uuid::Uuid::parse_str(&war_snap.war_id)
                .ok()
                .map(WarId)
                .unwrap_or_else(|| WarId::new());

            let aggressor = uuid::Uuid::parse_str(&war_snap.aggressor)
                .ok()
                .map(NationId)
                .unwrap_or_else(|| NationId::new());

            let defender = uuid::Uuid::parse_str(&war_snap.defender)
                .ok()
                .map(NationId)
                .unwrap_or_else(|| NationId::new());

            // Parse casus belli
            let casus_belli = if war_snap.casus_belli.starts_with("TerritorialDispute:") {
                let province_str = war_snap.casus_belli.strip_prefix("TerritorialDispute:");
if let Some(s) = province_str {
                    if let Ok(uuid) = uuid::Uuid::parse_str(s) {
                        CasusBelli::TerritorialDispute(ProvinceId(uuid))
                    } else {
                        CasusBelli::ResourceConflict
                    }
                } else {
                    CasusBelli::ResourceConflict
                }
            } else if war_snap.casus_belli.starts_with("Liberation:") {
                let province_str = war_snap.casus_belli.strip_prefix("Liberation:");
                if let Some(s) = province_str {
                    if let Ok(uuid) = uuid::Uuid::parse_str(s) {
                        CasusBelli::Liberation(ProvinceId(uuid))
                    } else {
                        CasusBelli::ResourceConflict
                    }
                } else {
                    CasusBelli::ResourceConflict
                }
            } else if war_snap.casus_belli == "PreemptiveStrike" {
                CasusBelli::PreemptiveStrike
            } else {
                CasusBelli::ResourceConflict
            };

            // Parse war goal
            let war_goal = if war_snap.war_goal.starts_with("ConquerProvince:") {
                let province_str = war_snap.war_goal.strip_prefix("ConquerProvince:");
                if let Some(s) = province_str {
                    if let Ok(uuid) = uuid::Uuid::parse_str(s) {
                        WarGoal::ConquerProvince(ProvinceId(uuid))
                    } else {
                        WarGoal::Total
                    }
                } else {
                    WarGoal::Total
                }
            } else if war_snap.war_goal == "Humiliate" {
                WarGoal::Humiliate
            } else {
                WarGoal::Total
            };

            world_state.world.spawn(WarDeclaration {
                war_id,
                aggressor,
                defender,
                casus_belli,
                war_goal,
                declared_tick: war_snap.declared_tick,
            });
        }

        // Restore peace treaties
        for peace_snap in snapshot.peace_treaties {
            let war_id = uuid::Uuid::parse_str(&peace_snap.war_id)
                .ok()
                .map(WarId)
                .unwrap_or_else(|| WarId::new());

            let victor = peace_snap
                .victor
                .and_then(|v| uuid::Uuid::parse_str(&v).ok().map(NationId));

            let provinces_transferred: Vec<(ProvinceId, NationId)> = peace_snap
                .provinces_transferred
                .iter()
                .filter_map(|(p_str, n_str)| {
                    let p = uuid::Uuid::parse_str(p_str).ok().map(ProvinceId)?;
                    let n = uuid::Uuid::parse_str(n_str).ok().map(NationId)?;
                    Some((p, n))
                })
                .collect();

            world_state.world.spawn(PeaceTreaty {
                war_id,
                victor,
                terms: PeaceTerms {
                    provinces_transferred,
                    war_reparations: peace_snap.war_reparations,
                    cannot_redeclare_until: peace_snap.cannot_redeclare_until,
                },
                signed_tick: peace_snap.signed_tick,
            });
        }

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
                .filter_map(|m| uuid::Uuid::parse_str(m).ok().map(NationId))
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

    /// Perform auto-save if conditions are met
    pub fn auto_save(&mut self, config: &AutoSaveConfig) -> Result<()> {
        if !config.enabled {
            return Ok(());
        }

        // Check if we should save this tick
        if self.tick % config.interval_ticks != 0 {
            return Ok(());
        }

        // Create save directory if it doesn't exist
        fs::create_dir_all(&config.save_directory)
            .context("Failed to create auto-save directory")?;

        // Calculate slot number (rotate through slots)
        let slot_num = (self.tick / config.interval_ticks) % config.max_slots as u64;
        let filename = format!("autosave_slot_{}.json", slot_num);
        let filepath = config.save_directory.join(&filename);

        // Save
        self.save_to_file(&filepath)?;

        // Also save a "latest" file for easy access
        let latest_path = config.save_directory.join("autosave_latest.json");
        self.save_to_file(&latest_path)?;

        Ok(())
    }

    /// Get list of available save files in a directory
    pub fn list_saves<P: AsRef<Path>>(directory: P) -> Result<Vec<PathBuf>> {
        let mut saves = Vec::new();
        
        if !directory.as_ref().exists() {
            return Ok(saves);
        }

        for entry in fs::read_dir(directory).context("Failed to read save directory")? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                saves.push(path);
            }
        }

        // Sort by modification time (newest first)
        saves.sort_by(|a, b| {
            let time_a = fs::metadata(a).and_then(|m| m.modified()).ok();
            let time_b = fs::metadata(b).and_then(|m| m.modified()).ok();
            time_b.cmp(&time_a)
        });

        Ok(saves)
    }

    /// Quick save to default location
    pub fn quick_save(&mut self) -> Result<PathBuf> {
        let save_dir = PathBuf::from("saves/manual");
        fs::create_dir_all(&save_dir).context("Failed to create save directory")?;
        
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("quicksave_{}.json", timestamp);
        let filepath = save_dir.join(&filename);
        
        self.save_to_file(&filepath)?;
        Ok(filepath)
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
    use tempfile::{NamedTempFile, TempDir};
    use crate::core::tick::TickPipeline;

    #[test]
    fn test_save_and_load_basic() -> Result<()> {
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
    fn test_save_and_load_with_nations() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        // Create world with nations
        let mut world = WorldState::new(123);
        world.spawn_nation("Test Nation A".to_string(), [255, 0, 0], false);
        world.spawn_nation("Test Nation B".to_string(), [0, 255, 0], true);
        world.advance_tick();
        world.advance_tick();
        world.advance_tick();
        
        let hash_before = world.state_hash();
        world.save_to_file(path)?;

        // Load it back
        let mut loaded = WorldState::load_from_file(path)?;
        
        assert_eq!(loaded.tick, 3);
        assert_eq!(loaded.seed, 123);
        assert_eq!(loaded.nation_count(), 2);
        
        // Hash should match (determinism verification)
        let hash_after = loaded.state_hash();
        assert_eq!(hash_before, hash_after, "State hash mismatch after save/load");

        Ok(())
    }

    #[test]
    fn test_save_load_continue_determinism() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        // Create simple world and run to tick 100
        let mut world1 = WorldState::new(999);
        world1.spawn_nation("Nation Alpha".to_string(), [100, 100, 255], false);
        world1.spawn_nation("Nation Beta".to_string(), [255, 100, 100], false);
        
        // Run without pipeline to keep it simple
        for _ in 0..100 {
            world1.advance_tick();
        }
        
        // Save at tick 100
        world1.save_to_file(path)?;
        assert_eq!(world1.tick, 100);

        // Load and verify tick preserved
        let mut world2 = WorldState::load_from_file(path)?;
        assert_eq!(world2.tick, 100, "Tick should be preserved after load");
        assert_eq!(world2.seed, 999, "Seed should be preserved after load");
        assert_eq!(world2.nation_count(), 2, "Nation count should be preserved");

        // Continue both worlds and verify they maintain same tick
        for _ in 0..100 {
            world1.advance_tick();
            world2.advance_tick();
        }
        
        assert_eq!(world1.tick, 200);
        assert_eq!(world2.tick, 200, "Both worlds should reach same tick after continuation");

        Ok(())
    }

    #[test]
    fn test_auto_save_functionality() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = AutoSaveConfig {
            enabled: true,
            interval_ticks: 10,
            max_slots: 3,
            save_directory: temp_dir.path().to_path_buf(),
        };

        let mut world = WorldState::new(555);
        world.spawn_nation("Test Nation".to_string(), [128, 128, 128], false);

        // Run through several auto-save intervals
        for _tick in 0..35 {
            world.auto_save(&config)?;
            world.advance_tick();
        }

        // Should have created 3 regular slots + latest
        let saves = WorldState::list_saves(temp_dir.path())?;
        assert!(saves.len() >= 3, "Should have at least 3 auto-save files");

        // Verify latest file exists
        let latest_path = temp_dir.path().join("autosave_latest.json");
        assert!(latest_path.exists(), "Latest auto-save should exist");

        // Load the latest save
        let loaded = WorldState::load_from_file(&latest_path)?;
        assert!(loaded.tick % 10 == 0, "Auto-save should occur at interval boundaries");

        Ok(())
    }

    #[test]
    fn test_quick_save() -> Result<()> {
        let mut world = WorldState::new(777);
        world.spawn_nation("Quick Save Test".to_string(), [200, 200, 200], false);
        world.advance_tick();

        let save_path = world.quick_save()?;
        assert!(save_path.exists());

        let loaded = WorldState::load_from_file(&save_path)?;
        assert_eq!(loaded.tick, 1);
        assert_eq!(loaded.seed, 777);

        // Clean up
        if save_path.exists() {
            fs::remove_file(save_path).ok();
        }

        Ok(())
    }

    #[test]
    fn test_list_saves() -> Result<()> {
        let temp_dir = TempDir::new()?;
        
        // Create several save files
        for i in 0..5 {
            let mut world = WorldState::new(i);
            world.advance_tick();
            let path = temp_dir.path().join(format!("save_{}.json", i));
            world.save_to_file(&path)?;
            std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
        }

        let saves = WorldState::list_saves(temp_dir.path())?;
        assert_eq!(saves.len(), 5);

        Ok(())
    }

    #[test]
    fn test_state_hash_determinism() {
        let mut world1 = WorldState::new(42);
        let mut world2 = WorldState::new(42);

        assert_eq!(world1.state_hash(), world2.state_hash());
    }
}
