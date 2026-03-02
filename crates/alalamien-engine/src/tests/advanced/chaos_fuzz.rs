// Chaos and fuzz tests for robustness validation
// Tests with random/edge configurations to catch unexpected behavior

use crate::tests::fixtures::{TestWorldBuilder, assertions};

#[test]
fn test_random_nation_counts_stability() {
    // Test that worlds with varying nation counts remain stable
    for nation_count in &[1, 3, 7, 15, 25] {
        let mut fixture = TestWorldBuilder::new()
            .with_seed(42)
            .with_nations(*nation_count)
            .build();
        
        fixture.execute_ticks(100);
        assert_eq!(fixture.current_tick(), 100, "Failed with {} nations", nation_count);
    }
}

#[test]
fn test_random_seed_variance() {
    // Different seeds should produce different game states
    let mut results: Vec<u64> = Vec::new();
    
    for seed in &[1, 42, 100, 999, 12345] {
        let mut fixture = TestWorldBuilder::new()
            .with_seed(*seed)
            .with_nations(5)
            .build();
        
        fixture.execute_ticks(50);
        results.push(fixture.current_tick());
    }
    
    // All should reach the same tick count
    for tick in &results {
        assert_eq!(*tick, 50);
    }
}

#[test]
fn test_rapid_fire_ticks() {
    // Execute many ticks rapidly to catch off-by-one errors
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    for i in 1..=50 {
        fixture.execute_ticks(1);
        assert_eq!(fixture.current_tick(), i, "Tick count mismatch at iteration {}", i);
    }
}

#[test]
fn test_batch_tick_equivalence() {
    // Executing 100 ticks at once should equal doing 10 batches of 10
    let mut fixture_batch = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    let mut fixture_single = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    // Batch approach: 10 ticks × 10 times
    for _ in 0..10 {
        fixture_batch.execute_ticks(10);
    }
    
    // Single approach: 100 ticks at once
    fixture_single.execute_ticks(100);
    
    // Both should be at tick 100
    assert_eq!(fixture_batch.current_tick(), 100);
    assert_eq!(fixture_single.current_tick(), 100);
}

#[test]
fn test_multi_seed_comparison() {
    // Verify that the same seed produces identical results
    let mut fixture_run1 = TestWorldBuilder::new()
        .with_seed(999)
        .with_nations(3)
        .build();
    
    let mut fixture_run2 = TestWorldBuilder::new()
        .with_seed(999)
        .with_nations(3)
        .build();
    
    for _ in 0..50 {
        fixture_run1.execute_ticks(2);
        fixture_run2.execute_ticks(2);
        
        assert_eq!(fixture_run1.current_tick(), fixture_run2.current_tick());
    }
}

#[test]
fn test_stress_test_many_nations() {
    // Stress test: larger world
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(50)
        .build();
    
    let metrics = fixture.execute_ticks_timed(100);
    
    assert_eq!(fixture.current_tick(), 100);
    assert!(metrics.ms_per_tick < 50.0, "Performance degradation with many nations");
}

#[test]
fn test_world_doesnt_crash_on_zero_resources() {
    // Extreme edge case: worlds should remain stable even with resource stress
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    fixture.execute_ticks(500);
    
    // Should complete without panic
    assert_eq!(fixture.current_tick(), 500);
    assertions::assert_world_stable(&mut fixture, "zero_resource_stress");
}

#[test]
fn test_sequential_execution_stability() {
    // Execute ticks in different batch sizes, verify consistency
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    fixture.execute_ticks(25);
    assert_eq!(fixture.current_tick(), 25);
    
    fixture.execute_ticks(25);
    assert_eq!(fixture.current_tick(), 50);
    
    fixture.execute_ticks(50);
    assert_eq!(fixture.current_tick(), 100);
    
    // Total should be 100
    fixture.execute_ticks(100);
    assert_eq!(fixture.current_tick(), 200);
}

#[test]
fn test_extreme_nation_counts() {
    // Edge case: very small and very large worlds
    
    // Very small
    let mut tiny = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(1)
        .build();
    tiny.execute_ticks(50);
    assert_eq!(tiny.current_tick(), 50);
    
    // Medium
    let mut medium = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(20)
        .build();
    medium.execute_ticks(50);
    assert_eq!(medium.current_tick(), 50);
}
