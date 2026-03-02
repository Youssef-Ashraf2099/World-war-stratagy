/// Integration tests for subsystem interactions
/// 
/// Open/Closed Principle: Easy to add new integration scenarios
/// Liskov Substitution: All tests follow same pattern
/// Interface Segregation: Each test focuses on specific interaction
/// 
/// Tests cover:
/// - Multiple subsystems working together
/// - Complex game scenarios
/// - State consistency across multi-phase execution

pub mod warfare_diplomacy;
pub mod economic_military;
pub mod alliance_warfare;
pub mod multi_nation_scenarios;
pub mod economic_scenarios;
pub mod diplomatic_scenarios;
pub mod game_depth_scenarios;
pub mod civil_war_cascade;
pub mod espionage_scenario;

pub use super::fixtures::{TestWorldBuilder, TestWorldFixture, assertions};
