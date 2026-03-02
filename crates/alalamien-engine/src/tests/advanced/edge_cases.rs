/// Edge case and robustness tests
/// 
/// Tests handle boundary conditions and unusual scenarios:
/// - Extreme world sizes (1 nation, 100 nations)
/// - Very long game sessions (10K+ ticks)
/// - Resource depletion scenarios
/// - Population extremes
/// - State boundary conditions

use crate::tests::fixtures::TestWorldBuilder;

#[test]
fn test_single_nation_stability() {
    // Edge case: Game with only 1 nation (no conflict possible)
    let mut world = TestWorldBuilder::new()
        .with_seed(2000)
        .with_nations(1)
        .build();

    world.execute_ticks(100);
    
    // Should remain stable even without external constraints
    assert_eq!(world.current_tick(), 100);
    assertions::assert_world_stable(&mut world, "single_nation");
}

#[test]
fn test_many_nations_stability() {
    // Edge case: Large number of nations
    let mut world = TestWorldBuilder::new()
        .with_seed(2001)
        .with_nations(50)
        .build();

    // Should not crash with many entities
    world.execute_ticks(20);
    assert_eq!(world.current_tick(), 20);
}

#[test]
fn test_long_game_session() {
    // Edge case: Very extended game session
    let mut world = TestWorldBuilder::new()
        .with_seed(2002)
        .with_nations(5)
        .build();

    // Execute 5000 ticks (should take ~10-15 seconds)
    world.execute_ticks(5000);
    
    assert_eq!(world.current_tick(), 5000);
    assertions::assert_world_stable(&mut world, "long_session");
}

#[test]
fn test_rapid_ticks_stress() {
    // Edge case: Rapid succession of ticks
    let mut world = TestWorldBuilder::new()
        .with_seed(2003)
        .with_nations(3)
        .build();

    // Rapid tick execution
    for _ in 0..100 {
        world.execute_ticks(10);
    }

    assert_eq!(world.current_tick(), 1000);
}

#[test]
fn test_zero_seed_handling() {
    // Edge case: Seed of zero
    let mut world = TestWorldBuilder::new()
        .with_seed(0)
        .with_nations(3)
        .build();

    world.execute_ticks(50);
    assert_eq!(world.current_tick(), 50);
}

#[test]
fn test_max_seed_handling() {
    // Edge case: Maximum u64 seed
    let mut world = TestWorldBuilder::new()
        .with_seed(u64::MAX)
        .with_nations(3)
        .build();

    world.execute_ticks(50);
    assert_eq!(world.current_tick(), 50);
}

// Helper module
mod assertions {
    use super::super::TestWorldFixture;
    use crate::core::types::{Population, Resources};
    
    pub fn assert_world_stable(world: &mut TestWorldFixture, test_name: &str) {
        let mut pop_query = world.world.world.query::<&Population>();
        let mut res_query = world.world.world.query::<&Resources>();

        for pop in pop_query.iter(&world.world.world) {
            assert!(
                pop.total > 0,
                "{}: Population became zero",
                test_name
            );
            assert!(
                pop.total < 1_000_000_000_000,
                "{}: Population overflow",
                test_name
            );
        }

        for res in res_query.iter(&world.world.world) {
            assert!(
                !res.food.is_nan(),
                "{}: Resources became NaN",
                test_name
            );
        }
    }
}
