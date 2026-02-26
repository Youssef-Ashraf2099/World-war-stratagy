//! Math utilities for simulation calculations

/// Clamp a value between min and max
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Linear interpolation
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Smooth step interpolation (cubic Hermite)
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = clamp((x - edge0) / (edge1 - edge0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Calculate percentage change
pub fn percentage_change(old: f64, new: f64) -> f64 {
    if old == 0.0 {
        return 0.0;
    }
    ((new - old) / old) * 100.0
}

/// Calculate growth rate from two values
pub fn growth_rate(initial: f64, final_val: f64, periods: f64) -> f64 {
    if initial <= 0.0 || periods <= 0.0 {
        return 0.0;
    }
    ((final_val / initial).powf(1.0 / periods) - 1.0) * 100.0
}

/// Calculate moving average
pub fn moving_average(values: &[f64], window: usize) -> Vec<f64> {
    if values.is_empty() || window == 0 {
        return vec![];
    }

    let window = window.min(values.len());
    let mut result = Vec::new();

    for i in 0..values.len() {
        let start = if i >= window { i - window + 1 } else { 0 };
        let slice = &values[start..=i];
        let avg = slice.iter().sum::<f64>() / slice.len() as f64;
        result.push(avg);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_percentage_change() {
        assert_eq!(percentage_change(100.0, 150.0), 50.0);
        assert_eq!(percentage_change(100.0, 75.0), -25.0);
        assert_eq!(percentage_change(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_moving_average() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ma = moving_average(&values, 3);
        
        assert_eq!(ma[0], 1.0); // [1] avg
        assert_eq!(ma[1], 1.5); // [1,2] avg
        assert_eq!(ma[2], 2.0); // [1,2,3] avg
        assert_eq!(ma[3], 3.0); // [2,3,4] avg
        assert_eq!(ma[4], 4.0); // [3,4,5] avg
    }
}
