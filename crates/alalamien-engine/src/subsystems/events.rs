//! Dynamic World Events System (V0.6)
//!
//! Generates random events that affect nations, adding unpredictability and narrative depth:
//! - Economic events (trade deals, market crashes, resource discoveries)
//! - Military events (coups, military reforms, terrorism)
//! - Diplomatic events (peace movements, border incidents)
//! - Natural disasters (earthquakes, floods, droughts)
//! - Social events (cultural movements, elections, corruption scandals)
//!
//! ## Event Mechanics
//! - Probability-based triggering using deterministic RNG
//! - Nation-specific and global events
//! - Immediate and duration-based effects
//! - Stacking event modifiers

use bevy_ecs::prelude::*;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::core::tick::TickPhase;
use crate::core::types::{
    Nation, NationId, Legitimacy, Resources, GDP,
    OwnedBy, Population,
};
use crate::core::deterministic::DeterministicRng;

// ============================================================================
// EVENT TYPES
// ============================================================================

/// Unique identifier for events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct EventId(pub uuid::Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Categories of world events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Economic,
    Military,
    Diplomatic,
    Natural,
    Social,
    Political,
}

/// Specific event types with associated effects
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    // Economic Events (5)
    TradeBonus { gdp_modifier: f64 },           // +10% GDP for 20 ticks
    MarketCrash { gdp_penalty: f64 },           // -20% GDP for 15 ticks
    ResourceDiscovery { resource_bonus: f64 },   // +50% resources for 30 ticks
    EconomicReform { growth_rate: f64 },        // +2% GDP growth permanently
    CurrencyCrisis { inflation: f64 },          // -15% GDP, -10 legitimacy
    
    // Military Events (4)
    MilitaryCoup { legitimacy_loss: f64 },      // -30 legitimacy
    MilitaryReform { combat_bonus: f64 },       // +15% combat strength for 25 ticks
    TerroristAttack { casualties: u64 },        // -5000 population, -5 legitimacy
    VeteranUprise { morale_bonus: f64 },        // +20 morale for all armies
    
    // Diplomatic Events (3)
    PeaceMovement { war_exhaustion_reduction: f64 }, // -50% war exhaustion
    BorderIncident { tension_increase: f64 },        // Raises tension with neighbors
    DiplomaticTriumph { relation_bonus: i32 },       // +20 relations with all nations
    
    // Natural Disasters (4)
    Earthquake { infrastructure_damage: f64 },  // -30% infrastructure
    Flood { food_shortage: f64 },               // -40% food production
    Drought { population_loss: u64 },           // -2% population over 10 ticks
    Plague { mortality_rate: f64 },             // -10% population, -15 legitimacy
    
    // Social/Political Events (5)
    ElectionSuccess { legitimacy_gain: f64 },   // +15 legitimacy
    CorruptionScandal { legitimacy_loss: f64 }, // -20 legitimacy
    CulturalRenaissance { gdp_boost: f64 },     // +5% GDP, +10 legitimacy
    Strikes { production_penalty: f64 },        // -25% production for 10 ticks
    Immigration { population_gain: u64 },       // +50000 population, +5% GDP
}

impl EventType {
    /// Get the category this event belongs to
    pub fn category(&self) -> EventCategory {
        match self {
            EventType::TradeBonus { .. }
            | EventType::MarketCrash { .. }
            | EventType::ResourceDiscovery { .. }
            | EventType::EconomicReform { .. }
            | EventType::CurrencyCrisis { .. } => EventCategory::Economic,
            
            EventType::MilitaryCoup { .. }
            | EventType::MilitaryReform { .. }
            | EventType::TerroristAttack { .. }
            | EventType::VeteranUprise { .. } => EventCategory::Military,
            
            EventType::PeaceMovement { .. }
            | EventType::BorderIncident { .. }
            | EventType::DiplomaticTriumph { .. } => EventCategory::Diplomatic,
            
            EventType::Earthquake { .. }
            | EventType::Flood { .. }
            | EventType::Drought { .. }
            | EventType::Plague { .. } => EventCategory::Natural,
            
            EventType::ElectionSuccess { .. }
            | EventType::CorruptionScandal { .. }
            | EventType::CulturalRenaissance { .. }
            | EventType::Strikes { .. }
            | EventType::Immigration { .. } => EventCategory::Social,
        }
    }
    
    /// Get a human-readable name for the event
    pub fn name(&self) -> &str {
        match self {
            EventType::TradeBonus { .. } => "Trade Boom",
            EventType::MarketCrash { .. } => "Market Crash",
            EventType::ResourceDiscovery { .. } => "Resource Discovery",
            EventType::EconomicReform { .. } => "Economic Reform",
            EventType::CurrencyCrisis { .. } => "Currency Crisis",
            EventType::MilitaryCoup { .. } => "Military Coup",
            EventType::MilitaryReform { .. } => "Military Modernization",
            EventType::TerroristAttack { .. } => "Terrorist Attack",
            EventType::VeteranUprise { .. } => "Veteran Morale Boost",
            EventType::PeaceMovement { .. } => "Peace Movement",
            EventType::BorderIncident { .. } => "Border Incident",
            EventType::DiplomaticTriumph { .. } => "Diplomatic Success",
            EventType::Earthquake { .. } => "Earthquake",
            EventType::Flood { .. } => "Flood",
            EventType::Drought { .. } => "Drought",
            EventType::Plague { .. } => "Plague Outbreak",
            EventType::ElectionSuccess { .. } => "Successful Election",
            EventType::CorruptionScandal { .. } => "Corruption Scandal",
            EventType::CulturalRenaissance { .. } => "Cultural Renaissance",
            EventType::Strikes { .. } => "Labor Strikes",
            EventType::Immigration { .. } => "Immigration Wave",
        }
    }
}

/// Active world event affecting one or more nations
#[derive(Debug, Clone, Component)]
pub struct WorldEvent {
    pub id: EventId,
    pub event_type: EventType,
    pub target_nation: NationId,
    pub start_tick: u64,
    pub duration: u64,        // 0 for instant events
    pub remaining_ticks: u64,
}

impl WorldEvent {
    /// Check if event is still active
    pub fn is_active(&self) -> bool {
        self.remaining_ticks > 0 || self.duration == 0
    }
    
    /// Decrement remaining duration
    pub fn tick(&mut self) {
        if self.remaining_ticks > 0 {
            self.remaining_ticks -= 1;
        }
    }
}

/// Configuration for event probabilities and mechanics
#[derive(Debug, Clone)]
pub struct EventConfig {
    /// Base probability per tick (0.0 - 1.0)
    pub base_probability: f64,
    /// Maximum active events per nation
    pub max_events_per_nation: usize,
    /// Cooldown ticks between events for same nation
    pub event_cooldown: u64,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            base_probability: 0.02, // 2% chance per tick per nation
            max_events_per_nation: 2,
            event_cooldown: 20,
        }
    }
}

// ============================================================================
// EVENT PHASE
// ============================================================================

pub struct EventPhase {
    config: EventConfig,
    last_event_tick: HashMap<NationId, u64>,
}

impl EventPhase {
    pub fn new() -> Self {
        Self {
            config: EventConfig::default(),
            last_event_tick: HashMap::new(),
        }
    }
    
    pub fn with_config(config: EventConfig) -> Self {
        Self {
            config,
            last_event_tick: HashMap::new(),
        }
    }
    
    /// Check if nation can receive new event
    fn can_trigger_event(&self, nation_id: NationId, current_tick: u64, active_event_count: usize) -> bool {
        // Check max events limit
        if active_event_count >= self.config.max_events_per_nation {
            return false;
        }
        
        // Check cooldown
        if let Some(&last_tick) = self.last_event_tick.get(&nation_id) {
            if current_tick - last_tick < self.config.event_cooldown {
                return false;
            }
        }
        
        true
    }
    
    /// Roll for and spawn events for all nations
    fn trigger_events(&mut self, world: &mut World, current_tick: u64) {
        // Pre-calculate active event counts for all nations
        let mut active_event_counts: HashMap<NationId, usize> = HashMap::new();
        {
            let mut query = world.query::<&WorldEvent>();
            for event in query.iter(world) {
                if event.is_active() {
                    *active_event_counts.entry(event.target_nation).or_insert(0) += 1;
                }
            }
        }
        
        // Gather eligible nations
        let mut eligible_nations: Vec<(Entity, NationId)> = Vec::new();
        {
            let mut query = world.query::<(Entity, &Nation)>();
            for (entity, nation) in query.iter(world) {
                let active_count = active_event_counts.get(&nation.id).copied().unwrap_or(0);
                if self.can_trigger_event(nation.id, current_tick, active_count) {
                    eligible_nations.push((entity, nation.id));
                }
            }
        }
        
        // Determine which events to spawn (using RNG in scoped block)
        let events_to_spawn: Vec<(Entity, NationId, EventType)> = {
            let rng = world.resource::<DeterministicRng>();
            eligible_nations.into_iter()
                .filter(|_| rng.next_f64() < self.config.base_probability)
                .map(|(entity, nation_id)| {
                    let event_type = self.generate_random_event(rng);
                    (entity, nation_id, event_type)
                })
                .collect()
        };
        
        // Spawn the events (RNG borrow is now dropped)
        for (nation_entity, nation_id, event_type) in events_to_spawn {
            self.spawn_event(world, nation_entity, nation_id, event_type, current_tick);
            self.last_event_tick.insert(nation_id, current_tick);
        }
    }
    
    /// Generate a random event type
    fn generate_random_event(&self, rng: &DeterministicRng) -> EventType {
        let roll = rng.next_f64();
        
        // Distribution: 25% economic, 20% military, 15% diplomatic, 20% natural, 20% social
        if roll < 0.25 {
            self.generate_economic_event(rng)
        } else if roll < 0.45 {
            self.generate_military_event(rng)
        } else if roll < 0.60 {
            self.generate_diplomatic_event(rng)
        } else if roll < 0.80 {
            self.generate_natural_event(rng)
        } else {
            self.generate_social_event(rng)
        }
    }
    
    fn generate_economic_event(&self, rng: &DeterministicRng) -> EventType {
        match rng.next_usize(5) {
            0 => EventType::TradeBonus { gdp_modifier: 0.10 },
            1 => EventType::MarketCrash { gdp_penalty: 0.20 },
            2 => EventType::ResourceDiscovery { resource_bonus: 0.50 },
            3 => EventType::EconomicReform { growth_rate: 0.02 },
            _ => EventType::CurrencyCrisis { inflation: 0.15 },
        }
    }
    
    fn generate_military_event(&self, rng: &DeterministicRng) -> EventType {
        match rng.next_usize(4) {
            0 => EventType::MilitaryCoup { legitimacy_loss: 30.0 },
            1 => EventType::MilitaryReform { combat_bonus: 0.15 },
            2 => EventType::TerroristAttack { casualties: 5000 },
            _ => EventType::VeteranUprise { morale_bonus: 20.0 },
        }
    }
    
    fn generate_diplomatic_event(&self, rng: &DeterministicRng) -> EventType {
        match rng.next_usize(3) {
            0 => EventType::PeaceMovement { war_exhaustion_reduction: 0.50 },
            1 => EventType::BorderIncident { tension_increase: 0.20 },
            _ => EventType::DiplomaticTriumph { relation_bonus: 20 },
        }
    }
    
    fn generate_natural_event(&self, rng: &DeterministicRng) -> EventType {
        match rng.next_usize(4) {
            0 => EventType::Earthquake { infrastructure_damage: 0.30 },
            1 => EventType::Flood { food_shortage: 0.40 },
            2 => EventType::Drought { population_loss: 20000 },
            _ => EventType::Plague { mortality_rate: 0.10 },
        }
    }
    
    fn generate_social_event(&self, rng: &DeterministicRng) -> EventType {
        match rng.next_usize(5) {
            0 => EventType::ElectionSuccess { legitimacy_gain: 15.0 },
            1 => EventType::CorruptionScandal { legitimacy_loss: 20.0 },
            2 => EventType::CulturalRenaissance { gdp_boost: 0.05 },
            3 => EventType::Strikes { production_penalty: 0.25 },
            _ => EventType::Immigration { population_gain: 50000 },
        }
    }
    
    /// Spawn a new event entity
    fn spawn_event(
        &self,
        world: &mut World,
        _nation_entity: Entity,
        nation_id: NationId,
        event_type: EventType,
        current_tick: u64,
    ) {
        let duration = self.get_event_duration(&event_type);
        
        let event = WorldEvent {
            id: EventId::new(),
            event_type: event_type.clone(),
            target_nation: nation_id,
            start_tick: current_tick,
            duration,
            remaining_ticks: duration,
        };
        
        info!(
            nation = ?nation_id,
            event = event_type.name(),
            duration = duration,
            "World event triggered"
        );
        
        world.spawn(event);
    }
    
    /// Get duration for event type (0 = instant)
    fn get_event_duration(&self, event_type: &EventType) -> u64 {
        match event_type {
            // Instant events
            EventType::MilitaryCoup { .. }
            | EventType::TerroristAttack { .. }
            | EventType::CorruptionScandal { .. }
            | EventType::EconomicReform { .. }
            | EventType::DiplomaticTriumph { .. } => 0,
            
            // Short duration (10-15 ticks)
            EventType::MarketCrash { .. }
            | EventType::CurrencyCrisis { .. }
            | EventType::Strikes { .. }
            | EventType::Drought { .. } => 10,
            
            // Medium duration (20-25 ticks)
            EventType::TradeBonus { .. }
            | EventType::MilitaryReform { .. }
            | EventType::PeaceMovement { .. }
            | EventType::Earthquake { .. }
            | EventType::Flood { .. } => 20,
            
            // Long duration (30+ ticks)
            EventType::ResourceDiscovery { .. }
            | EventType::Plague { .. }
            | EventType::CulturalRenaissance { .. }
            | EventType::Immigration { .. } => 30,
            
            // Special cases
            EventType::VeteranUprise { .. }
            | EventType::ElectionSuccess { .. }
            | EventType::BorderIncident { .. } => 15,
        }
    }
    
    /// Count active events for a nation
    fn count_active_events(&self, world: &mut World, nation_id: NationId) -> usize {
        let mut query = world.query::<&WorldEvent>();
        query.iter(world)
            .filter(|event| event.target_nation == nation_id && event.is_active())
            .count()
    }
    
    /// Apply event effects and clean up expired events
    fn process_active_events(&mut self, world: &mut World) {
        // Gather events to apply
        let mut events_to_apply: Vec<(Entity, WorldEvent)> = Vec::new();
        {
            let mut query = world.query::<(Entity, &WorldEvent)>();
            for (entity, event) in query.iter(world) {
                if event.is_active() {
                    events_to_apply.push((entity, event.clone()));
                }
            }
        }
        
        // Apply effects
        for (_entity, event) in &events_to_apply {
            self.apply_event_effects(world, event);
        }
        
        // Update durations and despawn expired events
        let mut events_to_despawn: Vec<Entity> = Vec::new();
        {
            let mut query = world.query::<(Entity, &mut WorldEvent)>();
            for (entity, mut event) in query.iter_mut(world) {
                event.tick();
                if !event.is_active() && event.duration > 0 {
                    events_to_despawn.push(entity);
                    debug!(
                        event = event.event_type.name(),
                        nation = ?event.target_nation,
                        "Event expired"
                    );
                }
            }
        }
        
        // Despawn expired events
        for entity in events_to_despawn {
            world.despawn(entity);
        }
    }
    
    /// Apply effects of an active event
    fn apply_event_effects(&self, world: &mut World, event: &WorldEvent) {
        match &event.event_type {
            EventType::TradeBonus { gdp_modifier } => {
                self.apply_gdp_modifier(world, event.target_nation, *gdp_modifier);
            }
            EventType::MarketCrash { gdp_penalty } => {
                self.apply_gdp_modifier(world, event.target_nation, -*gdp_penalty);
            }
            EventType::ResourceDiscovery { resource_bonus } => {
                self.apply_resource_bonus(world, event.target_nation, *resource_bonus);
            }
            EventType::EconomicReform { growth_rate } => {
                self.apply_growth_rate_boost(world, event.target_nation, *growth_rate);
            }
            EventType::CurrencyCrisis { inflation } => {
                self.apply_gdp_modifier(world, event.target_nation, -*inflation);
                self.apply_legitimacy_change(world, event.target_nation, -10.0);
            }
            EventType::MilitaryCoup { legitimacy_loss } => {
                self.apply_legitimacy_change(world, event.target_nation, -*legitimacy_loss);
            }
            EventType::TerroristAttack { casualties } => {
                self.apply_population_loss(world, event.target_nation, *casualties);
                self.apply_legitimacy_change(world, event.target_nation, -5.0);
            }
            EventType::ElectionSuccess { legitimacy_gain } => {
                self.apply_legitimacy_change(world, event.target_nation, *legitimacy_gain);
            }
            EventType::CorruptionScandal { legitimacy_loss } => {
                self.apply_legitimacy_change(world, event.target_nation, -*legitimacy_loss);
            }
            EventType::Plague { mortality_rate } => {
                self.apply_population_loss_percentage(world, event.target_nation, *mortality_rate);
                self.apply_legitimacy_change(world, event.target_nation, -15.0);
            }
            EventType::Immigration { population_gain } => {
                self.apply_population_gain(world, event.target_nation, *population_gain);
                self.apply_gdp_modifier(world, event.target_nation, 0.05);
            }
            EventType::CulturalRenaissance { gdp_boost } => {
                self.apply_gdp_modifier(world, event.target_nation, *gdp_boost);
                self.apply_legitimacy_change(world, event.target_nation, 10.0);
            }
            _ => {
                // Other events have effects handled elsewhere or are placeholders
                debug!(
                    event = event.event_type.name(),
                    "Event has no implemented effect yet"
                );
            }
        }
    }
    
    // ========================================================================
    // EFFECT APPLICATION HELPERS
    // ========================================================================
    
    fn apply_gdp_modifier(&self, world: &mut World, nation_id: NationId, modifier: f64) {
        let mut query = world.query::<(&Nation, &mut GDP)>();
        for (nation, mut gdp) in query.iter_mut(world) {
            if nation.id == nation_id {
                gdp.value *= 1.0 + modifier;
                debug!(nation = ?nation_id, modifier = modifier, new_gdp = gdp.value, "GDP modified by event");
            }
        }
    }
    
    fn apply_growth_rate_boost(&self, world: &mut World, nation_id: NationId, boost: f64) {
        let mut query = world.query::<(&Nation, &mut GDP)>();
        for (nation, mut gdp) in query.iter_mut(world) {
            if nation.id == nation_id {
                gdp.growth_rate += boost;
                debug!(nation = ?nation_id, boost = boost, new_rate = gdp.growth_rate, "Growth rate increased");
            }
        }
    }
    
    fn apply_legitimacy_change(&self, world: &mut World, nation_id: NationId, delta: f64) {
        let mut query = world.query::<(&Nation, &mut Legitimacy)>();
        for (nation, mut legitimacy) in query.iter_mut(world) {
            if nation.id == nation_id {
                legitimacy.modify(delta);
                debug!(nation = ?nation_id, delta = delta, new_value = legitimacy.value, "Legitimacy changed by event");
            }
        }
    }
    
    fn apply_resource_bonus(&self, world: &mut World, nation_id: NationId, bonus: f64) {
        let mut query = world.query::<(&OwnedBy, &mut Resources)>();
        for (owner, mut resources) in query.iter_mut(world) {
            if owner.nation_id == nation_id {
                resources.food *= 1.0 + bonus;
                resources.iron *= 1.0 + bonus;
                resources.oil *= 1.0 + bonus;
                resources.rare_earths *= 1.0 + bonus;
            }
        }
    }
    
    fn apply_population_loss(&self, world: &mut World, nation_id: NationId, casualties: u64) {
        let mut query = world.query::<(&OwnedBy, &mut Population)>();
        for (owner, mut population) in query.iter_mut(world) {
            if owner.nation_id == nation_id {
                population.total = population.total.saturating_sub(casualties);
            }
        }
    }
    
    fn apply_population_loss_percentage(&self, world: &mut World, nation_id: NationId, percentage: f64) {
        let mut query = world.query::<(&OwnedBy, &mut Population)>();
        for (owner, mut population) in query.iter_mut(world) {
            if owner.nation_id == nation_id {
                let loss = (population.total as f64 * percentage) as u64;
                population.total = population.total.saturating_sub(loss);
            }
        }
    }
    
    fn apply_population_gain(&self, world: &mut World, nation_id: NationId, gain: u64) {
        let mut query = world.query::<(&OwnedBy, &mut Population)>();
        for (owner, mut population) in query.iter_mut(world) {
            if owner.nation_id == nation_id {
                population.total += gain;
                break; // Apply only to first matching province
            }
        }
    }
}

impl TickPhase for EventPhase {
    fn name(&self) -> &str {
        "Events"
    }
    
    fn execute(&mut self, world: &mut World) {
        // Get current tick (use 0 if resource doesn't exist)
        let current_tick = 0; // TODO: Add tick tracking from world resource
        
        // Process existing events (apply effects, update durations)
        self.process_active_events(world);
        
        // Trigger new events
        self.trigger_events(world, current_tick);
        
        debug!("EventPhase complete");
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::world::WorldState;
    
    #[test]
    fn test_event_creation() {
        let event_type = EventType::TradeBonus { gdp_modifier: 0.10 };
        assert_eq!(event_type.name(), "Trade Boom");
        assert_eq!(event_type.category(), EventCategory::Economic);
    }
    
    #[test]
    fn test_event_probability() {
        let mut world_state = WorldState::new(42);
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        
        let mut phase = EventPhase::new();
        phase.execute(&mut world_state.world);
        
        // Events should be possible to spawn (though random)
        let event_count = world_state.world.query::<&WorldEvent>().iter(&world_state.world).count();
        // With 2% probability and seed 42, we can't guarantee events, just that system runs
        assert!(event_count <= 1, "Should not spawn more than max events");
    }
    
    #[test]
    fn test_event_categories() {
        let economic = EventType::MarketCrash { gdp_penalty: 0.20 };
        let military = EventType::MilitaryCoup { legitimacy_loss: 30.0 };
        let natural = EventType::Earthquake { infrastructure_damage: 0.30 };
        
        assert_eq!(economic.category(), EventCategory::Economic);
        assert_eq!(military.category(), EventCategory::Military);
        assert_eq!(natural.category(), EventCategory::Natural);
    }
    
    #[test]
    fn test_event_duration() {
        let phase = EventPhase::new();
        
        let instant = EventType::MilitaryCoup { legitimacy_loss: 30.0 };
        let short = EventType::MarketCrash { gdp_penalty: 0.20 };
        let long = EventType::ResourceDiscovery { resource_bonus: 0.50 };
        
        assert_eq!(phase.get_event_duration(&instant), 0);
        assert_eq!(phase.get_event_duration(&short), 10);
        assert_eq!(phase.get_event_duration(&long), 30);
    }
    
    #[test]
    fn test_event_effects_legitimacy() {
        let mut world_state = WorldState::new(42);
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        
        let initial_legitimacy = world_state.world.get::<Legitimacy>(nation).unwrap().value;
        
        let phase = EventPhase::new();
        phase.apply_legitimacy_change(&mut world_state.world, nation_id, 10.0);
        
        let final_legitimacy = world_state.world.get::<Legitimacy>(nation).unwrap().value;
        assert_eq!(final_legitimacy, initial_legitimacy + 10.0);
    }
    
    #[test]
    fn test_max_events_per_nation() {
        let config = EventConfig {
            base_probability: 1.0, // 100% chance
            max_events_per_nation: 2,
            event_cooldown: 0,
        };
        
        let mut world_state = WorldState::new(123);
        let _nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        
        let mut phase = EventPhase::with_config(config);
        
        // Run multiple ticks
        for _ in 0..5 {
            phase.execute(&mut world_state.world);
        }
        
        // Should have at most max_events_per_nation
        let event_count = world_state.world.query::<&WorldEvent>().iter(&world_state.world).count();
        assert!(event_count <= 2, "Should respect max events limit");
    }
    
    #[test]
    fn test_event_expiration() {
        let mut world_state = WorldState::new(42);
        let nation = world_state.spawn_nation("Test Nation".to_string(), [255, 0, 0], false);
        let nation_id = world_state.world.get::<Nation>(nation).unwrap().id;
        
        // Manually spawn a short-duration event
        let mut event = WorldEvent {
            id: EventId::new(),
            event_type: EventType::MarketCrash { gdp_penalty: 0.20 },
            target_nation: nation_id,
            start_tick: 0,
            duration: 2,
            remaining_ticks: 2,
        };
        
        assert!(event.is_active());
        event.tick();
        assert!(event.is_active());
        event.tick();
        assert!(!event.is_active());
    }
    
    #[test]
    fn test_random_event_generation() {
        let rng = DeterministicRng::new(42);
        let phase = EventPhase::new();
        
        // Generate multiple events to test distribution
        let mut economic_count = 0;
        let mut military_count = 0;
        
        for _ in 0..100 {
            let event = phase.generate_random_event(&rng);
            match event.category() {
                EventCategory::Economic => economic_count += 1,
                EventCategory::Military => military_count += 1,
                _ => {}
            }
        }
        
        // Should have generated some of each category
        assert!(economic_count > 0, "Should generate economic events");
        assert!(military_count > 0, "Should generate military events");
    }
}
