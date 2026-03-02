// Test module organization following SOLID principles
// 
// Structure:
// - Unit tests: Test individual components/systems
// - Integration tests: Test subsystem interactions  
// - Advanced tests: Complex scenarios validating game quality
// - Fixtures: Reusable test utilities and helpers

pub mod fixtures;
pub mod unit;
pub mod integration;
pub mod advanced;

// Re-export commonly used test fixtures
pub use fixtures::{TestWorldBuilder, TestWorldFixture, TickMetrics};
