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

/// Nation economic stress (V0.5)
/// Tracks deficit and economic pressure for legitimacy calculations
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct EconomicStress {
    /// Current tick's total deficit (costs - income)
    pub current_deficit: f64,
    /// Running total of deficits (resets annually)
    pub accumulated_deficit: f64,
    /// GDP for inflation calculation
    pub gdp: f64,
}

impl Default for EconomicStress {
    fn default() -> Self {
        Self {
            current_deficit: 0.0,
            accumulated_deficit: 0.0,
            gdp: 1_000_000.0,
        }
    }
}

impl EconomicStress {
    /// Calculate deficit-to-GDP ratio for legitimacy impact
    pub fn deficit_inflation_rate(&self) -> f64 {
        if self.gdp <= 0.0 {
            0.0
        } else {
            (self.current_deficit / self.gdp).max(0.0)
        }
    }
}

/// Nation casualty tracking (V0.5)
/// Tracks personnel losses across all armies for legitimacy calculations
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct CasualtyLog {
    /// Total personnel killed this tick
    pub personnel_lost: u64,
    /// Total personnel at start of tick (for casualty ratio)
    pub total_personnel: u64,
}

impl Default for CasualtyLog {
    fn default() -> Self {
        Self {
            personnel_lost: 0,
            total_personnel: 1000, // Placeholder
        }
    }
}

impl CasualtyLog {
    /// Calculate casualty ratio (losses / total) for legitimacy calculations
    pub fn casualty_ratio(&self) -> f64 {
        if self.total_personnel == 0 {
            0.0
        } else {
            (self.personnel_lost as f64 / self.total_personnel as f64).min(1.0)
        }
    }

    /// Reset this tick's losses (call at end of combat phase)
    pub fn reset_losses(&mut self) {
        self.personnel_lost = 0;
    }
}

/// Nation alliance crisis tracking (V0.5)
/// Tracks alliance crises (cohesion < 25) for legitimacy calculations
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct AllianceCrisisLog {
    /// Number of alliances in crisis state
    pub alliances_in_crisis: u32,
    /// Total alliance count (for burden calculation)
    pub total_alliances: u32,
}

impl Default for AllianceCrisisLog {
    fn default() -> Self {
        Self {
            alliances_in_crisis: 0,
            total_alliances: 0,
        }
    }
}

impl AllianceCrisisLog {
    /// Check if this nation is in alliance crisis
    pub fn has_crisis(&self) -> bool {
        self.alliances_in_crisis > 0
    }

    /// Calculate alliance burden including crisis penalty
    /// Base burden: -0.1 per alliance
    /// Crisis penalty: -0.2 per alliance in crisis
    pub fn alliance_burden(&self) -> f64 {
        let base_burden = self.total_alliances as f64 * -0.1;
        let crisis_penalty = self.alliances_in_crisis as f64 * -0.2;
        base_burden + crisis_penalty
    }
}

/// Nation diplomatic isolation tracking (V0.5)
/// Tracks diplomatic relations quality for legitimacy calculations
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct DiplomaticIsolationLog {
    /// Number of hostile relations (reputation < -25)
    pub hostile_relations: u32,
    /// Number of friendly relations (reputation > 25)
    pub friendly_relations: u32,
    /// Total number of diplomatic relations
    pub total_relations: u32,
}

impl Default for DiplomaticIsolationLog {
    fn default() -> Self {
        Self {
            hostile_relations: 0,
            friendly_relations: 0,
            total_relations: 0,
        }
    }
}

impl DiplomaticIsolationLog {
    /// Calculate isolation penalty for legitimacy
    /// Penalty increases with hostile relations and lack of friends
    pub fn isolation_penalty(&self) -> f64 {
        if self.total_relations == 0 {
            return 0.0; // No penalty if no relations tracked yet
        }
        
        let hostile_ratio = self.hostile_relations as f64 / self.total_relations as f64;
        let friendly_ratio = self.friendly_relations as f64 / self.total_relations as f64;
        
        // Hostile relations: -0.15 per 10% of relations that are hostile
        let hostile_penalty = hostile_ratio * -1.5;
        
        // Lack of friends: -0.1 if less than 25% relations are friendly
        let isolation_penalty = if friendly_ratio < 0.25 {
            -0.1
        } else {
            0.0
        };
        
        hostile_penalty + isolation_penalty
    }
    
    /// Check if nation is diplomatically isolated
    pub fn is_isolated(&self) -> bool {
        self.total_relations > 0 && self.friendly_relations == 0
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
// V0.4 ALLIANCES & DIPLOMACY
// ============================================================================

/// Alliance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AllianceId(pub Uuid);

impl AllianceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AllianceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Alliance doctrine types (mirrors alliance_dataset::AllianceDoctrine for serialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllianceDoctrine {
    DefensiveAgreement,
    OffensivePact,
    EconomicBloc,
    ResearchConsortium,
    BalanceOfPower,
}

impl AllianceDoctrine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DefensiveAgreement => "DefensiveAgreement",
            Self::OffensivePact => "OffensivePact",
            Self::EconomicBloc => "EconomicBloc",
            Self::ResearchConsortium => "ResearchConsortium",
            Self::BalanceOfPower => "BalanceOfPower",
        }
    }
}

impl Default for AllianceDoctrine {
    fn default() -> Self {
        Self::DefensiveAgreement
    }
}

/// Represents an alliance between multiple nations (V0.4 feature)
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Alliance {
    pub alliance_id: AllianceId,
    pub alliance_name: String,           // e.g., "Sovereign Shield Pact"
    pub founding_nation: NationId,
    pub members: Vec<NationId>,
    pub cohesion: f64,                   // 0-100, decay each tick
    pub doctrine: AllianceDoctrine,
    pub founded_tick: Tick,
    pub threat_reduction: f64,           // 0.15-0.50, affects war calculations
    pub cohesion_decay_rate: f64,        // 0.5-2.5 per tick
}

impl Default for Alliance {
    fn default() -> Self {
        Self {
            alliance_id: AllianceId::new(),
            alliance_name: "Unnamed Alliance".to_string(),
            founding_nation: NationId::default(),
            members: Vec::new(),
            cohesion: 100.0,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            founded_tick: 0,
            threat_reduction: 0.25,
            cohesion_decay_rate: 1.0,
        }
    }
}

impl Alliance {
    pub fn add_member(&mut self, nation_id: NationId) {
        if !self.members.contains(&nation_id) {
            self.members.push(nation_id);
        }
    }

    pub fn remove_member(&mut self, nation_id: NationId) {
        self.members.retain(|&id| id != nation_id);
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn is_dissolved(&self) -> bool {
        self.cohesion < 15.0 || self.member_count() < 2
    }

    pub fn decay_cohesion(&mut self) {
        self.cohesion = (self.cohesion - self.cohesion_decay_rate).max(0.0);
    }

    pub fn boost_cohesion(&mut self, amount: f64) {
        self.cohesion = (self.cohesion + amount).min(100.0);
    }
}

/// Diplomatic relation between two nations (V0.4)
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct DiplomaticRelation {
    pub nation_a: NationId,
    pub nation_b: NationId,
    pub reputation: f64,                 // -100 to +100
    pub trade_dependency: f64,           // 0-1, how dependent on each other
    pub threat_alignment: f64,           // -1 to +1, shared enemy = +1, conflict = -1
    pub last_war: Option<Tick>,
    pub allied_since: Option<Tick>,
    pub last_updated: Tick,
}

impl Default for DiplomaticRelation {
    fn default() -> Self {
        Self {
            nation_a: NationId::default(),
            nation_b: NationId::default(),
            reputation: 0.0,
            trade_dependency: 0.0,
            threat_alignment: 0.0,
            last_war: None,
            allied_since: None,
            last_updated: 0,
        }
    }
}

impl DiplomaticRelation {
    pub fn alliance_score(&self) -> f64 {
        // Composite score for alliance decision-making
        self.trade_dependency * 0.3 + self.threat_alignment.abs() * 0.4 + (self.reputation + 100.0) / 200.0 * 0.3
    }

    pub fn is_friendly(&self) -> bool {
        self.reputation > 25.0
    }

    pub fn is_hostile(&self) -> bool {
        self.reputation < -25.0
    }

    pub fn modify_reputation(&mut self, delta: f64) {
        self.reputation = (self.reputation + delta).clamp(-100.0, 100.0);
    }
}

// ============================================================================
// V0.4 NUCLEAR WEAPONS SYSTEM
// ============================================================================

/// Nuclear capability readiness (0-100)
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct NuclearCapability {
    pub readiness: f64,  // 0-100, percentage of readiness
    pub development_rate: f64,  // how fast ability increases per tick if developing
}

impl Default for NuclearCapability {
    fn default() -> Self {
        Self {
            readiness: 0.0,
            development_rate: 0.5,  // default: +0.5% per tick if not violating treaty
        }
    }
}

impl NuclearCapability {
    pub fn new(initial: f64) -> Self {
        Self {
            readiness: initial.clamp(0.0, 100.0),
            development_rate: 0.5,
        }
    }

    pub fn can_use(&self) -> bool {
        self.readiness >= 30.0  // minimum 30% to deploy
    }

    pub fn develop(&mut self, rate: f64) {
        self.readiness = (self.readiness + rate).min(100.0);
    }

    pub fn reset(&mut self) {
        self.readiness = 0.0;
    }
}

/// Nuclear posture of a nation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum NuclearPosture {
    Dormant,      // Not developing, treaty compliant
    Developing,   // Actively researching
    Deployed,     // Ready to use
    Deterrent,    // Being used as deterrence against allies/rivals
}

impl Default for NuclearPosture {
    fn default() -> Self {
        Self::Dormant
    }
}

/// Nuclear treaty identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct NuclearTreatyId(pub Uuid);

impl NuclearTreatyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NuclearTreatyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Membership in a nuclear non-proliferation treaty
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct NuclearTreatyMembership {
    pub treaty_id: NuclearTreatyId,
    pub joined_tick: Tick,
}

impl NuclearTreatyMembership {
    pub fn new(treaty_id: NuclearTreatyId, tick: Tick) -> Self {
        Self {
            treaty_id,
            joined_tick: tick,
        }
    }
}

/// Record of a treaty violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuclearViolation {
    pub violation_tick: Tick,
    pub violation_type: NuclearViolationType,
}

/// Types of treaty violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NuclearViolationType {
    DevelopmentWhileInTreaty,  // Increased readiness while in treaty
    UseWhileInTreaty,          // Actually deployed nuclear weapon while in treaty
}

/// Tracks all nuclear treaty violations for a nation
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct NuclearViolationRecord {
    pub violations: Vec<NuclearViolation>,
}

impl Default for NuclearViolationRecord {
    fn default() -> Self {
        Self {
            violations: Vec::new(),
        }
    }
}

impl NuclearViolationRecord {
    pub fn is_violator(&self) -> bool {
        !self.violations.is_empty()
    }

    pub fn add_violation(&mut self, violation_type: NuclearViolationType, tick: Tick) {
        self.violations.push(NuclearViolation {
            violation_tick: tick,
            violation_type,
        });
    }

    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Record of a nuclear weapon use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuclearUseIncident {
    pub use_tick: Tick,
    pub attacker: NationId,
    pub target: NationId,
    pub target_provinces: Vec<ProvinceId>,  // which provinces were hit
    pub war_context: Option<WarId>,
}

/// Tracks all nuclear weapon uses
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct NuclearUseRecord {
    pub uses: Vec<NuclearUseIncident>,
}

impl Default for NuclearUseRecord {
    fn default() -> Self {
        Self {
            uses: Vec::new(),
        }
    }
}

impl NuclearUseRecord {
    pub fn add_use(
        &mut self,
        attacker: NationId,
        target: NationId,
        target_provinces: Vec<ProvinceId>,
        war_context: Option<WarId>,
        tick: Tick,
    ) {
        self.uses.push(NuclearUseIncident {
            use_tick: tick,
            attacker,
            target,
            target_provinces,
            war_context,
        });
    }

    pub fn total_uses(&self) -> usize {
        self.uses.len()
    }

    pub fn uses_against(&self, target: NationId) -> usize {
        self.uses.iter().filter(|u| u.target == target).count()
    }
}

/// Tracks initial war state for desperation calculations (bot AI)
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct WarStartSnapshot {
    pub war_id: WarId,
    pub start_tick: Tick,
    pub territory_at_start: usize,  // number of provinces owned when war started
    pub military_at_start: f64,     // military capacity at war start
}

impl WarStartSnapshot {
    pub fn new(war_id: WarId, start_tick: Tick, territory: usize, military: f64) -> Self {
        Self {
            war_id,
            start_tick,
            territory_at_start: territory,
            military_at_start: military,
        }
    }
}

// ============================================================================
// NOTIFICATIONS AND NEWS SYSTEM
// ============================================================================

/// Unique identifier for notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct NotificationId(pub Uuid);

impl NotificationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NotificationId {
    fn default() -> Self {
        Self::new()
    }
}

/// Severity level of a notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationSeverity {
    /// Informational updates
    Info,
    /// Important events requiring attention
    Warning,
    /// Critical events requiring immediate action
    Critical,
    /// Game-changing events
    Major,
}

/// Types of notifications in the game
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationType {
    // Warfare events
    WarDeclared { attacker: NationId, defender: NationId },
    WarEnded { victor: Option<NationId>, defeated: NationId },
    MajorBattle { attacker: NationId, defender: NationId, casualties: u32 },
    TerritoryLost { nation: NationId, provinces_lost: u32 },
    TerritoryGained { nation: NationId, provinces_gained: u32 },
    
    // Diplomacy events
    AllianceFormed { nations: Vec<NationId> },
    AllianceBroken { former_allies: Vec<NationId> },
    TreatyViolation { violator: NationId, treaty_type: String },
    DiplomaticIsolation { nation: NationId },
    
    // Nuclear events
    NuclearWeaponUsed { attacker: NationId, target: NationId, provinces: Vec<ProvinceId> },
    NuclearTreatyViolation { violator: NationId },
    NuclearCapabilityAchieved { nation: NationId },
    
    // Economic events
    EconomicCrisis { nation: NationId, gdp_loss: f64 },
    EconomicBoom { nation: NationId, gdp_gain: f64 },
    SanctionsImposed { target: NationId, imposer: NationId },
    TradeAgreement { nations: Vec<NationId> },
    
    // Espionage events
    EspionageRevealed { perpetrator: NationId, target: NationId, operation: String },
    AgentCaught { spy_nation: NationId, caught_by: NationId },
    IntelligenceGained { nation: NationId, target: NationId },
    
    // Vassalage events
    VassalizationOccurred { overlord: NationId, vassal: NationId },
    VassalRebellion { vassal: NationId, overlord: NationId },
    VassalLiberated { freed_nation: NationId, former_overlord: NationId },
    
    // Political events
    LegitimacyCrisis { nation: NationId, legitimacy: f64 },
    RegimeChange { nation: NationId },
    CivilUnrest { nation: NationId },
    
    // Provincial events
    ProvinceConquered { province: ProvinceId, new_owner: NationId },
    ProvinceLost { province: ProvinceId, former_owner: NationId },
    
    // General events
    GameEvent { message: String },
}

/// A notification/news item in the game
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub tick: Tick,
    pub severity: NotificationSeverity,
    pub related_nations: Vec<NationId>,
    pub read: bool,
}

impl Notification {
    pub fn new(
        notification_type: NotificationType,
        title: String,
        message: String,
        tick: Tick,
        severity: NotificationSeverity,
        related_nations: Vec<NationId>,
    ) -> Self {
        Self {
            id: NotificationId::new(),
            notification_type,
            title,
            message,
            tick,
            severity,
            related_nations,
            read: false,
        }
    }

    /// Mark notification as read
    pub fn mark_read(&mut self) {
        self.read = true;
    }

    /// Check if notification involves a specific nation
    pub fn involves_nation(&self, nation_id: NationId) -> bool {
        self.related_nations.contains(&nation_id)
    }
}

/// Component storing all notifications for a nation
#[derive(Debug, Clone, Component, Serialize, Deserialize, Default)]
pub struct NotificationLog {
    pub notifications: Vec<Notification>,
}

impl NotificationLog {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    /// Add a notification to the log
    pub fn add(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    /// Get unread notifications
    pub fn unread(&self) -> Vec<&Notification> {
        self.notifications.iter().filter(|n| !n.read).collect()
    }

    /// Get notifications by severity
    pub fn by_severity(&self, severity: NotificationSeverity) -> Vec<&Notification> {
        self.notifications.iter().filter(|n| n.severity == severity).collect()
    }

    /// Get recent notifications (last N ticks)
    pub fn recent(&self, current_tick: Tick, ticks_back: u64) -> Vec<&Notification> {
        let cutoff = current_tick.saturating_sub(ticks_back);
        self.notifications.iter().filter(|n| n.tick >= cutoff).collect()
    }

    /// Mark all notifications as read
    pub fn mark_all_read(&mut self) {
        for notification in &mut self.notifications {
            notification.read = true;
        }
    }

    /// Get notification count by severity
    pub fn count_by_severity(&self, severity: NotificationSeverity) -> usize {
        self.notifications.iter().filter(|n| n.severity == severity).count()
    }

    /// Prune old notifications (keep last N)
    pub fn prune(&mut self, keep_last: usize) {
        if self.notifications.len() > keep_last {
            let start = self.notifications.len() - keep_last;
            self.notifications = self.notifications.split_off(start);
        }
    }
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
