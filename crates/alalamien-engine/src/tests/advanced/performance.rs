/// Performance benchmark tests
/// 
/// Measures:
/// - Tick execution time (target: <15ms/tick)
/// - Memory usage patterns
/// - Scaling characteristics (N nations → how much slower?)
/// - Bottleneck identification

use super::super::fixtures::TestWorldBuilder;

#[test]
fn test_performance_single_nation() {
    let mut world = TestWorldBuilder::new()
        .with_seed(1000)
        .with_nations(1)
        .build();

    let metrics = world.execute_ticks_timed(100);
    println!("1-nation, 100-tick: {}", metrics.summary());
    
    // Target: <15ms per tick
    assertions::assert_performance_acceptable(&metrics, 15.0, "single_nation_perf");
}

#[test]
fn test_performance_5_nations() {
    let mut world = TestWorldBuilder::new()
        .with_seed(1001)
        .with_nations(5)
        .build();

    let metrics = world.execute_ticks_timed(100);
    println!("5-nation, 100-tick: {}", metrics.summary());
    
    assertions::assert_performance_acceptable(&metrics, 20.0, "5_nations_perf");
}

#[test]
fn test_performance_10_nations() {
    let mut world = TestWorldBuilder::new()
        .with_seed(1002)
        .with_nations(10)
        .build();

    let metrics = world.execute_ticks_timed(100);
    println!("10-nation, 100-tick: {}", metrics.summary());
    
    // Slightly higher target for larger world
    assertions::assert_performance_acceptable(&metrics, 30.0, "10_nations_perf");
}

#[test]
fn test_performance_scaling_10_vs_5() {
    // Establish baseline
    let mut world5 = TestWorldBuilder::new()
        .with_seed(1003)
        .with_nations(5)
        .build();
    let metrics5 = world5.execute_ticks_timed(50);

    // Test larger world
    let mut world10 = TestWorldBuilder::new()
        .with_seed(1004)
        .with_nations(10)
        .build();
    let metrics10 = world10.execute_ticks_timed(50);

    // Performance should scale reasonably (not exponentially)
    let ratio = metrics10.ms_per_tick / metrics5.ms_per_tick;
    println!("10-nation vs 5-nation performance ratio: {:.2}x", ratio);
    
    // Should be < 3x slower (linear scaling would be 2x)
    assert!(
        ratio < 3.0,
        "Performance scaling too bad: 10 nations is {:.2}x slower than 5",
        ratio
    );
}

// Helper for performance assertions
mod assertions {
    use super::super::super::fixtures::TickMetrics;
    
    pub fn assert_performance_acceptable(
        metrics: &TickMetrics,
        max_ms_per_tick: f64,
        test_name: &str,
    ) {
        assert!(
            metrics.ms_per_tick <= max_ms_per_tick,
            "{}: Performance unacceptable: {:.2}ms/tick exceeds {:.2}ms/tick limit",
            test_name,
            metrics.ms_per_tick,
            max_ms_per_tick
        );
    }
}
