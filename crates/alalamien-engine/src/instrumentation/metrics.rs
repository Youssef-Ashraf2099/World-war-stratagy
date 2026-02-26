//! Metrics collection for performance monitoring

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Thread-safe metrics registry
#[derive(Clone)]
pub struct MetricsRegistry {
    inner: Arc<RwLock<MetricsInner>>,
}

struct MetricsInner {
    counters: HashMap<String, u64>,
    timings: HashMap<String, Vec<Duration>>,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetricsInner {
                counters: HashMap::new(),
                timings: HashMap::new(),
            })),
        }
    }

    /// Increment a counter
    pub fn increment_counter(&self, name: &str) {
        let mut inner = self.inner.write().unwrap();
        *inner.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    /// Record a timing measurement
    pub fn record_timing(&self, name: &str, duration: Duration) {
        let mut inner = self.inner.write().unwrap();
        inner
            .timings
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> Option<u64> {
        let inner = self.inner.read().unwrap();
        inner.counters.get(name).copied()
    }

    /// Get average timing for a metric
    pub fn get_average_timing(&self, name: &str) -> Option<Duration> {
        let inner = self.inner.read().unwrap();
        inner.timings.get(name).and_then(|timings| {
            if timings.is_empty() {
                None
            } else {
                let sum: Duration = timings.iter().sum();
                Some(sum / timings.len() as u32)
            }
        })
    }

    /// Clear all metrics
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.counters.clear();
        inner.timings.clear();
    }

    /// Get snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let inner = self.inner.read().unwrap();
        MetricsSnapshot {
            counters: inner.counters.clone(),
            average_timings: inner
                .timings
                .iter()
                .filter_map(|(k, v)| {
                    if v.is_empty() {
                        None
                    } else {
                        let sum: Duration = v.iter().sum();
                        Some((k.clone(), sum / v.len() as u32))
                    }
                })
                .collect(),
        }
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub average_timings: HashMap<String, Duration>,
}

/// Helper struct for timing scoped operations
pub struct TimingGuard {
    name: String,
    registry: MetricsRegistry,
    start: Instant,
}

impl TimingGuard {
    pub fn new(name: String, registry: MetricsRegistry) -> Self {
        Self {
            name,
            registry,
            start: Instant::now(),
        }
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        self.registry.record_timing(&self.name, duration);
    }
}

/// Helper macro for timing a scope
#[macro_export]
macro_rules! time_scope {
    ($metrics:expr, $name:expr) => {
        let _timing_guard = $crate::instrumentation::metrics::TimingGuard::new(
            $name.to_string(),
            $metrics.clone(),
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_counter() {
        let metrics = MetricsRegistry::new();
        
        metrics.increment_counter("test");
        metrics.increment_counter("test");
        metrics.increment_counter("test");

        assert_eq!(metrics.get_counter("test"), Some(3));
    }

    #[test]
    fn test_timing() {
        let metrics = MetricsRegistry::new();
        
        metrics.record_timing("test", Duration::from_millis(100));
        metrics.record_timing("test", Duration::from_millis(200));

        let avg = metrics.get_average_timing("test").unwrap();
        assert_eq!(avg, Duration::from_millis(150));
    }

    #[test]
    fn test_timing_guard() {
        let metrics = MetricsRegistry::new();
        
        {
            let _guard = TimingGuard::new("test".to_string(), metrics.clone());
            sleep(Duration::from_millis(10));
        }

        let avg = metrics.get_average_timing("test").unwrap();
        assert!(avg >= Duration::from_millis(10));
    }
}
