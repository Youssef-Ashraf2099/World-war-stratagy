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

/// Represents an army stationed in a province
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Army {
    pub owner: NationId,
    pub personnel: u64,
    pub forced_march: bool,
}

impl Default for Army {
    fn default() -> Self {
        Self {
            owner: NationId::default(),
            personnel: 10_000,
            forced_march: false,
        }
    }
}

/// Marks a province as occupied by a foreign power
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Occupation {
    pub occupied_by: NationId,
}

/// Tracks external relations (simplified V0.3 WarState)
#[derive(Debug, Clone, Component, Default, Serialize, Deserialize)]
pub struct WarState {
    pub at_war_with: Vec<NationId>,
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
