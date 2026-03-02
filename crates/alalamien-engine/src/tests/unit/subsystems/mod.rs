/// Subsystem unit tests organized by subsystem type
/// 
/// Each subsystem gets its own test module to ensure:
/// - Changes don't break existing behavior
/// - New features don't create regressions
/// - Edge cases are properly tested

pub mod warfare;
pub mod diplomacy;
pub mod economic;
pub mod legitimacy;
pub mod alliance;
pub mod intervention;
pub mod events;
pub mod factions;
pub mod nuclear;
pub mod notifications;

// Import test fixtures for subsystem tests
pub use super::super::fixtures::{TestWorldBuilder, TestWorldFixture, assertions};
