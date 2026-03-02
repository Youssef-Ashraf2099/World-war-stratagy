//! Espionage subsystem (V0.7)
//! Manages spy operations, intelligence gathering, sabotage, counter-intelligence
//! Provides imperfect information about enemy nation states

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use tracing::{debug, info};

use crate::core::types::*;
use crate::core::deterministic::DeterministicRng;
use crate::core::tick::TickPhase;
use crate::subsystems::notifications::create_espionage_revealed_notification;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Unique identifier for spy agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct SpyAgentId(pub Uuid);

impl SpyAgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SpyAgentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for intelligence reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Serialize, Deserialize)]
pub struct IntelReportId(pub Uuid);

impl IntelReportId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for IntelReportId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of spy operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpyMissionType {
    /// Passive intelligence gathering
    Reconnaissance,
    /// Active resource/troop sabotage
    Sabotage,
    /// Attempt to eliminate enemy leader/general
    Assassination,
    /// Spread propaganda and disinformation
    Propaganda,
    /// Infiltrate government/military
    Infiltration,
}

/// Status of a spy operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionStatus {
    Active,
    Completed,
    Failed,
    Discovered,
}

/// Effectiveness rating of a spy agent (0-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRating {
    Amateur,      // 0-30
    Professional, // 30-70
    Elite,        // 70-100
}

impl AgentRating {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 70.0 => AgentRating::Elite,
            s if s >= 30.0 => AgentRating::Professional,
            _ => AgentRating::Amateur,
        }
    }

    pub fn success_probability(&self) -> f64 {
        match self {
            AgentRating::Amateur => 0.40,
            AgentRating::Professional => 0.65,
            AgentRating::Elite => 0.85,
        }
    }
}

/// Individual spy agent
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct SpyAgent {
    pub id: SpyAgentId,
    pub owner_nation_id: NationId,
    pub target_nation_id: NationId,
    /// Skill score 0-100
    pub skill: f64,
    /// Current cover status: higher = safer
    pub cover: f64,
    /// How many ticks before cover is blown
    pub compromised_in: Option<u64>,
    /// Accumulated intelligence from operations
    pub intelligence_gathered: f64,
    pub current_mission: Option<SpyMissionType>,
    pub mission_status: MissionStatus,
    pub mission_start_tick: u64,
}

impl SpyAgent {
    pub fn new(
        owner_nation_id: NationId,
        target_nation_id: NationId,
        skill: f64,
        mission_start_tick: u64,
    ) -> Self {
        Self {
            id: SpyAgentId::new(),
            owner_nation_id,
            target_nation_id,
            skill: skill.clamp(0.0, 100.0),
            cover: 80.0, // Start with good cover
            compromised_in: None,
            intelligence_gathered: 0.0,
            current_mission: None,
            mission_status: MissionStatus::Active,
            mission_start_tick,
        }
    }

    pub fn rating(&self) -> AgentRating {
        AgentRating::from_score(self.skill)
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self.mission_status, MissionStatus::Discovered)
    }

    pub fn is_compromised(&self) -> bool {
        self.cover < 20.0
    }
}

/// Intelligence report about a target nation
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct IntelligenceReport {
    pub id: IntelReportId,
    pub observer_nation_id: NationId,
    pub target_nation_id: NationId,
    pub tick_reported: u64,
    pub accuracy: f64, // 0.0 - 100.0, how accurate is this info
    pub reported_military_strength: f64,
    pub reported_gdp: f64,
    pub reported_legitimacy: f64,
    pub has_war_intel: bool,
    pub suspected_alliances: Vec<NationId>,
}

impl IntelligenceReport {
    pub fn new(
        observer_nation_id: NationId,
        target_nation_id: NationId,
        tick_reported: u64,
        accuracy: f64,
    ) -> Self {
        Self {
            id: IntelReportId::new(),
            observer_nation_id,
            target_nation_id,
            tick_reported,
            accuracy: accuracy.clamp(0.0, 100.0),
            reported_military_strength: 0.0,
            reported_gdp: 0.0,
            reported_legitimacy: 50.0,
            has_war_intel: false,
            suspected_alliances: Vec::new(),
        }
    }
}

/// Intelligence network for a nation: tracks all intelligence about other nations
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct IntelligenceNetwork {
    pub nation_id: NationId,
    /// Latest intelligence reports by target nation
    pub intelligence_by_target: HashMap<NationId, IntelligenceReport>,
    /// Average counter-intelligence capability of enemies
    pub average_counter_intel: f64,
    /// Total resources invested in espionage this tick
    pub espionage_budget: f64,
    /// Detection risk: higher = more likely to be discovered (0-100)
    pub detection_risk: f64,
}

impl IntelligenceNetwork {
    pub fn new(nation_id: NationId) -> Self {
        Self {
            nation_id,
            intelligence_by_target: HashMap::new(),
            average_counter_intel: 10.0,
            espionage_budget: 0.0,
            detection_risk: 10.0,
        }
    }

    /// Get intelligence report for target nation (or None if no intel)
    pub fn get_intelligence(&self, target: NationId) -> Option<&IntelligenceReport> {
        self.intelligence_by_target.get(&target)
    }

    /// Add or update intelligence report
    pub fn update_intelligence(&mut self, report: IntelligenceReport) {
        self.intelligence_by_target.insert(report.target_nation_id, report);
    }

    /// Decay intelligence accuracy over time
    pub fn decay_intelligence(&mut self) {
        const ACCURACY_DECAY: f64 = 0.5; // Lost 0.5% accuracy per tick
        for report in self.intelligence_by_target.values_mut() {
            report.accuracy = (report.accuracy - ACCURACY_DECAY).max(0.0);
        }
    }
}

/// Counter-intelligence capability of a nation
#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct CounterIntelligence {
    pub nation_id: NationId,
    /// Counter-intel capability (0-100): higher = better detection
    pub capability: f64,
    /// Active investigations underway
    pub investigations: Vec<(NationId, u64)>, // (suspected_nation, ticks_investigating)
    /// Recently discovered spies
    pub discovered_agents: Vec<(NationId, u64)>, // (agent_owner, tick_discovered)
}

impl CounterIntelligence {
    pub fn new(nation_id: NationId) -> Self {
        Self {
            nation_id,
            capability: 20.0,
            investigations: Vec::new(),
            discovered_agents: Vec::new(),
        }
    }

    /// Check if a spy would be detected based on cover and counter-intel
    pub fn would_detect_spy(&self, agent: &SpyAgent, rng: &DeterministicRng) -> bool {
        // Detection probability increases with lower cover and higher counter-intel
        let base_detection = (100.0 - agent.cover) / 100.0;
        let counter_intel_factor = self.capability / 100.0;
        let detection_probability = (base_detection * 0.7 + counter_intel_factor * 0.3).min(0.95);

        rng.next_f64() < detection_probability
    }
}

// ============================================================================
// ESPIONAGE PHASE
// ============================================================================

/// Espionage phase for the tick pipeline
pub struct EspionagePhase;

impl EspionagePhase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_phase(&mut self, world: &mut World) {
        // Step 1: Execute ongoing missions
        Self::execute_missions(world);

        // Step 2: Manage agent cover (increases chance of discovery)
        Self::degrade_cover();

        // Step 3: Detect compromised agents
        Self::detect_compromised_agents(world);

        // Step 4: Decay intelligence information (older intel becomes less accurate)
        Self::decay_intelligence_reports();

        // Step 5: Counter-intelligence investigations
        Self::counter_intelligence_sweep(world);

        debug!("EspionagePhase complete");
    }

    /// Execute active spy missions and gather intelligence
    fn execute_missions(world: &mut World) {
        // Get seed from somewhere - for now use 0 as placeholder
        // In real usage, this would come from WorldState
        let rng = DeterministicRng::new(42);

        // Collect spy agents
        let agents: Vec<SpyAgent> = {
            let mut query = world.query::<&SpyAgent>();
            query
                .iter(world)
                .filter(|a| a.is_alive())
                .cloned()
                .collect()
        };

        // Collect target nations' state for intel gathering
        let nation_states: HashMap<NationId, (f64, f64)> = {
            let mut result = HashMap::new();
            let mut query = world.query::<(&Nation, Option<&GDP>, Option<&Legitimacy>)>();
            for (nation, gdp, legitimacy) in query.iter(world) {
                let gdp_val = gdp.map(|g| g.value).unwrap_or(1_000_000.0);
                let legit_val = legitimacy.map(|l| l.value).unwrap_or(50.0);
                result.insert(nation.id, (gdp_val, legit_val));
            }
            result
        };

        // Execute each agent's mission
        for agent in agents {
            if let Some(mission_type) = agent.current_mission {
                let target_state = nation_states.get(&agent.target_nation_id);

                match mission_type {
                    SpyMissionType::Reconnaissance => {
                        Self::execute_reconnaissance(world, &agent, target_state, &rng);
                    }
                    SpyMissionType::Sabotage => {
                        Self::execute_sabotage(world, &agent);
                    }
                    SpyMissionType::Propaganda => {
                        Self::execute_propaganda(world, &agent);
                    }
                    _ => {} // Other mission types handled separately
                }
            }
        }
    }

    /// Gather intelligence about target nation through reconnaissance
    fn execute_reconnaissance(
        world: &mut World,
        agent: &SpyAgent,
        target_state: Option<&(f64, f64)>,
        rng: &DeterministicRng,
    ) {
        if let Some((gdp, legitimacy)) = target_state {
            // Higher skill = more accurate intelligence
            let accuracy = agent.skill + (rng.next_f64() * 20.0);

            let mut report = IntelligenceReport::new(
                agent.owner_nation_id,
                agent.target_nation_id,
                0, // tick - would come from world resource
                accuracy,
            );

            // Add noise proportional to inverse of accuracy
            let noise_factor = (100.0 - accuracy) / 100.0;
            report.reported_gdp = gdp * (1.0 + (rng.next_f64() - 0.5) * 2.0 * noise_factor);
            report.reported_legitimacy = legitimacy * (1.0 + (rng.next_f64() - 0.5) * 2.0 * noise_factor);
            report.has_war_intel = accuracy > 50.0;

            // Update intelligence network
            let mut query = world.query::<&mut IntelligenceNetwork>();
            for mut network in query.iter_mut(world) {
                if network.nation_id == agent.owner_nation_id {
                    network.update_intelligence(report.clone());
                    break;
                }
            }
        }
    }

    /// Perform sabotage: reduce target's resources
    fn execute_sabotage(world: &mut World, agent: &SpyAgent) {
        // Find target nation's owned provinces and reduce resources
        let mut query = world.query::<(&OwnedBy, &mut Resources)>();
        for (owned_by, mut resources) in query.iter_mut(world) {
            if owned_by.nation_id == agent.target_nation_id {
                // Sabotage reduces resources proportional to agent skill
                let sabotage_amount = (agent.skill / 100.0) * 100.0;
                resources.food = (resources.food - sabotage_amount).max(0.0);
                break; // Only sabotage one province per mission
            }
        }
    }

    /// Execute propaganda: reduce target nation's legitimacy
    fn execute_propaganda(world: &mut World, agent: &SpyAgent) {
        let mut query = world.query::<(&Nation, &mut Legitimacy)>();
        for (nation, mut legitimacy) in query.iter_mut(world) {
            if nation.id == agent.target_nation_id {
                // Propaganda reduces legitimacy based on agent skill
                let legitimacy_loss = (agent.skill / 100.0) * 5.0;
                legitimacy.modify(-legitimacy_loss);
                break;
            }
        }
    }

    /// Degrade agent cover over time (repeated missions increase detection risk)
    fn degrade_cover() {
        const COVER_DEGRADATION: f64 = 2.0; // Cover degrades 2% per tick
        // This would need world access - placeholder for now
    }

    /// Detect completely compromised agents
    fn detect_compromised_agents(world: &mut World) {
        let rng = DeterministicRng::new(42);

        let mut agents_to_discover = Vec::new();
        let mut query = world.query::<&SpyAgent>();
        for agent in query.iter(world) {
            if agent.is_compromised() && rng.next_f64() < 0.3 {
                agents_to_discover.push((agent.owner_nation_id, agent.id, agent.target_nation_id));
            }
        }

        // Remove discovered agents and log in counter-intelligence
        for (_owner_nation, agent_id, target_nation) in agents_to_discover {
            let mut query = world.query::<&mut SpyAgent>();
            for mut agent in query.iter_mut(world) {
                if agent.id == agent_id {
                    agent.mission_status = MissionStatus::Discovered;
                }
            }

            // Create notification for espionage revealed
            create_espionage_revealed_notification(world, _owner_nation, target_nation, "Unknown Operation".to_string(), 0);

            // Log in target nation's counter-intelligence
            let mut query = world.query::<&mut CounterIntelligence>();
            for mut counter_intel in query.iter_mut(world) {
                if counter_intel.nation_id == target_nation {
                    counter_intel.discovered_agents.push((_owner_nation, 0)); // tick would be current
                }
            }
        }
    }

    /// Decay intelligence accuracy over time
    fn decay_intelligence_reports() {
        // This would need world access - placeholder for now
    }

    /// Run counter-intelligence sweeps
    fn counter_intelligence_sweep(world: &mut World) {
        let rng = DeterministicRng::new(42);

        let mut query = world.query::<&mut CounterIntelligence>();
        for mut counter_intel in query.iter_mut(world) {
            // Increase counter-intel capability slightly each tick
            counter_intel.capability = (counter_intel.capability + 0.1).min(100.0);

            // Process ongoing investigations
            counter_intel.investigations.retain_mut(|(_suspected_nation, ticks_investigating)| {
                *ticks_investigating += 1;

                // Investigation succeeds after 5 ticks with 60% probability
                if *ticks_investigating >= 5 && rng.next_f64() < 0.6 {
                    false // Remove from investigations
                } else {
                    true
                }
            });
        }
    }
}

impl Default for EspionagePhase {
    fn default() -> Self {
        Self::new()
    }
}

impl TickPhase for EspionagePhase {
    fn name(&self) -> &str {
        "Espionage"
    }

    fn execute(&mut self, world: &mut World) {
        self.execute_phase(world);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spy_agent_creation() {
        let nation_a = NationId::new();
        let nation_b = NationId::new();
        let agent = SpyAgent::new(nation_a, nation_b, 75.0, 0);

        assert_eq!(agent.owner_nation_id, nation_a);
        assert_eq!(agent.target_nation_id, nation_b);
        assert_eq!(agent.skill, 75.0);
        assert!(agent.is_alive());
        assert!(!agent.is_compromised());
    }

    #[test]
    fn test_agent_rating_classification() {
        let amateur = SpyAgent::new(NationId::new(), NationId::new(), 25.0, 0);
        let professional = SpyAgent::new(NationId::new(), NationId::new(), 50.0, 0);
        let elite = SpyAgent::new(NationId::new(), NationId::new(), 85.0, 0);

        assert_eq!(amateur.rating(), AgentRating::Amateur);
        assert_eq!(professional.rating(), AgentRating::Professional);
        assert_eq!(elite.rating(), AgentRating::Elite);
    }

    #[test]
    fn test_agent_success_probability() {
        assert_eq!(AgentRating::Amateur.success_probability(), 0.40);
        assert_eq!(AgentRating::Professional.success_probability(), 0.65);
        assert_eq!(AgentRating::Elite.success_probability(), 0.85);
    }

    #[test]
    fn test_intelligence_network_decay() {
        let mut network = IntelligenceNetwork::new(NationId::new());
        let report = IntelligenceReport::new(
            NationId::new(),
            NationId::new(),
            0,
            100.0,
        );

        let target = report.target_nation_id;
        network.update_intelligence(report);
        assert_eq!(
            network.get_intelligence(target).unwrap().accuracy,
            100.0
        );

        network.decay_intelligence();
        assert!(network.get_intelligence(target).unwrap().accuracy < 100.0);
    }

    #[test]
    fn test_counter_intelligence_capability_increase() {
        let mut counter_intel = CounterIntelligence::new(NationId::new());
        let initial_capability = counter_intel.capability;

        // Simulate capability improvement
        counter_intel.capability = (counter_intel.capability + 0.1).min(100.0);
        assert!(counter_intel.capability > initial_capability);
    }

    #[test]
    fn test_agent_cover_degradation() {
        let mut agent = SpyAgent::new(NationId::new(), NationId::new(), 70.0, 0);
        agent.current_mission = Some(SpyMissionType::Reconnaissance);

        let initial_cover = agent.cover;
        const COVER_DEGRADATION: f64 = 2.0;
        agent.cover = (agent.cover - COVER_DEGRADATION).max(0.0);

        assert!(agent.cover < initial_cover);
        assert!(!agent.is_compromised()); // cover still > 20
    }

    #[test]
    fn test_agent_becomes_compromised() {
        let mut agent = SpyAgent::new(NationId::new(), NationId::new(), 50.0, 0);
        agent.cover = 10.0; // Below compromise threshold

        assert!(agent.is_compromised());
        assert_eq!(agent.mission_status, MissionStatus::Active);
    }

    #[test]
    fn test_intelligence_report_accuracy_clamping() {
        let report1 = IntelligenceReport::new(NationId::new(), NationId::new(), 0, 150.0);
        let report2 = IntelligenceReport::new(NationId::new(), NationId::new(), 0, -50.0);

        assert_eq!(report1.accuracy, 100.0);
        assert_eq!(report2.accuracy, 0.0);
    }

    #[test]
    fn test_spy_mission_type_variants() {
        // Ensure all mission types can be created and compared
        let recon = SpyMissionType::Reconnaissance;
        let sabotage = SpyMissionType::Sabotage;

        assert_ne!(recon, sabotage);
        assert_eq!(recon, SpyMissionType::Reconnaissance);
    }
}
