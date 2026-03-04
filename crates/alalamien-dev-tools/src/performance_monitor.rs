//! Performance monitoring tool
//! 
//! Displays FPS, memory usage, and subsystem tick times.

pub struct PerformanceMonitor {
    pub fps: f32,
    pub memory_usage_mb: f32,
    pub tick_time_ms: f32,
}
