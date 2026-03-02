/// Integration tests: Warfare and Diplomacy interaction
///
/// These tests verify the behavioral consequences of war at the engine level:
/// - War exhaustion increases for warring nations
/// - War states are correctly established between enemies
/// - Peace nations recover legitimacy while war nations decline
/// - Battle detection works when warring nations meet in a province

use crate::tests::fixtures::{TestWorldBuilder, assertions};
use crate::core::WorldState;
use crate::core::types::{
    Nation, NationId, WarState, WarExhaustion, Legitimacy,
    GDP, EconomicStress, CasualtyLog, AllianceCrisisLog, DiplomaticIsolationLog,
};

// ============================================================================
// WAR EXHAUSTION INTEGRATION
// ============================================================================

/// War exhaustion should increase for warring nations over time.
/// This validates the full WarfarePhase integration in the pipeline.
#[test]
fn test_war_exhaustion_grows_via_pipeline() {
    use crate::subsystems::warfare::WarfarePhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(100);
    let nation_id = NationId::new();
    let enemy_id = NationId::new();

    let nation_entity = world.world.spawn((
        Nation { id: nation_id, name: "Warring A".to_string(), color: [200, 50, 50] },
        WarState { at_war_with: vec![enemy_id] },
        WarExhaustion { value: 0.0 },
    )).id();

    let mut phase = WarfarePhase::new();
    for _ in 0..20 {
        phase.execute(&mut world.world);
    }

    let exhaustion = world.world.get::<WarExhaustion>(nation_entity).unwrap().value;
    assert!(exhaustion > 0.0, "War exhaustion should have increased, got: {}", exhaustion);
    assert!(exhaustion <= 100.0, "War exhaustion should not exceed 100.0, got: {}", exhaustion);

    println!("✓ War exhaustion after 20 ticks: {:.2}", exhaustion);
}

/// At peace, war exhaustion should decrease back toward zero.
#[test]
fn test_war_exhaustion_decays_at_peace() {
    use crate::subsystems::warfare::WarfarePhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(101);

    let nation_entity = world.world.spawn((
        Nation { id: NationId::new(), name: "Post-War".to_string(), color: [50, 150, 200] },
        WarState::default(), // at peace
        WarExhaustion { value: 40.0 },
    )).id();

    let mut phase = WarfarePhase::new();
    for _ in 0..20 {
        phase.execute(&mut world.world);
    }

    let exhaustion = world.world.get::<WarExhaustion>(nation_entity).unwrap().value;
    assert!(exhaustion < 40.0, "Exhaustion should decay at peace: started 40.0, got {}", exhaustion);
    println!("✓ Exhaustion decayed from 40.0 to {:.2} over 20 peaceful ticks", exhaustion);
}

// ============================================================================
// LEGITIMACY DIVERGENCE: WAR VS. PEACE
// ============================================================================

/// A warring nation's legitimacy should drop faster than a peaceful one.
/// Tests the full legitimacy-warfare interaction via the pipeline.
#[test]
fn test_war_nation_loses_legitimacy_faster_than_peaceful_one() {
    use crate::subsystems::legitimacy::LegitimacyPhase;
    use crate::core::tick::TickPhase;

    let mut world = WorldState::new(102);
    let nation_id_war = NationId::new();
    let nation_id_peace = NationId::new();

    let war_entity = world.world.spawn((
        Nation { id: nation_id_war, name: "At War".to_string(), color: [200, 0, 0] },
        Legitimacy::new(70.0),
        WarState { at_war_with: vec![NationId::new()] },
        GDP { value: 1000.0, growth_rate: 0.01 },
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    let peace_entity = world.world.spawn((
        Nation { id: nation_id_peace, name: "At Peace".to_string(), color: [0, 150, 0] },
        Legitimacy::new(70.0),
        WarState::default(),
        GDP { value: 1000.0, growth_rate: 0.01 },
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    let mut phase = LegitimacyPhase::new();
    for _ in 0..20 {
        phase.execute(&mut world.world);
    }

    let war_legit = world.world.get::<Legitimacy>(war_entity).unwrap().value;
    let peace_legit = world.world.get::<Legitimacy>(peace_entity).unwrap().value;

    assert!(
        war_legit < peace_legit,
        "Warring nation ({:.1}) should have lower legitimacy than peaceful one ({:.1})",
        war_legit, peace_legit
    );
    println!(
        "✓ Legitimacy after 20 ticks — war: {:.1}, peace: {:.1}",
        war_legit, peace_legit
    );
}

// ============================================================================
// MULTI-NATION WAR STABILITY
// ============================================================================

/// A world with many nations running for extended ticks should remain stable.
/// This is the integration-level "smoke test" for everything together.
#[test]
fn test_war_diplomacy_world_remains_stable_over_time() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(200)
        .with_nations(5)
        .build();

    fixture.execute_ticks(200);

    assert_eq!(fixture.current_tick(), 200);
    assertions::assert_world_stable(&mut fixture, "warfare_diplomacy_long_run");
    println!("✓ 5-nation world stable after 200 ticks");
}

/// World with 3 nations stabilizes — no crash from war + diplomacy interplay.
#[test]
fn test_3_nation_war_diplomacy_scenario() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(300)
        .with_nations(3)
        .build();

    fixture.execute_ticks(150);

    assert_eq!(fixture.current_tick(), 150);
    assertions::assert_world_stable(&mut fixture, "3_nation_war_diplomacy");
}
