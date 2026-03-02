//! Deterministic random number generation
//!
//! Ensures reproducible simulation runs from the same seed.

use bevy_ecs::system::Resource;
use std::sync::atomic::{AtomicU64, Ordering};

/// Deterministic PRNG using xorshift algorithm
#[derive(Resource)]
pub struct DeterministicRng {
    state: AtomicU64,
}

impl DeterministicRng {
    /// Create a new RNG with a seed
    pub fn new(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(if seed == 0 { 1 } else { seed }),
        }
    }

    /// Generate next random u64
    pub fn next_u64(&self) -> u64 {
        let mut x = self.state.load(Ordering::SeqCst);
        loop {
            let new_x = {
                let mut val = x;
                val ^= val << 13;
                val ^= val >> 7;
                val ^= val << 17;
                val
            };
            match self.state.compare_exchange(x, new_x, Ordering::SeqCst, Ordering::SeqCst) {
                Ok(_) => return new_x,
                Err(cur) => x = cur,
            }
        }
    }

    /// Generate random u32
    pub fn next_u32(&self) -> u32 {
        (self.next_u64() & 0xFFFFFFFF) as u32
    }

    /// Generate random usize in range [0, bound)
    pub fn next_usize(&self, bound: usize) -> usize {
        (self.next_u64() % bound as u64) as usize
    }

    /// Generate random f64 in range [0.0, 1.0)
    pub fn next_f64(&self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }

    /// Generate random f64 in range [min, max)
    pub fn next_range(&self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }

    /// Generate random bool with given probability
    pub fn next_bool(&self, probability: f64) -> bool {
        self.next_f64() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_output() {
        let rng1 = DeterministicRng::new(42);
        let rng2 = DeterministicRng::new(42);

        // Same seed should produce same sequence
        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_f64_range() {
        let rng = DeterministicRng::new(42);
        
        for _ in 0..1000 {
            let val = rng.next_f64();
            assert!(val >= 0.0 && val < 1.0);
        }
    }

    #[test]
    fn test_custom_range() {
        let rng = DeterministicRng::new(42);
        
        for _ in 0..1000 {
            let val = rng.next_range(10.0, 20.0);
            assert!(val >= 10.0 && val < 20.0);
        }
    }
}
