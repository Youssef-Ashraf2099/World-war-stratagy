/// Integration tests: Multi-nation complex scenarios
///
/// These tests verify emergent behavior and system stability when many
/// nations interact over extended timeframes.
///
/// We check for:
/// - Engine execution stability without crashes
/// - Global metric consistency (total nations doesn't exceed bounds)
/// - Diplomatic relationship evolution in a crowded world

use crate::tests::fixtures::{TestWorldBuilder, assertions};
use crate::core::types::{Nation, Legitimacy, GDP, DiplomaticRelation};

#[test]
fn test_5_nation_world_stability_and_metrics() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(700)
        .with_nations(5)
        .build();

    let initial_nation_count = fixture.world.world.query::<&Nation>().iter(&fixture.world.world).count();

    // Run for a medium duration
    let metrics = fixture.execute_ticks_timed(100);

    assert_eq!(fixture.current_tick(), 100);
    println!("5-nation 100-tick performance: {}", metrics.summary());

    // Verify engine consistency
    assertions::assert_world_stable(&mut fixture, "5_nation_medium_run");

    // Verify no nations inexplicably vanished without factions taking their place
    // (In V0.6, nations only die if they collapse into factions)
    let final_nation_count = fixture.world.world.query::<&Nation>().iter(&fixture.world.world).count();
    assert!(
        final_nation_count >= initial_nation_count,
        "Nation count should not spontaneously decrease without factionalization"
    );
}

#[test]
fn test_10_nation_world_long_run_dynamics() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(800)
        .with_nations(10)
        .build();

    let initial_diplomacy_count = fixture.world.world.query::<&DiplomaticRelation>().iter(&fixture.world.world).count();

    // Run for a long duration
    let metrics = fixture.execute_ticks_timed(200);

    assert_eq!(fixture.current_tick(), 200);
    println!("10-nation 200-tick performance: {}", metrics.summary());

    // Verify large-scale stability
    assertions::assert_world_stable(&mut fixture, "10_nation_long_run");

    // In a 10-nation world over 200 ticks, we expect diplomatic relations to form naturally
    let final_diplomacy_count = fixture.world.world.query::<&DiplomaticRelation>().iter(&fixture.world.world).count();
    assert!(
        final_diplomacy_count >= initial_diplomacy_count,
        "Diplomatic network should grow or remain stable over time"
    );
}

#[test]
fn test_global_legitimacy_and_gdp_drift() {
    // Tests that global resources/metrics don't NaN out or explode
    let mut fixture = TestWorldBuilder::new()
        .with_seed(900)
        .with_nations(6)
        .build();

    fixture.execute_ticks(150);

    // Ensure all remaining nations have valid, bounded legitimacy and GDP
    let mut query = fixture.world.world.query::<(&Nation, &Legitimacy, &GDP)>();
    for (nation, legit, gdp) in query.iter(&fixture.world.world) {
        assert!(legit.value >= 0.0 && legit.value <= 100.0, "{} out of bounds legitimacy: {}", nation.name, legit.value);
        assert!(!gdp.value.is_nan(), "{} has NaN GDP", nation.name);
        assert!(gdp.value >= 0.0, "{} has negative GDP: {}", nation.name, gdp.value);
    }
    
    println!("✓ Global metrics valid for all nations after 150 ticks");
}
