/// Test fixtures and utilities for SOLID-aligned testing
/// 
/// This module provides reusable components for tests:
/// - TestWorldBuilder: Fluent API for setting up test worlds
/// - TestWorldFixture: Standard test world with known state
/// - TickMetrics: Utilities for collecting test data
/// - Helper functions: Common setup patterns

use crate::core::{WorldState, types::*};
use crate::core::tick::TickPipeline;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Builder for creating test worlds with specific configurations
pub struct TestWorldBuilder {
    seed: u64,
    nation_count: usize,
    nation_configs: Vec<(String, [u8; 3], bool)>,
}

impl TestWorldBuilder {
    /// Create a new test world builder with default seed
    pub fn new() -> Self {
        Self {
            seed: 42,
            nation_count: 1,
            nation_configs: vec![],
        }
    }

    /// Set the random seed for deterministic tests
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Add a nation to the world
    pub fn add_nation(mut self, name: String, color: [u8; 3], player_controlled: bool) -> Self {
        self.nation_configs.push((name, color, player_controlled));
        self.nation_count += 1;
        self
    }

    /// Add multiple nations with default coloring
    pub fn with_nations(mut self, count: usize) -> Self {
        self.nation_count = count;
        for i in 0..count {
            let color = [
                (50 + i * 10) as u8,
                (100 + i * 5) as u8,
                (150 - i as u8 * 3),
            ];
            self.nation_configs.push((
                format!("TestNation_{}", i),
                color,
                i == 0,
            ));
        }
        self
    }

    /// Build the test world
    pub fn build(self) -> TestWorldFixture {
        let mut world_state = WorldState::new(self.seed);
        let mut nation_ids = Vec::new();

        for (name, color, player_controlled) in self.nation_configs {
            let entity = world_state.spawn_nation(name, color, player_controlled);
            nation_ids.push(entity);
        }

        TestWorldFixture {
            world: world_state,
            nation_entities: nation_ids,
            seed: self.seed,
            created_at: Instant::now(),
        }
    }
}

impl Default for TestWorldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard test world fixture with known state
pub struct TestWorldFixture {
    pub world: WorldState,
    pub nation_entities: Vec<bevy_ecs::entity::Entity>,
    pub seed: u64,
    pub created_at: Instant,
}

impl TestWorldFixture {
    /// Get elapsed time since fixture creation
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Execute N ticks with the V0.6 pipeline
    pub fn execute_ticks(&mut self, count: u64) {
        let mut pipeline = TickPipeline::new_v0_6();
        pipeline.execute_many(&mut self.world, count);
    }

    /// Execute with timing information
    pub fn execute_ticks_timed(&mut self, count: u64) -> TickMetrics {
        let start = Instant::now();
        self.execute_ticks(count);
        let elapsed = start.elapsed();

        TickMetrics {
            ticks_executed: count,
            elapsed,
            ms_per_tick: elapsed.as_secs_f64() * 1000.0 / count as f64,
        }
    }

    /// Get current world tick
    pub fn current_tick(&self) -> u64 {
        self.world.current_tick()
    }

    /// Get nation count
    pub fn nation_count(&self) -> usize {
        self.nation_entities.len()
    }

    /// Count entities of a specific component type (via query)
    pub fn count_component<T: bevy_ecs::component::Component>(&mut self) -> usize {
        self.world.world.query::<&T>().iter(&self.world.world).count()
    }
}

/// Metrics collected during tick execution
#[derive(Debug, Clone)]
pub struct TickMetrics {
    pub ticks_executed: u64,
    pub elapsed: Duration,
    pub ms_per_tick: f64,
}

impl TickMetrics {
    /// Check if performance is within expected range
    pub fn is_performance_acceptable(&self, max_ms_per_tick: f64) -> bool {
        self.ms_per_tick <= max_ms_per_tick
    }

    /// Format as readable string
    pub fn summary(&self) -> String {
        format!(
            "{} ticks in {:.2}s ({:.2} ms/tick)",
            self.ticks_executed,
            self.elapsed.as_secs_f64(),
            self.ms_per_tick
        )
    }
}

/// Fixture for determinism validation tests
pub struct DeterminismTestFixture {
    pub seed: u64,
    pub iterations: usize,
    pub run_results: Vec<DeterminismRunResult>,
}

/// Result from a single determinism test run
pub struct DeterminismRunResult {
    pub run_number: usize,
    pub tick_count: u64,
    pub checkpoints: Vec<(u64, usize, usize)>, // (tick, nation_count, province_count)
    pub execution_time: Duration,
}

impl DeterminismTestFixture {
    /// Create a new determinism test fixture
    pub fn new(seed: u64, iterations: usize) -> Self {
        Self {
            seed,
            iterations,
            run_results: Vec::new(),
        }
    }

    /// Add a run result
    pub fn add_run(&mut self, result: DeterminismRunResult) {
        self.run_results.push(result);
    }

    /// Check if all runs are identical (deterministic)
    pub fn is_deterministic(&self) -> bool {
        if self.run_results.is_empty() {
            return true;
        }

        let first = &self.run_results[0];
        self.run_results.iter().skip(1).all(|run| {
            run.checkpoints == first.checkpoints
        })
    }

    /// Get performance summary
    pub fn performance_summary(&self) -> PerformanceSummary {
        let total_time: Duration = self.run_results.iter()
            .map(|r| r.execution_time)
            .sum();
        let avg_time = total_time / self.run_results.len().max(1) as u32;
        let total_ticks = self.run_results.iter()
            .map(|r| r.tick_count)
            .sum::<u64>();

        PerformanceSummary {
            total_runs: self.run_results.len(),
            total_execution_time: total_time,
            average_per_run: avg_time,
            total_ticks_processed: total_ticks,
            ms_per_tick_average: total_time.as_secs_f64() * 1000.0 / total_ticks.max(1) as f64,
        }
    }
}

/// Performance metrics summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_runs: usize,
    pub total_execution_time: Duration,
    pub average_per_run: Duration,
    pub total_ticks_processed: u64,
    pub ms_per_tick_average: f64,
}

impl PerformanceSummary {
    /// Format as readable string
    pub fn display(&self) -> String {
        format!(
            "Runs: {}, Total: {:.2}s, Avg/Run: {:.2}s, {}ms/tick",
            self.total_runs,
            self.total_execution_time.as_secs_f64(),
            self.average_per_run.as_secs_f64(),
            self.ms_per_tick_average as u64
        )
    }
}

/// Common test assertions
pub mod assertions {
    use super::*;

    /// Assert world state is stable (no NaN, no overflow)
    pub fn assert_world_stable(fixture: &mut TestWorldFixture, test_name: &str) {
        let mut query_pop = fixture.world.world.query::<&crate::core::types::Population>();
        let mut query_res = fixture.world.world.query::<&crate::core::types::Resources>();

        for pop in query_pop.iter(&fixture.world.world) {
            assert!(
                pop.total > 0,
                "{}: Population became zero or negative",
                test_name
            );
            assert!(
                pop.total < 1_000_000_000_000,
                "{}: Population overflow detected: {}",
                test_name,
                pop.total
            );
        }

        for res in query_res.iter(&fixture.world.world) {
            assert!(
                !res.food.is_nan(),
                "{}: Food resource became NaN",
                test_name
            );
            assert!(
                !res.iron.is_nan(),
                "{}: Iron resource became NaN",
                test_name
            );
            assert!(
                !res.oil.is_nan(),
                "{}: Oil resource became NaN",
                test_name
            );
        }
    }

    /// Assert determinism between two sets of checkpoints
    pub fn assert_deterministic_checkpoints(
        run1: &[(u64, usize, usize)],
        run2: &[(u64, usize, usize)],
        test_name: &str,
    ) {
        assert_eq!(
            run1.len(),
            run2.len(),
            "{}: Checkpoint count mismatch",
            test_name
        );

        for (i, ((_, n1, p1), (_t2, n2, p2))) in run1.iter().zip(run2.iter()).enumerate() {
            assert_eq!(
                n1, n2,
                "{}: Nation count divergence at checkpoint {}: {} vs {}",
                test_name, i, n1, n2
            );
            assert_eq!(
                p1, p2,
                "{}: Province count divergence at checkpoint {}: {} vs {}",
                test_name, i, p1, p2
            );
        }
    }

    /// Assert performance is acceptable
    pub fn assert_performance_acceptable(
        metrics: &TickMetrics,
        max_ms_per_tick: f64,
        test_name: &str,
    ) {
        assert!(
            metrics.is_performance_acceptable(max_ms_per_tick),
            "{}: Performance unacceptable: {:.2}ms/tick exceeds {:.2}ms/tick limit",
            test_name,
            metrics.ms_per_tick,
            max_ms_per_tick
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creates_world() {
        let fixture = TestWorldBuilder::new()
            .with_seed(123)
            .with_nations(5)
            .build();

        assert_eq!(fixture.seed, 123);
        assert_eq!(fixture.nation_count(), 5);
    }

    #[test]
    fn test_fixture_tick_execution() {
        let mut fixture = TestWorldBuilder::new()
            .with_nations(3)
            .build();

        let initial_tick = fixture.current_tick();
        fixture.execute_ticks(10);
        assert_eq!(fixture.current_tick(), initial_tick + 10);
    }

    #[test]
    fn test_performance_metrics() {
        let mut fixture = TestWorldBuilder::new()
            .with_nations(2)
            .build();

        let metrics = fixture.execute_ticks_timed(5);
        assert_eq!(metrics.ticks_executed, 5);
        assert!(metrics.elapsed.as_millis() >= 0);
        assert!(metrics.ms_per_tick <= 100.0); // Assume < 100ms per tick
    }

    #[test]
    fn test_determinism_fixture() {
        let fixture = DeterminismTestFixture::new(42, 3);
        assert_eq!(fixture.seed, 42);
        assert_eq!(fixture.iterations, 3);
        assert!(fixture.is_deterministic()); // Empty is deterministic
    }
}
