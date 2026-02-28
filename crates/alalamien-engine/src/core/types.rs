//! Core type definitions for Alalamien War
//!
//! This module defines the fundamental entities and data structures
//! used throughout the simulation.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use glam::Vec2;

/// Unique identifier for entities in the simulation
pub type EntityId = Uuid;

/// Game tick counter
pub type Tick = u64;

// ============================================================================
// RESOURCE TYPES
// ============================================================================

/// Primary resource types in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Food,
    Iron,
    Oil,
    RareEarths,
    Water,
    TradePorts,
}

/// Resource storage container
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Resources {
    pub food: f64,
    pub iron: f64,
    pub oil: f64,
    pub rare_earths: f64,
    pub water: f64,
    pub trade_ports: u32,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            food: 100.0,
            iron: 50.0,
            oil: 50.0,
            rare_earths: 10.0,
            water: 100.0,
            trade_ports: 1,
        }
    }
}

impl Resources {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, resource_type: ResourceType) -> f64 {
        match resource_type {
            ResourceType::Food => self.food,
            ResourceType::Iron => self.iron,
            ResourceType::Oil => self.oil,
            ResourceType::RareEarths => self.rare_earths,
            ResourceType::Water => self.water,
            ResourceType::TradePorts => self.trade_ports as f64,
        }
    }

    pub fn set(&mut self, resource_type: ResourceType, value: f64) {
        match resource_type {
            ResourceType::Food => self.food = value,
            ResourceType::Iron => self.iron = value,
            ResourceType::Oil => self.oil = value,
            ResourceType::RareEarths => self.rare_earths = value,
            ResourceType::Water => self.water = value,
            ResourceType::TradePorts => self.trade_ports = value.max(0.0) as u32,
        }
    }

    pub fn add(&mut self, resource_type: ResourceType, amount: f64) {
        let current = self.get(resource_type);
        self.set(resource_type, current + amount);
    }
}

// ============================================================================
// PROVINCE
// ============================================================================

/// Province identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct ProvinceId(pub Uuid);

impl ProvinceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProvinceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Province entity component
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Province {
    pub id: ProvinceId,
    pub name: String,
    pub position: Vec2,
    pub dominant_resource: ResourceType,
}

/// Province population component
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Population {
    pub total: u64,
    pub growth_rate: f64,
}

impl Default for Population {
    fn default() -> Self {
        Self {
            total: 1_000_000,
            growth_rate: 0.01,
        }
    }
}

/// Province infrastructure level
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Infrastructure {
    pub level: u32,
    pub max_level: u32,
}

impl Default for Infrastructure {
    fn default() -> Self {
        Self {
            level: 1,
            max_level: 10,
        }
    }
}

/// Province ownership - links a province to its owning nation
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct OwnedBy {
    pub nation_id: NationId,
}

// ============================================================================
// NATION
// ============================================================================

/// Nation identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct NationId(pub Uuid);

impl NationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Nation entity component
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Nation {
    pub id: NationId,
    pub name: String,
    pub color: [u8; 3], // RGB color for map display
}

/// Nation legitimacy component
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Legitimacy {
    pub value: f64, // 0.0 - 100.0
}

impl Default for Legitimacy {
    fn default() -> Self {
        Self { value: 50.0 }
    }
}

impl Legitimacy {
    pub fn new(value: f64) -> Self {
        Self {
            value: value.clamp(0.0, 100.0),
        }
    }

    pub fn modify(&mut self, delta: f64) {
        self.value = (self.value + delta).clamp(0.0, 100.0);
    }

    pub fn is_stable(&self) -> bool {
        self.value >= 35.0
    }

    pub fn is_critical(&self) -> bool {
        self.value < 20.0
    }
}

/// Nation GDP component
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct GDP {
    pub value: f64,
    pub growth_rate: f64,
}

impl Default for GDP {
    fn default() -> Self {
        Self {
            value: 1_000_000.0,
            growth_rate: 0.02,
        }
    }
}

/// Nation military capacity
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct MilitaryCapacity {
    pub value: f64, // Gained by consuming Iron
}

/// Nation logistics capacity
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct Logistics {
    pub value: f64, // Gained by consuming Oil
}

// ============================================================================
// MILITARY & LOGISTICS DATA MODELS
// ============================================================================

/// Identifies a province as the capital of its owning nation
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Capital;

// ============================================================================
// V0.3 COMBAT & WARFARE TYPES
// ============================================================================

/// Army identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArmyId(pub Uuid);

impl ArmyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ArmyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an army with combat capabilities (V0.3 enhanced)
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Army {
    pub army_id: ArmyId,
    pub owner: NationId,
    pub location: ProvinceId,
    
    // Unit composition
    pub infantry: u64,
    pub armor: u64,
    pub artillery: u64,
    
    // Combat state
    pub morale: f64,           // 0-100
    pub organization: f64,     // 0-100 (combat readiness)
    pub supply_state: f64,     // 0-1 (from LogisticsPhase)
    pub entrenchment: f64,     // 0-1 (defensive bonus)
    
    // Movement (future use)
    pub movement_points: f64,
    pub destination: Option<ProvinceId>,
}

impl Default for Army {
    fn default() -> Self {
        Self {
            army_id: ArmyId::new(),
            owner: NationId::default(),
            location: ProvinceId::default(),
            infantry: 10_000,
            armor: 1_000,
            artillery: 500,
            morale: 80.0,
            organization: 100.0,
            supply_state: 1.0,
            entrenchment: 0.0,
            movement_points: 100.0,
            destination: None,
        }
    }
}

impl Army {
    /// Calculate total combat strength
    pub fn combat_strength(&self) -> f64 {
        const INFANTRY_POWER: f64 = 1.0;
        const ARMOR_POWER: f64 = 3.0;
        const ARTILLERY_POWER: f64 = 2.0;
        const ENTRENCHMENT_BONUS: f64 = 0.5;
        
        let base_strength = 
            (self.infantry as f64 * INFANTRY_POWER) +
            (self.armor as f64 * ARMOR_POWER) +
            (self.artillery as f64 * ARTILLERY_POWER);
        
        base_strength 
            * (self.morale / 100.0)
            * (self.organization / 100.0)
            * self.supply_state
            * (1.0 + self.entrenchment * ENTRENCHMENT_BONUS)
    }
    
    /// Check if army can attack
    pub fn can_attack(&self) -> bool {
        self.organization > 30.0 && self.supply_state > 0.2
    }
}

/// Battle identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BattleId(pub Uuid);

impl BattleId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for BattleId {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an ongoing battle in a province
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct ProvinceBattle {
    pub battle_id: BattleId,
    pub province_id: ProvinceId,
    pub tick_started: Tick,
    pub duration: u32,
    
    pub attackers: Vec<ArmyId>,
    pub defenders: Vec<ArmyId>,
    
    pub attacker_casualties: BattleCasualties,
    pub defender_casualties: BattleCasualties,
    
    pub terrain_modifier: f64,
    pub weather_modifier: f64,
}

/// Battle casualties breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BattleCasualties {
    pub infantry_lost: u64,
    pub armor_lost: u64,
    pub artillery_lost: u64,
}

/// Battle outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleResult {
    Ongoing,
    AttackerVictory,
    DefenderVictory,
    Stalemate,
}

/// Terrain type affecting combat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Plains,
    Forest,
    Mountains,
    Urban,
    Desert,
    Swamp,
    Coastal,
}

impl TerrainType {
    pub fn defender_modifier(&self) -> f64 {
        match self {
            Self::Plains => 1.0,
            Self::Forest => 1.2,
            Self::Mountains => 1.5,
            Self::Urban => 1.3,
            Self::Desert => 1.0,
            Self::Swamp => 1.1,
            Self::Coastal => 1.0,
        }
    }
    
    pub fn attacker_modifier(&self) -> f64 {
        match self {
            Self::Mountains => 0.7,
            Self::Swamp => 0.8,
            Self::Desert => 0.9,
            _ => 1.0,
        }
    }
}

impl Default for TerrainType {
    fn default() -> Self {
        Self::Plains
    }
}

/// Province terrain component
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Terrain {
    pub terrain_type: TerrainType,
    pub defensibility: f64,  // 0-1
}

impl Default for Terrain {
    fn default() -> Self {
        Self {
            terrain_type: TerrainType::Plains,
            defensibility: 0.5,
        }
    }
}

/// Military doctrine affecting combat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum MilitaryDoctrine {
    MassMobilization,
    ProfessionalForce,
    Mechanized,
    Asymmetric,
    DefensiveDepth,
    Blitzkrieg,
    NavalSupremacy,
}

impl MilitaryDoctrine {
    pub fn combat_modifier(&self) -> f64 {
        match self {
            Self::MassMobilization => 0.9,
            Self::ProfessionalForce => 1.2,
            Self::Mechanized => 1.4,
            Self::Asymmetric => 0.8,
            Self::DefensiveDepth => 0.85,
            Self::Blitzkrieg => 1.3,
            Self::NavalSupremacy => 1.0,
        }
    }
    
    pub fn defensive_modifier(&self) -> f64 {
        match self {
            Self::Asymmetric => 1.2,
            Self::DefensiveDepth => 1.3,
            _ => 1.0,
        }
    }
}

impl Default for MilitaryDoctrine {
    fn default() -> Self {
        Self::ProfessionalForce
    }
}

/// War identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WarId(pub Uuid);

impl WarId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for WarId {
    fn default() -> Self {
        Self::new()
    }
}

/// Casus belli for war declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CasusBelli {
    TerritorialDispute(ProvinceId),
    ResourceConflict,
    PreemptiveStrike,
    Liberation(ProvinceId),
}

/// War goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarGoal {
    ConquerProvince(ProvinceId),
    Humiliate,
    Total,
}

/// War declaration
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct WarDeclaration {
    pub war_id: WarId,
    pub aggressor: NationId,
    pub defender: NationId,
    pub casus_belli: CasusBelli,
    pub war_goal: WarGoal,
    pub declared_tick: Tick,
}

/// Occupied province marker
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct OccupiedProvince {
    pub province_id: ProvinceId,
    pub occupier: NationId,
    pub original_owner: NationId,
    pub occupation_tick: Tick,
    pub resistance: f64,  // 0-1
}

/// War exhaustion component
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct WarExhaustion {
    pub value: f64,  // 0-100
}

/// Peace treaty
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct PeaceTreaty {
    pub war_id: WarId,
    pub victor: Option<NationId>,
    pub terms: PeaceTerms,
    pub signed_tick: Tick,
}

/// Peace treaty terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceTerms {
    pub provinces_transferred: Vec<(ProvinceId, NationId)>,
    pub war_reparations: f64,
    pub cannot_redeclare_until: Tick,
}

/// Tracks external relations (V0.2 simple version, V0.3 enhanced)
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct WarState {
    pub at_war_with: Vec<NationId>,
}

// ============================================================================
// V0.3 AI TYPES
// ============================================================================

/// AI personality affecting decision-making
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum AIPersonality {
    Defensive,
    Balanced,
    Aggressive,
}

impl Default for AIPersonality {
    fn default() -> Self {
        Self::Balanced
    }
}

/// Persistent AI memory used by V0.35 decision logic
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct AIMemory {
    pub recent_enemies: Vec<NationId>,
    pub successful_wars: u32,
    pub failed_wars: u32,
    pub last_decision_tick: Tick,
}

impl Default for AIMemory {
    fn default() -> Self {
        Self {
            recent_enemies: Vec::new(),
            successful_wars: 0,
            failed_wars: 0,
            last_decision_tick: 0,
        }
    }
}

/// AI threat knowledge entry used by intelligence system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatRecord {
    pub nation_id: NationId,
    pub score: f64,
    pub last_updated: Tick,
}

/// Intelligence profile used for fog-of-war-ready strategic awareness
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct IntelligenceProfile {
    pub intel_quality: f64,          // 0-1 confidence quality
    pub known_threats: Vec<ThreatRecord>,
    pub fog_strength: f64,           // 0-1, higher = more uncertainty
}

impl Default for IntelligenceProfile {
    fn default() -> Self {
        Self {
            intel_quality: 0.5,
            known_threats: Vec::new(),
            fog_strength: 0.5,
        }
    }
}

// ============================================================================
// V0.3 VASSALAGE & DIPLOMACY
// ============================================================================

/// Vassal relationship
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct VassalRelation {
    pub overlord: NationId,
    pub vassal: NationId,
    pub tribute_percentage: f64,
    pub established_tick: Tick,
    pub loyalty: f64,  // 0-100
}

/// Autonomy level for vassals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomyLevel {
    FullVassal,
    Protectorate,
    Tributary,
}

// ============================================================================
// TRADE & ECONOMY (V0.2)
// ============================================================================

/// Trade route identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeRouteId(pub Uuid);

impl TradeRouteId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TradeRouteId {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a trade route between two provinces
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct TradeRoute {
    pub id: TradeRouteId,
    pub from: ProvinceId,
    pub to: ProvinceId,
    pub resource_type: ResourceType,
    pub flow_rate: f64,
    pub active: bool,
}

/// Tracks resource deficits for a province
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct ResourceDeficit {
    pub food_deficit: f64,
    pub iron_deficit: f64,
    pub oil_deficit: f64,
}

// ============================================================================
// TAGS AND MARKERS
// ============================================================================

/// Marker for player-controlled nation
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct PlayerControlled;

/// Marker for AI-controlled nation
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct AIControlled;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resources_operations() {
        let mut resources = Resources::default();
        
        // Test get
        assert_eq!(resources.get(ResourceType::Food), 100.0);
        
        // Test set
        resources.set(ResourceType::Food, 150.0);
        assert_eq!(resources.get(ResourceType::Food), 150.0);
        
        // Test add
        resources.add(ResourceType::Food, 50.0);
        assert_eq!(resources.get(ResourceType::Food), 200.0);
    }

    #[test]
    fn test_legitimacy_bounds() {
        let mut legitimacy = Legitimacy::new(50.0);
        
        // Test upper bound
        legitimacy.modify(100.0);
        assert_eq!(legitimacy.value, 100.0);
        
        // Test lower bound
        legitimacy.modify(-150.0);
        assert_eq!(legitimacy.value, 0.0);
    }

    #[test]
    fn test_legitimacy_stability() {
        assert!(Legitimacy::new(50.0).is_stable());
        assert!(!Legitimacy::new(30.0).is_stable());
        assert!(Legitimacy::new(15.0).is_critical());
    }
}
