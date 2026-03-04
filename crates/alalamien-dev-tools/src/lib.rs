//! Alalamien Developer Tools
//! 
//! Feature-gated tools for debugging, profiling, and development.
//! Add `--features dev-tools` to enable.

pub mod performance_monitor;
pub mod state_inspector;
pub mod event_debugger;
pub mod scenario_editor;

/// Initialize all developer tools systems
pub fn init_dev_tools() {
    #[cfg(feature = "dev-tools")]
    {
        println!("🛠️  Developer Tools Initialized");
    }
}
