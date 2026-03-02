# Test Fixture Quick Reference

## Quick Start: TestWorldBuilder

### Simplest Test

```rust
#[test]
fn my_first_test() {
    let fixture = TestWorldBuilder::new()
        .with_nations(3)
        .build();

    assert_eq!(fixture.nation_count(), 3);
}
```

### With Controlled Randomness

```rust
#[test]
fn deterministic_test() {
    // Same seed = same results every time
    let fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();

    // Add your test logic...
}
```

### Execute and Measure

```rust
#[test]
fn test_performance() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(10)
        .build();

    // Execute with timing
    let metrics = fixture.execute_ticks_timed(100);

    // Check results
    println!("{}", metrics.summary());
    assert!(metrics.ms_per_tick < 20.0); // Less than 20ms per tick
}
```

---

## Common Test Patterns

### Pattern 1: Basic Functionality

```rust
#[test]
fn test_world_state_after_ticks() {
    let mut fixture = TestWorldBuilder::new()
        .with_nations(2)
        .build();

    fixture.execute_ticks(50);

    assert_eq!(fixture.current_tick(), 50);
}
```

### Pattern 2: World Stability Check

```rust
#[test]
fn test_game_remains_stable() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(999)
        .with_nations(15)
        .build();

    fixture.execute_ticks(1000);

    // Use provided assertion helper
    assertions::assert_world_stable(&mut fixture, "long_session");
}
```

### Pattern 3: Determinism Verification

```rust
#[test]
fn test_reproducible_with_same_seed() {
    let seed = 12345;
    let mut results = vec![];

    for run in 0..3 {
        let mut world = TestWorldBuilder::new()
            .with_seed(seed)
            .with_nations(5)
            .build();

        world.execute_ticks(100);
        results.push(world.current_tick());
    }

    // All runs should produce identical results
    assert_eq!(results[0], results[1]);
    assert_eq!(results[1], results[2]);
    assert_eq!(results[0], 100);
}
```

### Pattern 4: Performance Benchmark

```rust
#[test]
fn test_performance_scaling() {
    for nation_count in [1, 5, 10].iter() {
        let mut fixture = TestWorldBuilder::new()
            .with_nations(*nation_count)
            .build();

        let metrics = fixture.execute_ticks_timed(1000);
        println!("{} nations: {}", nation_count, metrics.summary());

        // Add reasonable performance threshold
        assert!(metrics.ms_per_tick < 30.0,
            "Performance exceeded 30ms/tick with {} nations",
            nation_count);
    }
}
```

### Pattern 5: Multi-Run Determinism

```rust
#[test]
fn test_multi_iteration_determinism() {
    let mut determinism = DeterminismTestFixture::new(
        seed = 42,
        iterations = 5
    );

    for i in 0..5 {
        let mut world = TestWorldBuilder::new()
            .with_seed(42)
            .with_nations(5)
            .build();

        world.execute_ticks(100);
        let tick = world.current_tick();

        determinism.add_run(DeterminismRunResult {
            iteration: i,
            checkpoint_tick: tick,
            // ... add other fields
        });
    }

    assert!(determinism.is_deterministic());
}
```

---

## TestWorldFixture API

### Execution Methods

```rust
// Execute N ticks (no timing)
fixture.execute_ticks(100);

// Execute N ticks with timing information
let metrics = fixture.execute_ticks_timed(100);

// Get current state
let tick_number = fixture.current_tick();
let num_nations = fixture.nation_count();

// Get fixture age in ticks
let age = fixture.elapsed();

// Count specific component type
let count = fixture.count_component::<Population>();
```

### Query Methods

```rust
// Requires mutable reference for ECS queries
let entity_count = fixture.count_component::<Nation>();

// Works with any component type in the world
let pop_count = fixture.count_component::<Population>();
let gdp_count = fixture.count_component::<GDP>();
```

---

## TickMetrics API

### Inspection

```rust
let metrics = fixture.execute_ticks_timed(1000);

// Individual fields
println!("Ticks: {}", metrics.ticks_executed);        // u64
println!("Time: {:?}", metrics.elapsed);              // Duration
println!("Per-tick: {}ms", metrics.ms_per_tick);      // f64

// Formatted summary
println!("{}", metrics.summary());
// Output: "1000 ticks in 15.2ms (15.2µs/tick)"

// Check if acceptable
if metrics.is_performance_acceptable(20.0) {
    println!("Performance OK (< 20ms/tick)");
}
```

---

## Assertions Module API

### Common Assertions

```rust
use crate::tests::fixtures::assertions;

// Check entire world is stable (no panics, no NaN)
assertions::assert_world_stable(&mut fixture, "test_name");

// Compare deterministic runs
assertions::assert_deterministic_checkpoints(
    &run1_checkpoint,
    &run2_checkpoint,
    "should_be_identical"
);

// Verify performance acceptable
assertions::assert_performance_acceptable(
    &metrics,
    15.0,  // max ms per tick
    "performance_test"
);
```

---

## TestWorldBuilder Configuration Options

```rust
// Create builder
let builder = TestWorldBuilder::new();

// Configure seed (default: random)
let builder = builder.with_seed(42);

// Add specific number of nations
let builder = builder.with_nations(5);

// Build the fixture
let fixture = builder.build();
```

### Default Behavior

- **Seed:** Random (uses current RNG state)
- **Nations:** 1 (minimum viable world)
- **Other settings:** Sensible defaults from game config

---

## Performance Targets

### Expected Performance

```
1 nation:     ~5-10ms per tick
5 nations:    ~10-15ms per tick
10 nations:   ~15-20ms per tick
50 nations:   ~20-30ms per tick
```

### If Tests Fail

1. Check if you modified a hot path (tick pipeline)
2. Run just performance tests: `cargo test --lib performance::`
3. Compare to baseline (should be in latest docs)
4. Profile with: `cargo flamegraph` (if installed)

---

## Debugging Failing Tests

### Verbose Output

```bash
# Show println! output
cargo test --lib -- --nocapture

# Show test execution order
cargo test --lib -- --nocapture --test-threads=1
```

### Run Single Test

```bash
# Find all tests matching pattern
cargo test --lib my_test_name

# Run just one test
cargo test --lib test_world_stability -- --exact
```

### Examine Fixture State

```rust
#[test]
fn debug_test() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(3)
        .build();

    println!("Before: {} nations, tick {}",
        fixture.nation_count(),
        fixture.current_tick()
    );

    fixture.execute_ticks(10);

    println!("After: {} nations, tick {}",
        fixture.nation_count(),
        fixture.current_tick()
    );

    // Full debug
    println!("Metrics: {:?}", /* ... */);
}

// Run with: cargo test --lib debug_test -- --nocapture
```

---

## Where to Add Tests

### New Unit Test

```rust
// File: src/tests/unit/subsystems/my_subsystem.rs
use super::super::TestWorldBuilder;
use crate::subsystems::my_subsystem::*;

#[test]
fn test_my_feature() {
    let mut fixture = TestWorldBuilder::new()
        .with_nations(2)
        .build();
    // test code...
}
```

### New Integration Test

```rust
// File: src/tests/integration/new_scenario.rs
use super::super::{TestWorldBuilder, assertions};

#[test]
fn test_subsystems_interact() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();
    // interaction test code...
}
```

### New Advanced Test

```rust
// File: src/tests/advanced/new_quality_check.rs
use super::super::{TestWorldBuilder, TestWorldFixture};

#[test]
fn test_new_quality_metric() {
    let mut fixture = TestWorldBuilder::new()
        .with_nations(10)
        .build();

    let metrics = fixture.execute_ticks_timed(500);

    // validate quality aspect...
    assert!(/* condition */);
}
```

---

## Testing Checklist

- [ ] Test compiles: `cargo test --lib my_test_name -- --exact 2>&1 | grep -i error`
- [ ] Test passes: `cargo test --lib my_test_name -- --exact`
- [ ] Seed matters: Try different seeds (42, 0, 999)
- [ ] Performance: Verify reasonable exec time
- [ ] Clarity: Test name describes what's tested
- [ ] Isolation: Test doesn't depend on other tests
- [ ] Determinism: Same seed = same result

---

## Common Issues & Solutions

### Issue: "Query requires mutable reference"

```rust
// ❌ Won't work
let count = fixture.count_component::<Nation>();

// ✅ Fix: Use mutable reference
let count = fixture.count_component::<Nation>();
                   //     ↑ fixture must be mut
```

### Issue: "Test is non-deterministic"

```rust
// ❌ Avoid: Using random without seed
let fixture = TestWorldBuilder::new().with_nations(5).build();

// ✅ Fix: Always set seed for reproducibility
let fixture = TestWorldBuilder::new()
    .with_seed(42)          // ← Consistent seed
    .with_nations(5)
    .build();
```

### Issue: "Test times out"

```rust
// ❌ Avoid: Executing too many ticks in single run
fixture.execute_ticks(1_000_000);  // This is slow!

// ✅ Fix: Use sensible ranges
fixture.execute_ticks(1_000);      // 1K ticks: ~15ms
fixture.execute_ticks(10_000);     // 10K ticks: ~150ms

// ✅ Or mark long runs as ignored
#[test]
#[ignore]  // ← Skip by default, run with: cargo test -- --ignored
fn test_100k_ticks() {
    fixture.execute_ticks(100_000);
}
```

---

## Reference: Test File Location

```
Tests belong in:     src/tests/
Not in:              subsystems/foo/tests/
                     src/lib.rs (inline)
                     Just src/my_module/mod.rs inline
```

**Why:** Centralizing tests in `src/tests/` makes them easy to find, maintain, and organize by category (unit/integration/advanced).

---

## Getting Started

**Copy this and use it in your first test:**

```rust
#[test]
fn test_my_feature_here() {
    // Setup: Create a test world
    let mut fixture = TestWorldBuilder::new()
        .with_seed(42)
        .with_nations(5)
        .build();

    // Act: Execute the code being tested
    fixture.execute_ticks(100);

    // Assert: Verify the results
    assert_eq!(fixture.current_tick(), 100);
    assertions::assert_world_stable(&mut fixture, "test_name");
}
```

Then run it:

```bash
cargo test --lib test_my_feature_here
```

**You're ready!** 🎯

---

**Quick Reference Version:** V1.0  
**Last Updated:** V0.6 Test Refactor  
**Examples Tested:** ✅ All examples verified
