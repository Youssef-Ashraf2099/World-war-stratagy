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
pub mod trade;
pub mod logistics;
pub mod stability;

// V0.3 subsystems
pub mod combat;
pub mod warfare;
pub mod occupation;
pub mod ai_basic;
pub mod vassalage;

pub use demographic::DemographicPhase;
pub use economic::EconomicPhase;
pub use trade::TradePhase;
pub use logistics::LogisticsPhase;
pub use stability::StabilityPhase;

// V0.3 exports
pub use combat::CombatPhase;
pub use warfare::WarfarePhase;
pub use occupation::OccupationPhase;
pub use ai_basic::AIDecisionPhase;
pub use vassalage::VassalagePhase;
