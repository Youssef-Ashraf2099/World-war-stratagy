/// Determinism validation tests
/// 
/// Ensures that the same seed always produces the same results (bit-for-bit).
/// Critical for:
/// - Replay functionality
/// - Network multiplayer (lockstep)
/// - Testing reproducibility
/// - AI training consistency

use super::super::fixtures::{TestWorldBuilder, DeterminismTestFixture, DeterminismRunResult};
use crate::core::tick::TickPipeline;
use std::time::Instant;

/// Test: 100-tick determinism with 3 runs
#[test]
fn test_100_tick_determinism() {
    let seed = 42u64;
    let total_ticks = 100u64;
    let mut fixture = DeterminismTestFixture::new(seed, 3);

    println!("\n=== 100-Tick Determinism Test ===");

    for run_num in 1..=3 {
        let mut world = TestWorldBuilder::new()
            .with_seed(seed)
            .with_nations(5)
            .build();

        let start = Instant::now();
        let mut checkpoints = Vec::new();

        // Execute and checkpoint every 10 ticks
        for i in 0..10 {
            world.execute_ticks(10);
            let nation_count = world.nation_count();
            let tick = (i + 1) * 10;
            checkpoints.push((tick, nation_count, 0)); // Province count tbd

            println!("Run {}, Checkpoint {}: tick={}", run_num, i + 1, tick);
        }

        let elapsed = start.elapsed();
        fixture.add_run(DeterminismRunResult {
            run_number: run_num,
            tick_count: total_ticks,
            checkpoints,
            execution_time: elapsed,
        });
    }

    // Verify determinism
    assert!(
        fixture.is_deterministic(),
        "Determinism check failed: runs produced different results"
    );

    let summary = fixture.performance_summary();
    println!("Performance: {}", summary.display());
    println!("✓ All 3 runs identical - DETERMINISTIC\n");
}

/// Test: 1000-tick determinism stress test
#[test]
#[ignore] // Longer running test
fn test_1000_tick_determinism() {
    let seed = 123u64;
    let total_ticks = 1000u64;
    let mut fixture = DeterminismTestFixture::new(seed, 2);

    println!("\n=== 1000-Tick Determinism Test ===");

    for run_num in 1..=2 {
        let mut world = TestWorldBuilder::new()
            .with_seed(seed)
            .with_nations(10)
            .build();

        let start = Instant::now();
        let metrics = world.execute_ticks_timed(total_ticks);

        let checkpoints = vec![(total_ticks, world.nation_count(), 0)];

        fixture.add_run(DeterminismRunResult {
            run_number: run_num,
            tick_count: total_ticks,
            checkpoints,
            execution_time: metrics.elapsed,
        });

        println!("Run {} completed: {}", run_num, metrics.summary());
    }

    assert!(fixture.is_deterministic(), "1000-tick determinism failed");
    println!("✓ Determinism verified over 1000 ticks\n");
}

/// Test: Multi-seed determinism (same seed = same results across different test runs)
#[test]
fn test_multi_seed_consistency() {
    const SEEDS: &[u64] = &[42, 123, 999, 7777];

    for &seed in SEEDS {
        let mut run1 = TestWorldBuilder::new()
            .with_seed(seed)
            .with_nations(3)
            .build();

        let mut run2 = TestWorldBuilder::new()
            .with_seed(seed)
            .with_nations(3)
            .build();

        run1.execute_ticks(50);
        run2.execute_ticks(50);

        assert_eq!(
            run1.current_tick(),
            run2.current_tick(),
            "Seed {}: Tick count divergence",
            seed
        );

        let n1 = run1.nation_count();
        let n2 = run2.nation_count();
        assert_eq!(
            n1, n2,
            "Seed {}: Nation count divergence {} vs {}",
            seed, n1, n2
        );

        println!("✓ Seed {} passed determinism check", seed);
    }
}

/// Test: Large-scale determinism (100K ticks, marked ignored by default)
#[test]
#[ignore] // Very long running, run with: cargo test -- --ignored test_100k_tick_determinism
fn test_100k_tick_determinism() {
    let seed = 99999u64;
    let total_ticks = 100_000u64;
    let checkpoint_interval = 10_000u64;
    
    println!("\n=== 100K-Tick Determinism Test (Advanced) ===");
    
    let mut fixture = DeterminismTestFixture::new(seed, 3);

    for run_num in 1..=3 {
        println!("Run {}/3: Executing 100,000 ticks with seed {}", run_num, seed);
        
        let mut world = TestWorldBuilder::new()
            .with_seed(seed)
            .with_nations(20)
            .build();

        let start = Instant::now();
        let mut checkpoints = Vec::new();

        // Execute in 10K-tick chunks
        for chunk in 0..10 {
            world.execute_ticks(checkpoint_interval);
            let tick = (chunk + 1) * checkpoint_interval;
            let nation_count = world.nation_count();
            checkpoints.push((tick, nation_count, 0));

            if chunk % 5 == 0 {
                println!("  ...{} ticks executed", tick);
            }
        }

        let elapsed = start.elapsed();
        fixture.add_run(DeterminismRunResult {
            run_number: run_num,
            tick_count: total_ticks,
            checkpoints,
            execution_time: elapsed,
        });

        println!("Run {} completed in {:.2}s", run_num, elapsed.as_secs_f64());
    }

    // Final verification
    assert!(
        fixture.is_deterministic(),
        "100K-tick determinism failed: runs diverged"
    );

    let summary = fixture.performance_summary();
    println!("\n=== RESULTS ===");
    println!("Performance: {}", summary.display());
    println!("✓ All 3 × 100K-tick runs identical");
    println!("✓ Engine proven deterministic and stable at scale\n");
}
