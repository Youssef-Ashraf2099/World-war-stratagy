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
pub mod ai_advanced;
pub mod vassalage;

// V0.4 subsystems
pub mod alliance;
pub mod alliance_dataset;
pub mod diplomacy;

// V0.5 subsystems
pub mod legitimacy;

// V0.6 subsystems
pub mod factions;
pub mod events;
pub mod intervention;

// V0.7 subsystems
pub mod espionage;

// V0.8 subsystems
pub mod nuclear;

// V0.9 subsystems
pub mod notifications;

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
pub use ai_advanced::AdvancedAIDecisionPhase;
pub use vassalage::VassalagePhase;

// V0.4 exports
pub use alliance::AlliancePhase;
pub use diplomacy::DiplomacyPhase;

// V0.5 exports
pub use legitimacy::LegitimacyPhase;

// V0.6 exports
pub use factions::FactionCivilWarPhase;
pub use events::EventPhase;
pub use intervention::InterventionPhase;
// V0.7 exports
pub use espionage::EspionagePhase;

// V0.8 exports
pub use nuclear::NuclearPhase;

// V0.9 exports
pub use notifications::NotificationPhase;