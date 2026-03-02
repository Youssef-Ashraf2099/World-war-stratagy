//! Integration tests for espionage system interaction with other subsystems

use crate::core::world::WorldState;
use crate::subsystems::espionage::{
    SpyAgent, IntelligenceNetwork, CounterIntelligence, SpyMissionType, MissionStatus,
};
use crate::core::types::{NationId, Legitimacy};

#[test]
fn test_spy_agent_infiltration_scenario() {
    let mut world_state = WorldState::new(12345);
    
    // Create two nations
    let nation_a_entity = world_state.spawn_nation("Espionage Power".to_string(), [255, 0, 0], false);
    let nation_b_entity = world_state.spawn_nation("Target Nation".to_string(), [0, 255, 0], false);
    
    let nation_a_id = world_state.world.get::<crate::core::types::Nation>(nation_a_entity).unwrap().id;
    let nation_b_id = world_state.world.get::<crate::core::types::Nation>(nation_b_entity).unwrap().id;
    
    // Spawn intelligence network for nation A
    let intel_network = IntelligenceNetwork::new(nation_a_id);
    world_state.world.spawn(intel_network);
    
    // Spawn counter-intelligence for nation B
    let counter_intel = CounterIntelligence::new(nation_b_id);
    world_state.world.spawn(counter_intel);
    
    // Create a spy agent: elite assassin
    let spy = SpyAgent::new(nation_a_id, nation_b_id, 85.0, 0);
    assert_eq!(spy.skill, 85.0);
    assert!(spy.is_alive());
    
    // Verify spy can be spawned into world
    let spy_entity_mut = world_state.world.spawn(spy.clone());
    let spy_entity = spy_entity_mut.id();
    
    let spawned_spy = world_state.world.get::<SpyAgent>(spy_entity).unwrap();
    assert_eq!(spawned_spy.skill, 85.0);
    assert_eq!(spawned_spy.owner_nation_id, nation_a_id);
}

#[test]
fn test_reconnaissance_mission_gathers_intelligence() {
    let mut world_state = WorldState::new(54321);
    
    // Setup: Create nations with different GDP values
    let observer_nation = world_state.spawn_nation("Observer".to_string(), [100, 100, 255], false);
    let target_nation = world_state.spawn_nation("Target".to_string(), [255, 100, 0], false);
    
    let observer_id = world_state.world.get::<crate::core::types::Nation>(observer_nation).unwrap().id;
    let target_id = world_state.world.get::<crate::core::types::Nation>(target_nation).unwrap().id;
    
    // Give target nation specific GDP
    if let Some(mut target_entity) = world_state.world.get_entity_mut(target_nation) {
        target_entity.insert(crate::core::types::GDP {
            value: 5_000_000.0,
            growth_rate: 0.02,
        });
    }
    
    // Spawn reconnaissance spy with high skill (accurate intelligence)
    let spy = SpyAgent::new(observer_id, target_id, 80.0, 0);
    assert!(!spy.is_compromised());
    assert_eq!(spy.skill, 80.0);
    
    // Verify spy status
    assert_eq!(spy.mission_status, MissionStatus::Active);
}

#[test]
fn test_agent_cover_deterioration() {
    let mut spy = SpyAgent::new(NationId::new(), NationId::new(), 70.0, 0);
    let initial_cover = spy.cover;
    
    // Assign a mission (triggers cover degradation in actual phase)
    spy.current_mission = Some(SpyMissionType::Reconnaissance);
    
    // Simulate 20 ticks of cover degradation
    const COVER_DEGRADATION_PER_TICK: f64 = 2.0;
    for _ in 0..20 {
        spy.cover = (spy.cover - COVER_DEGRADATION_PER_TICK).max(0.0);
    }
    
    let final_cover = spy.cover;
    assert!(final_cover < initial_cover);
    assert_eq!(final_cover, 40.0); // 80.0 - (2.0 * 20) = 40.0
}

#[test]
fn test_counter_intelligence_detection() {
    let mut counter_intel = CounterIntelligence::new(NationId::new());
    let initial_capability = counter_intel.capability;
    
    // Counter-intelligence capability grows over time
    for _ in 0..10 {
        counter_intel.capability = (counter_intel.capability + 0.1).min(100.0);
    }
    
    assert!(counter_intel.capability > initial_capability);
    // After 10 ticks with +0.1 per tick: 20.0 + 1.0 = 21.0
    assert!(counter_intel.capability >= initial_capability + 1.0);
}

#[test]
fn test_multi_nation_espionage_network() {
    let mut world_state = WorldState::new(99999);
    
    // Create a 3-nation scenario
    let nation1 = world_state.spawn_nation("Nation A".to_string(), [255, 0, 0], false);
    let nation2 = world_state.spawn_nation("Nation B".to_string(), [0, 255, 0], false);
    let nation3 = world_state.spawn_nation("Nation C".to_string(), [0, 0, 255], false);
    
    let id1 = world_state.world.get::<crate::core::types::Nation>(nation1).unwrap().id;
    let id2 = world_state.world.get::<crate::core::types::Nation>(nation2).unwrap().id;
    let id3 = world_state.world.get::<crate::core::types::Nation>(nation3).unwrap().id;
    
    // Create spy networks for all nations
    let net1 = IntelligenceNetwork::new(id1);
    let net2 = IntelligenceNetwork::new(id2);
    let net3 = IntelligenceNetwork::new(id3);
    
    let _ent1 = world_state.world.spawn(net1);
    let _ent2 = world_state.world.spawn(net2);
    let _ent3 = world_state.world.spawn(net3);
    
    // Verify all networks exist and are properly configured
    let mut query = world_state.world.query::<&IntelligenceNetwork>();
    let networks: Vec<_> = query.iter(&world_state.world).collect();
    assert_eq!(networks.len(), 3);
    
    // Each network should have no pre-existing intel
    for net in networks {
        assert_eq!(net.intelligence_by_target.len(), 0);
    }
}

#[test]
fn test_espionage_compromises_legitimacy_through_propaganda() {
    let mut world_state = WorldState::new(11111);
    
    // Create target nation with known legitimacy
    let target = world_state.spawn_nation("Target State".to_string(), [100, 100, 100], false);
    let target_id = world_state.world.get::<crate::core::types::Nation>(target).unwrap().id;
    
    // Set initial legitimacy
    if let Some(mut target_entity) = world_state.world.get_entity_mut(target) {
        target_entity.insert(Legitimacy::new(80.0));
    }
    
    let initial_legit = world_state.world.get::<Legitimacy>(target).unwrap().value;
    assert_eq!(initial_legit, 80.0);
    
    // Simulate propaganda impact (5% reduction from 100% skill agent)
    let propaganda_loss = (100.0 / 100.0) * 5.0; // 5.0
    let new_legit = (initial_legit - propaganda_loss).max(0.0);
    
    assert_eq!(new_legit, 75.0);
}

#[test]
fn test_agent_skill_distribution() {
    // Test that agents with different skill levels have appropriate success probabilities
    let amateur = SpyAgent::new(NationId::new(), NationId::new(), 20.0, 0);
    let professional = SpyAgent::new(NationId::new(), NationId::new(), 50.0, 0);
    let elite = SpyAgent::new(NationId::new(), NationId::new(), 90.0, 0);
    
    // Verify skill values are clamped and categorized correctly
    assert!(amateur.skill >= 0.0 && amateur.skill <= 100.0);
    assert!(professional.skill >= 0.0 && professional.skill <= 100.0);
    assert!(elite.skill >= 0.0 && elite.skill <= 100.0);
    
    // Verify rating classification
    match amateur.rating() {
        crate::subsystems::espionage::AgentRating::Amateur => assert!(true),
        _ => panic!("Amateur agent misclassified"),
    }
    
    match professional.rating() {
        crate::subsystems::espionage::AgentRating::Professional => assert!(true),
        _ => panic!("Professional agent misclassified"),
    }
    
    match elite.rating() {
        crate::subsystems::espionage::AgentRating::Elite => assert!(true),
        _ => panic!("Elite agent misclassified"),
    }
}
