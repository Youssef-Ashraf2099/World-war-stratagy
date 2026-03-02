/// Advanced tests for game quality and functionality
/// 
/// Categories:
/// - Determinism validation (reproducibility)
/// - Performance benchmarks (efficiency)
/// - Edge cases (robustness)
/// - Quality metrics (game feel)
/// - Regression tests (no regressions)
/// - Chaos/Fuzz tests (random configurations)
/// - Subsystem performance (per-subsystem profiling)

pub mod determinism;
pub mod performance;
pub mod edge_cases;
pub mod quality_metrics;
pub mod regression;
pub mod chaos_fuzz;
pub mod subsystem_performance;

pub use super::fixtures::{
    TestWorldBuilder, TestWorldFixture, TickMetrics,
    DeterminismTestFixture, DeterminismRunResult, PerformanceSummary, assertions,
};
