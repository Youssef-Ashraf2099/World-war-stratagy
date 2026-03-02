/// Unit tests organized by component following SOLID principles
/// 
/// Single Responsibility: Each module tests one component/system
/// Open/Closed: Easy to extend with new test cases
/// Liskov Substitution: Consistent test patterns across modules
/// Interface Segregation: Test modules import only what they need
/// Dependency Inversion: Tests depend on public interfaces

pub mod core_types;
pub mod core_world;
pub mod core_tick;
pub mod subsystems;

pub use super::fixtures::{TestWorldBuilder, TestWorldFixture, TickMetrics};
