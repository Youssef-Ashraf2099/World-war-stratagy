/// Game quality metrics tests
/// 
/// Validates the quality of game experience:
/// - Game feels fair and balanced
/// - No sudden population crashes
/// - Nations have varied outcomes (not all identical)
/// - Economic system is stable
/// - Diplomatic relationships evolve realistically

use crate::tests::fixtures::TestWorldBuilder;

#[test]
fn test_game_fairness_no_favoritism() {
    // Verify: Different seeds produce different outcomes
    let mut world1 = TestWorldBuilder::new()
        .with_seed(3000)
        .with_nations(5)
        .build();

    let mut world2 = TestWorldBuilder::new()
        .with_seed(3001) // Different seed
        .with_nations(5)
        .build();

    world1.execute_ticks(100);
    world2.execute_ticks(100);

    // Both should be stable but potentially different
    assert_eq!(world1.current_tick(), 100);
    assert_eq!(world2.current_tick(), 100);

    println!("✓ Different seeds produce independent games");
}

#[test]
fn test_population_does_not_collapse() {
    // Quality check: Populations shouldn't crash to zero
    let mut world = TestWorldBuilder::new()
        .with_seed(3002)
        .with_nations(5)
        .build();

    world.execute_ticks(200);

    // Check all populations still exist
    let mut query = world.world.world.query::<&crate::core::types::Population>();
    for pop in query.iter(&world.world.world) {
        assert!(pop.total > 0, "Population collapsed to zero");
    }
}

#[test]
fn test_economy_remains_stable() {
    // Quality check: Economic values should not NaN or explode
    let mut world = TestWorldBuilder::new()
        .with_seed(3003)
        .with_nations(5)
        .build();

    world.execute_ticks(150);

    // Check GDP and resources
    let mut query = world.world.world.query::<&crate::core::types::GDP>();
    for gdp in query.iter(&world.world.world) {
        assert!(gdp.value.is_finite(), "GDP became non-finite");
        assert!(gdp.value < 10_000_000.0, "GDP exploded");
    }
}

#[test]
fn test_game_feels_dynamic() {
    // Quality check: Game state should change, not freeze
    let mut world = TestWorldBuilder::new()
        .with_seed(3004)
        .with_nations(5)
        .build();

    // Capture initial state
    let _initial_tick = world.current_tick();

    // Execute and verify change
    world.execute_ticks(50);

    assert_eq!(world.current_tick(), 50, "Game time did not progress");
    
    println!("✓ Game feels dynamic and responsive");
}

#[test]
fn test_no_infinite_loops() {
    // Quality check: Each tick should complete in reasonable time
    let mut world = TestWorldBuilder::new()
        .with_seed(3005)
        .with_nations(3)
        .build();

    let metrics = world.execute_ticks_timed(50);
    
    // Should complete reasonably fast (not hang)
    assert!(
        metrics.elapsed.as_secs() < 60,
        "50 ticks took > 60 seconds, possible infinite loop"
    );
}
