# V0.5 Week 1 — Quick Start Guide

## Decision Summary: Option 1 ✅

**Path:** Pure engine development (weeks 1-8) → Bevy UI integration (week 9+)

**Decision Rationale:**

- Solidify core before architecture changes
- Reduce risk of mid-subsystem refactoring
- Complete all 6 subsystems in isolation
- Maintain determinism validation throughout

**Timeline Impact:**

- V0.5 Legitimacy: Weeks 1-3 (3 weeks)
- V0.6 Factions & Events: Weeks 4-5 (2-3 weeks)
- V0.6 Hardening: Week 6 (1 week)
- V1.0 Polish & CLI: Weeks 7-8 (2 weeks)
- Bevy UI Integration: Week 9+ (2-3 weeks)

**Total to V1.0 with UI: ~13-15 weeks** ✓

---

## What We've Prepared

### 1. Updated ROADMAP.md

- Marked Option 1 selected
- Updated project timeline
- Clarified pure engine → UI integration flow

### 2. V05_IMPLEMENTATION_PLAN.md

**Comprehensive 1,800+ line technical guide covering:**

- Architecture overview (11 + 1 new phases)
- Legitimacy formula (war, deficit, alliance, peace)
- Week-by-week breakdown (task lists, exit criteria)
- Code structure (legitimacy.rs, types.rs, tick.rs)
- Testing strategy (20+ unit tests)
- Performance targets (<17ms/tick)
- Success criteria checklist

---

## Week 1 Mission (Days 1-7)

### Goal

Build legitimacy calculation engine, pass 99 tests

### Specific Tasks

#### Day 1-2: Setup & File Creation

```
[ ] Create crates/alalamien-engine/src/subsystems/legitimacy.rs (skeleton)
[ ] Add legitimacy fields to Nation struct in types.rs:
    - legitimacy: f32 (0-100)
    - war_exhaustion: f32
    - deficit_accumulated: f32
    - years_at_peace: u32
    - alliance_burden: f32
[ ] Update tick.rs to register LegitimacyPhase (phase 11)
```

#### Day 3-4: Core Functions

```
[ ] Implement calculate_war_exhaustion()
[ ] Implement calculate_deficit_stress()
[ ] Implement calculate_alliance_burden()
[ ] Implement calculate_peace_bonus()
[ ] Implement apply_legitimacy_change()
```

#### Day 5-7: Tests & Validation

```
[ ] Unit test: war exhaustion formula (test_war_exhaustion_calculation)
[ ] Unit test: deficit stress (test_deficit_stress_application)
[ ] Unit test: alliance burden (test_alliance_burden_stacking)
[ ] Unit test: peace bonus (test_peace_bonus_recovery)
[ ] Unit test: legitimacy ceiling (test_legitimacy_ceiling)
[ ] Unit test: legitimacy floor (test_legitimacy_floor)
[ ] Unit test: stable nation (test_stable_nation_unchanged)
[ ] Unit test: equilibrium (test_equilibrium_calculation)
[ ] Verify: 99+ tests passing
[ ] Verify: Zero compilation errors
```

---

## Code Snippets to Use

### 1. Nation Structure Addition

```rust
// crates/alalamien-engine/src/core/types.rs

pub struct Nation {
    // ... existing fields (id, name, population, etc) ...

    // V0.5: Legitimacy System
    pub legitimacy: f32,              // 0-100 scale
    pub war_exhaustion: f32,          // Cumulative casualties
    pub deficit_accumulated: f32,     // Running deficit total
    pub years_at_peace: u32,          // Peace counter
    pub alliance_burden: f32,         // Alliance stress
}
```

### 2. Legitimacy Phase Registration

```rust
// crates/alalamien-engine/src/core/tick.rs

use crate::subsystems::legitimacy::LegitimacyPhase;

pub struct TickPipeline {
    // ... existing phases 1-10 ...
    legitimacy: LegitimacyPhase,  // NEW
}

impl TickPipeline {
    pub fn new_v0_5() -> Self {
        TickPipeline {
            // ... existing phases ...
            legitimacy: LegitimacyPhase,
        }
    }

    pub fn execute(&self, world: &mut World) {
        // ... phases 1-10 ...
        self.legitimacy.execute(world);  // NEW
    }
}
```

### 3. Core Calculation Template

```rust
// crates/alalamien-engine/src/subsystems/legitimacy.rs

use crate::core::types::{Nation, World};
use crate::core::tick::TickPhase;

pub struct LegitimacyPhase;

impl TickPhase for LegitimacyPhase {
    fn execute(&self, world: &mut World) {
        for nation in &mut world.nations {
            let change = calculate_legitimacy_change(nation, world);
            apply_legitimacy_change(nation, change);
        }
    }
}

fn calculate_legitimacy_change(nation: &Nation, world: &World) -> f32 {
    let war = calculate_war_exhaustion(nation, world);
    let deficit = calculate_deficit_stress(nation);
    let alliance = calculate_alliance_burden(nation, world);
    let peace = calculate_peace_bonus(nation);

    war + deficit + alliance + peace
}

fn calculate_war_exhaustion(nation: &Nation, world: &World) -> f32 {
    let active_wars = world.count_active_wars_for(nation.id);
    let casualty_ratio = nation.war_exhaustion / nation.population.max(1.0);

    let base = -0.5 * active_wars as f32;
    let casualties = -0.25 * casualty_ratio;

    (base + casualties).max(-2.0)
}

fn calculate_deficit_stress(nation: &Nation) -> f32 {
    if nation.total_deficit > 0.0 {
        (-0.75 * (nation.total_deficit / 100.0)).max(-1.5)
    } else {
        0.0
    }
}

fn calculate_alliance_burden(nation: &Nation, world: &World) -> f32 {
    let count = world.count_alliances_for(nation.id);
    let base = -0.1 * count as f32;
    let crisis = if world.has_allied_at_war(nation.id) { -0.3 } else { 0.0 };

    (base + crisis).max(-0.5)
}

fn calculate_peace_bonus(nation: &Nation) -> f32 {
    if !nation.at_war {
        let bonus = 0.3 * (1.0 + nation.years_at_peace as f32 * 0.05).min(2.0);
        bonus.min(2.0) // Cap at +2.0/tick
    } else {
        0.0
    }
}

fn apply_legitimacy_change(nation: &mut Nation, change: f32) {
    nation.legitimacy = (nation.legitimacy + change).max(0.0).min(100.0);
}
```

### 4. Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_war_exhaustion_calculation() {
        // Setup: nation with 2 active wars, 20% casualty ratio
        let mut nation = Nation::default();
        nation.id = 1;
        nation.at_war = true;
        nation.war_exhaustion = 200.0;
        nation.population = 1000.0;

        // Setup: mock world with 2 wars for this nation
        let world = test_world_with_wars(nationality_id=1, num_wars=2);

        // Execute
        let exhaustion = calculate_war_exhaustion(&nation, &world);

        // Expected: -0.5 * 2 (wars) - 0.25 * (200/1000) = -1.0 - 0.05 = -1.05
        let expected = -1.05;
        assert!((exhaustion - expected).abs() < 0.01,
                "Expected {}, got {}", expected, exhaustion);
    }

    #[test]
    fn test_legitimacy_ceiling() {
        let mut nation = Nation::default();
        nation.legitimacy = 99.0;

        apply_legitimacy_change(&mut nation, 5.0); // Try to add 5

        assert_eq!(nation.legitimacy, 100.0, "Legitimacy should cap at 100");
    }

    #[test]
    fn test_legitimacy_floor() {
        let mut nation = Nation::default();
        nation.legitimacy = 5.0;

        apply_legitimacy_change(&mut nation, -10.0); // Try to subtract 10

        assert_eq!(nation.legitimacy, 0.0, "Legitimacy should floor at 0");
    }

    #[test]
    fn test_stable_nation_no_change() {
        let mut nation = Nation::default();
        nation.legitimacy = 50.0;
        nation.at_war = false;
        nation.total_deficit = 0.0;
        nation.war_exhaustion = 0.0;

        let world = test_world_peaceful();
        let change = calculate_legitimacy_change(&nation, &world);

        // Peaceful nation at peace with no deficit should see modest decline or stability
        // Base no stress, but peace bonus applies
        assert!(change >= 0.0, "Peaceful nation should not decay");
    }
}
```

---

## Success Criteria for Week 1

End of week, you should have:

✅ **Code Created:**

- [ ] legitimacy.rs (200 lines)
- [ ] Nation fields added (5 new fields)
- [ ] tick.rs updated (LegitimacyPhase registered)

✅ **Tests Passing:**

- [ ] 8 unit tests written
- [ ] All 99+ tests pass (0 failures)
- [ ] 0 compilation errors
- [ ] <0.5s test suite runtime

✅ **Documentation:**

- [ ] Legitimacy formula verified in code comments

✅ **Verification:**

- [ ] Cargo builds clean
- [ ] `cargo test --release` passes all tests

---

## Debugging & Troubleshooting

### Issue: "No such method count_active_wars_for"

**Solution:** You need to implement this helper in World or LegitimacyPhase. Check how V0.4 tracks warfare state.

### Issue: "Legitimacy field doesn't serialize"

**Solution:** Ensure Nation struct has `#[derive(Serialize, Deserialize)]`

### Issue: Tests fail with "hash mismatch"

**Solution:** Determinism broken. Check:

1. Are you using SeededRng consistently?
2. Is loop order deterministic?
3. Any floating-point rounding differences?

### Issue: <17ms/tick target missed

**Solution:** Week 1 won't fail on performance. Week 2 integration may trigger profiling need.

---

## Commands You'll Need

```bash
# Run all tests
cargo test --release

# Run single test
cargo test --release -- test_war_exhaustion_calculation --nocapture

# Build without test
cargo build --release

# Check for warnings
cargo clippy

# Run with timing output
time cargo test --release
```

---

## What NOT to Do This Week

❌ Don't implement factions (that's V0.6)
❌ Don't start Bevy UI work (that's week 9)
❌ Don't add new resource types
❌ Don't modify economy/warfare formulas
❌ Don't skip writing tests
❌ Don't ignore determinism failures

---

## Next Phase (Week 2)

After Week 1 passes, Week 2 focuses on:

- Integration with Economy phase
- Integration with Warfare phase
- Integration with Alliance phase
- Integration with Diplomacy phase
- 6 subsystem interaction tests

But that's next week. Focus on Week 1 for now.

---

## You've Got This! 🚀

You've already built 4 subsystems (V0.1-V0.4, 91 tests).
V0.5 follows the exact same pattern.
Trust the process.

**Week 1 Mantra:** Build the calculation engine. Get 8 tests green. Ship it Friday.

---

**Status:** Ready to start. Pick a day this week and begin.
