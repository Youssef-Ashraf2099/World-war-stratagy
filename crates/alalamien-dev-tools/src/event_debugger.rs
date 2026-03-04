//! Event debugger tool
//! 
//! Filter and view notifications from subsystems in real-time.

pub struct EventDebugger {
    pub filter_subsystem: Option<String>,
    pub recent_events: Vec<String>,
}
