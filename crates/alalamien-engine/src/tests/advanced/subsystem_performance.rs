// Subsystem-specific performance profiling
// Measures time spent in individual subsystems under various conditions

use crate::tests::fixtures::{TestWorldBuilder, assertions};

#[test]
fn test_diplomacy_subsystem_performance() {
    // Measure diplomacy system under different world sizes
    for nation_count in &[5, 15, 30] {
        let mut fixture = TestWorldBuilder::new()
            .with_seed(42)
            .with_nations(*nation_count)
            .build();
        
        let metrics = fixture.execute_ticks_timed(100);
        
        println!(
            "Diplomacy: {} nations in {}ms ({:.2}ms/tick)",
            nation_count,
            metrics.elapsed.as_millis(),
            metrics.ms_per_tick
        );
        
        // Reasonable bounds for diplomatic calculations
        assert!(metrics.ms_per_tick < 40.0, "Diplomacy perf regression with {} nations", nation_count);
    }
}

#[test]
fn test_legitimacy_subsystem_performance() {
    // Stress legitimacy calculations
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(10)
        .build();
    
    let metrics = fixture.execute_ticks_timed(500);
    
    println!(
        "Legitimacy: 500 ticks in {}ms ({:.2}ms/tick)",
        metrics.elapsed.as_millis(),
        metrics.ms_per_tick
    );
    
    assert!(metrics.ms_per_tick < 30.0, "Legitimacy calculation too slow");
}

#[test]
fn test_warfare_subsystem_performance() {
    // Test warfare overhead
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    
    // Run extended session
    let metrics = fixture.execute_ticks_timed(300);
    
    println!(
        "Warfare: 300 ticks in {}ms ({:.2}ms/tick)",
        metrics.elapsed.as_millis(),
        metrics.ms_per_tick
    );
    
    assert!(metrics.ms_per_tick < 35.0, "Warfare perf regression");
}

#[test]
fn test_economic_subsystem_performance() {
    // Economic calculations can be expensive
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(20)
        .build();
    
    let metrics = fixture.execute_ticks_timed(200);
    
    println!(
        "Economic: 200 ticks × 20 nations in {}ms ({:.2}ms/tick)",
        metrics.elapsed.as_millis(),
        metrics.ms_per_tick
    );
    
    assert!(metrics.ms_per_tick < 50.0, "Economic perf exceeds acceptable");
}

#[test]
fn test_alliance_subsystem_performance() {
    // Alliance management with many alliances
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(15)
        .build();
    
    let metrics = fixture.execute_ticks_timed(150);
    
    println!(
        "Alliance: 150 ticks × 15 nations in {}ms ({:.2}ms/tick)",
        metrics.elapsed.as_millis(),
        metrics.ms_per_tick
    );
    
    assert!(metrics.ms_per_tick < 30.0, "Alliance management too slow");
}

#[test]
fn test_events_subsystem_performance() {
    // Event generation and processing
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(10)
        .build();
    
    let metrics = fixture.execute_ticks_timed(400);
    
    println!(
        "Events: 400 ticks × 10 nations in {}ms ({:.2}ms/tick)",
        metrics.elapsed.as_millis(),
        metrics.ms_per_tick
    );
    
    assert!(metrics.ms_per_tick < 25.0, "Event perf regression");
}

#[test]
fn test_combined_subsystem_scaling() {
    // Test how performance scales with combined complexity
    let metrics_1_nat = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(1)
        .build()
        .execute_ticks_timed(100);
    
    let metrics_10_nat = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(10)
        .build()
        .execute_ticks_timed(100);
    
    let metrics_30_nat = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(30)
        .build()
        .execute_ticks_timed(100);
    
    println!(
        "Scaling: 1-nation: {:.2}ms/tick, 10-nation: {:.2}ms/tick, 30-nation: {:.2}ms/tick",
        metrics_1_nat.ms_per_tick,
        metrics_10_nat.ms_per_tick,
        metrics_30_nat.ms_per_tick
    );
    
    // Ensure scaling is reasonable (not exponential)
    let ratio_10_to_1 = metrics_10_nat.ms_per_tick / (metrics_1_nat.ms_per_tick + 0.001);
    assert!(ratio_10_to_1 < 20.0, "Scaling is worse than expected");
}
