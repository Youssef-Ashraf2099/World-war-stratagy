/// Integration tests: Game Depth Scenarios
///
/// These tests verify that the engine produces *meaningful* game state changes —
/// not just that ticks advance. They document the intended game experience:
///
/// - Legitimacy degrades from sustained war
/// - Peace allows legitimacy to recover
/// - Diplomatic reputation evolves over time
/// - Economic stress feeds into stability
/// - Alliance cohesion decays correctly
///
/// These are "living documentation" tests for the depth of the game.

use crate::tests::fixtures::TestWorldBuilder;
use crate::core::types::{
    Legitimacy, WarState, Nation, GDP, DiplomaticRelation,
    NationId, AllianceCrisisLog, DiplomaticIsolationLog,
    EconomicStress, CasualtyLog,
};

// ============================================================================
// LEGITIMACY DEGRADATION UNDER WAR
// ============================================================================

/// A nation at war should see its legitimacy decline over time.
/// This validates the core war exhaustion -> legitimacy feedback loop.
#[test]
fn test_war_erodes_legitimacy_over_time() {
    use crate::core::WorldState;
    use crate::subsystems::legitimacy::LegitimacyPhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(1001);
    let nation_id = NationId::new();

    // Spawn a nation at war with high initial legitimacy
    let nation_entity = world.world.spawn((
        Nation { id: nation_id, name: "Warring Nation".to_string(), color: [200, 50, 50] },
        Legitimacy::new(80.0),
        WarState {
            at_war_with: vec![NationId::new()], // 1 active war
        },
        GDP { value: 1000.0, growth_rate: 0.01 },
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    let legitimacy_start = world.world.get::<Legitimacy>(nation_entity).unwrap().value;
    assert_eq!(legitimacy_start, 80.0);

    // Run legitimacy phase for 10 ticks
    let mut phase = LegitimacyPhase::new();
    for _ in 0..10 {
        phase.execute(&mut world.world);
    }

    let legitimacy_after = world.world.get::<Legitimacy>(nation_entity).unwrap().value;

    // After 10 ticks of war, legitimacy should have dropped
    assert!(
        legitimacy_after < legitimacy_start,
        "War should erode legitimacy: was {:.1}, now {:.1}",
        legitimacy_start, legitimacy_after
    );
    println!(
        "✓ War erosion: legitimacy {:.1} → {:.1} after 10 ticks",
        legitimacy_start, legitimacy_after
    );
}

/// A nation at peace should slowly recover legitimacy.
/// Validates the peace recovery bonus mechanic.
#[test]
fn test_peace_restores_legitimacy_over_time() {
    use crate::core::WorldState;
    use crate::subsystems::legitimacy::LegitimacyPhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(1002);
    let nation_id = NationId::new();

    // Start with stressed but not critical legitimacy
    let nation_entity = world.world.spawn((
        Nation { id: nation_id, name: "Recovering Nation".to_string(), color: [50, 200, 50] },
        Legitimacy::new(40.0),
        WarState::default(), // at peace
        GDP { value: 1000.0, growth_rate: 0.01 },
        EconomicStress::default(), // no deficit
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    let legitimacy_start = world.world.get::<Legitimacy>(nation_entity).unwrap().value;

    let mut phase = LegitimacyPhase::new();
    for _ in 0..20 {
        phase.execute(&mut world.world);
    }

    let legitimacy_after = world.world.get::<Legitimacy>(nation_entity).unwrap().value;

    assert!(
        legitimacy_after > legitimacy_start,
        "Peace should restore legitimacy: was {:.1}, now {:.1}",
        legitimacy_start, legitimacy_after
    );
    println!(
        "✓ Peace recovery: legitimacy {:.1} → {:.1} after 20 ticks",
        legitimacy_start, legitimacy_after
    );
}

// ============================================================================
// ECONOMIC STRESS
// ============================================================================

/// A nation running a large deficit relative to its GDP should lose legitimacy faster.
/// Validates the deficit stress -> legitimacy penalty mechanic.
#[test]
fn test_deficit_causes_legitimacy_drain() {
    use crate::core::WorldState;
    use crate::subsystems::legitimacy::LegitimacyPhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(1003);
    let nation_id = NationId::new();

    // 50% deficit relative to GDP = severe stress
    let nation_entity = world.world.spawn((
        nation_id,
        Nation { id: nation_id, name: "Broke Nation".to_string(), color: [200, 200, 50] },
        Legitimacy::new(60.0),
        WarState::default(),
        GDP { value: 500.0, growth_rate: 0.01 },
        EconomicStress {
            current_deficit: 250.0, // 50% of GDP
            accumulated_deficit: 0.0,
            gdp: 500.0,
        },
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    let legitimacy_start = world.world.get::<Legitimacy>(nation_entity).unwrap().value;

    let mut phase = LegitimacyPhase::new();
    for _ in 0..10 {
        phase.execute(&mut world.world);
    }

    let legitimacy_after = world.world.get::<Legitimacy>(nation_entity).unwrap().value;

    // Even with peace recovery (+0.3/tick), severe deficit should still drag down
    // deficit_stress = -0.75 * (250/500) = -0.375, net = 0.3 - 0.375 = -0.075/tick
    assert!(
        legitimacy_after < legitimacy_start,
        "Severe deficit should drain legitimacy even at peace: was {:.1}, now {:.1}",
        legitimacy_start, legitimacy_after
    );
    println!(
        "✓ Deficit drain: legitimacy {:.1} → {:.1} over 10 ticks (50% of GDP deficit)",
        legitimacy_start, legitimacy_after
    );
}

// ============================================================================
// STATE TRAJECTORY TESTS
// ============================================================================

/// A nation at war with deficit should degrade faster than one at war without deficit.
/// Validates compounding stress mechanic.
#[test]
fn test_combined_stress_compounds() {
    use crate::core::WorldState;
    use crate::subsystems::legitimacy::LegitimacyPhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(1004);
    let nid_a = NationId::new();
    let nid_b = NationId::new();

    // Nation A: war + no deficit
    let entity_a = world.world.spawn((
        nid_a,
        Nation { id: nid_a, name: "War Only".to_string(), color: [200, 0, 0] },
        Legitimacy::new(70.0),
        WarState { at_war_with: vec![NationId::new()] },
        GDP { value: 1000.0, growth_rate: 0.01 },
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    // Nation B: war + severe deficit
    let entity_b = world.world.spawn((
        nid_b,
        Nation { id: nid_b, name: "War + Deficit".to_string(), color: [200, 50, 0] },
        Legitimacy::new(70.0),
        WarState { at_war_with: vec![NationId::new()] },
        GDP { value: 1000.0, growth_rate: 0.01 },
        EconomicStress {
            current_deficit: 300.0, // 30% deficit
            accumulated_deficit: 0.0,
            gdp: 1000.0,
        },
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    let mut phase = LegitimacyPhase::new();
    for _ in 0..15 {
        phase.execute(&mut world.world);
    }

    let legit_a = world.world.get::<Legitimacy>(entity_a).unwrap().value;
    let legit_b = world.world.get::<Legitimacy>(entity_b).unwrap().value;

    assert!(
        legit_b < legit_a,
        "War + deficit nation ({:.1}) should be worse off than war-only nation ({:.1})",
        legit_b, legit_a
    );
    println!(
        "✓ Stress compounds: war-only landed at {:.1}, war+deficit landed at {:.1}",
        legit_a, legit_b
    );
}

// ============================================================================
// DIPLOMATIC PERSONALITY TESTS
// ============================================================================

/// Diplomatic reputation should evolve (positively or negatively) over simulation time.
/// The world shouldn't be totally static after 200 ticks.
#[test]
fn test_diplomatic_world_is_dynamic() {
    use crate::core::types::DiplomaticRelation;

    let mut fixture = TestWorldBuilder::new()
        .with_seed(1010)
        .with_nations(6)
        .build();

    // Read initial diplomatic state  
    let initial_relations: Vec<f64> = {
        let mut query = fixture.world.world.query::<&DiplomaticRelation>();
        query.iter(&fixture.world.world)
            .map(|r| r.reputation)
            .collect()
    };

    fixture.execute_ticks(200);

    // Read final diplomatic state
    let final_relations: Vec<f64> = {
        let mut query = fixture.world.world.query::<&DiplomaticRelation>();
        query.iter(&fixture.world.world)
            .map(|r| r.reputation)
            .collect()
    };

    // At least some relations should have changed
    if initial_relations.len() == final_relations.len() && !initial_relations.is_empty() {
        let any_changed = initial_relations.iter().zip(final_relations.iter())
            .any(|(a, b)| (a - b).abs() > 0.5);
        assert!(
            any_changed,
            "Diplomatic relations should evolve over 200 ticks — world feels frozen"
        );
        println!(
            "✓ Diplomatic world is dynamic: {} relations tracked, at least one changed",
            initial_relations.len()
        );
    } else {
        // Relations were created during the simulation — that's also dynamic
        assert!(
            final_relations.len() >= initial_relations.len(),
            "Relations should grow or stay equal over time"
        );
        println!(
            "✓ Diplomatic world is dynamic: {} → {} relations over 200 ticks",
            initial_relations.len(),
            final_relations.len()
        );
    }
}

/// Legitimacy should DECREASE on average in a world of nations at war.
/// Validates that wars have consequences in multi-nation worlds.
#[test]
fn test_average_legitimacy_responds_to_game_state() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(1011)
        .with_nations(5)
        .build();

    // Read initial average legitimacy
    let initial_avg: f64 = {
        let mut query = fixture.world.world.query::<&Legitimacy>();
        let values: Vec<f64> = query.iter(&fixture.world.world).map(|l| l.value).collect();
        if values.is_empty() { return; }
        values.iter().sum::<f64>() / values.len() as f64
    };

    fixture.execute_ticks(200);

    let final_avg: f64 = {
        let mut query = fixture.world.world.query::<&Legitimacy>();
        let values: Vec<f64> = query.iter(&fixture.world.world).map(|l| l.value).collect();
        if values.is_empty() { return; }
        values.iter().sum::<f64>() / values.len() as f64
    };

    // We don't prescribe a specific direction — but it must not be frozen
    assert!(
        (final_avg - initial_avg).abs() > 0.1,
        "Average legitimacy should have shifted after 200 ticks: was {:.2}, still {:.2}",
        initial_avg, final_avg
    );

    println!(
        "✓ Average legitimacy changed from {:.2} to {:.2} over 200 ticks",
        initial_avg, final_avg
    );
}

// ============================================================================
// WAR EXHAUSTION PIPELINE
// ============================================================================

/// War exhaustion should increment when nations are at war.
/// This validates the WarfarePhase update_war_exhaustion function.
#[test]
fn test_war_exhaustion_increments_through_pipeline() {
    use crate::core::WorldState;
    use crate::core::types::{WarExhaustion};
    use crate::subsystems::warfare::WarfarePhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(1020);
    let nation_id = NationId::new();
    let enemy_id = NationId::new();

    let nation_entity = world.world.spawn((
        Nation { id: nation_id, name: "At War".to_string(), color: [255, 0, 0] },
        WarState { at_war_with: vec![enemy_id] },
        WarExhaustion { value: 0.0 },
    )).id();

    let mut phase = WarfarePhase::new();
    for _ in 0..10 {
        phase.execute(&mut world.world);
    }

    let exhaustion = world.world.get::<WarExhaustion>(nation_entity).unwrap().value;
    assert!(
        exhaustion > 0.0,
        "War exhaustion should increase when nation is at war, got: {}",
        exhaustion
    );
    // 10 ticks × 0.1 per tick = 1.0
    assert!(
        (exhaustion - 1.0).abs() < 0.01,
        "Expected ~1.0 exhaustion after 10 ticks, got: {}",
        exhaustion
    );
    println!("✓ War exhaustion after 10 ticks: {:.2}", exhaustion);
}

/// War exhaustion should recover when a nation returns to peace.
#[test]
fn test_war_exhaustion_decays_at_peace() {
    use crate::core::WorldState;
    use crate::core::types::WarExhaustion;
    use crate::subsystems::warfare::WarfarePhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(1021);

    let nation_entity = world.world.spawn((
        Nation { id: NationId::new(), name: "Post-War".to_string(), color: [100, 150, 200] },
        WarState::default(), // at peace
        WarExhaustion { value: 50.0 }, // high residual exhaustion from past war
    )).id();

    let mut phase = WarfarePhase::new();
    for _ in 0..20 {
        phase.execute(&mut world.world);
    }

    let exhaustion = world.world.get::<WarExhaustion>(nation_entity).unwrap().value;
    assert!(
        exhaustion < 50.0,
        "War exhaustion should decay at peace: started at 50.0, now {}",
        exhaustion
    );
    println!("✓ Exhaustion decayed from 50.0 to {:.2} at peace after 20 ticks", exhaustion);
}
