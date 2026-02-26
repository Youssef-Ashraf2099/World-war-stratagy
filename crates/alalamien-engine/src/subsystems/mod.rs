//! Game subsystems
//!
//! Each subsystem implements a specific aspect of the simulation:
//! - Demographic: Population growth, death, migration
//! - Economic: Production, consumption, GDP
//! - Diplomatic: Relations, alliances
//! - Logistics: Resource movement, trade
//! - Military: Army upkeep, combat
//! - Stability: Revolts, legitimacy

pub mod demographic;
pub mod economic;

// To be implemented in future versions
// pub mod diplomatic;
// pub mod logistics;
// pub mod military;
// pub mod stability;

pub use demographic::DemographicPhase;
pub use economic::EconomicPhase;
