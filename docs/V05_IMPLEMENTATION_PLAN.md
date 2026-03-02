# V0.5 Legitimacy System — Implementation Plan

## Executive Summary

**V0.5 Goal:** Add legitimacy mechanics to constrain nation overextension and create realistic rise/fall cycles.

**Duration:** 3 weeks (21 days)
**Effort:** 1 developer, part-time academic schedule (~40 hours/week)
**Output:** 1 new subsystem phase, ~800+ lines, 20+ unit tests, 100% deterministic

**Why V0.5 is critical:**

- V0.4 alliances create external commitments
- V0.5 adds internal constraint (legitimacy erosion)
- Together: realistic geopolitical equilibrium
- Without V0.5: empires can sustain unlimited overextension (breaks realism)

---

## Architecture Overview

### Current Pipeline (V0.4)

```
1. EconomyPhase         → Production, consumption, deficits
2. TradePhase           → Blockades, route disruption
3. WarfarePhase         → Logistics, attrition, unit losses
4. LogisticsPhase       → Supply line evaluation
5. CombatPhase          → Battle resolution, losses
6. OccupationPhase      → Territory control, stability penalties
7. StabilityPhase       → Population morale (placeholder)
8. AlliancePhase        → Cohesion decay, dissolution
9. DiplomacyPhase       → Reputation scoring, proposals
10. DemographicsPhase   → Population growth, migration
11. (EMPTY)             → Reserve for future
```

### New V0.5 Pipeline

```
Existing phases 1-10 (unchanged)
11. LegitimacyPhase     → NEW: legitimacy decay/recovery
12. (Reserve for future)
```

**LegitimacyPhase runs LAST** to aggregate all stressors:

- War exhaustion (accumulated losses)
- Economic deficit stress
- Alliance burden (commitment failures)
- Peace recovery bonus

---

## Core Legitimacy Formula

### Legitimacy Change Per Tick

```
legitimacy_next = legitimacy_current + legitimacy_change

legitimacy_change = war_exhaust + deficit_stress + alliance_burden + peace_bonus

where:

war_exhaust = -0.5 * (active_wars) - 0.25 * (casualties / population)
              (max decay: -2.0/tick)

deficit_stress = -0.75 * (total_deficit > 0 ? deficit_magnitude / 100 : 0)
                 (max decay: -1.5/tick)

alliance_burden = -0.1 * (number_of_alliances) + alliance_crisis_modifier
                  (max decay: -0.3/tick if multiple alliances at war)

peace_bonus = +0.3 * (is_at_peace ? 1 : 0) * (years_at_peace_factor)
              (recovery +0.3/tick when peaceful)

legitimacy_recovered = legitimacy + min(peace_bonus, 2.0/tick)
                       (cap recovery at +2.0/tick)
```

### Legitimacy Thresholds

```
legitimacy >= 50     : Normal regime, full authority
legitimacy 25-50     : Weakened regime, increased rebellion risk
legitimacy < 25      : Weak regime, civil unrest (stability penalties)
legitimacy < 0       : TOTAL COLLAPSE → nation fragments into factions (V0.6)
```

### Legitimacy Effects on Nation

| Legitimacy Range | Status       | Effects                                       |
| ---------------- | ------------ | --------------------------------------------- |
| 75-100           | **Stable**   | +10% production, normal alliance capabilities |
| 50-75            | **Healthy**  | Normal baseline                               |
| 25-50            | **Weakened** | -5% production, increased rebellion chance    |
| 0-25             | **Crisis**   | -15% production, alliance cohesion -1/tick    |
| < 0              | **Collapse** | Nation splinters into factions (V0.6 feature) |

---

## Implementation Roadmap (3-Week Breakdown)

### WEEK 1: Core System & Tests

**Objective:** Build legitimacy calculation engine, pass all unit tests.

#### Tasks

1. **Create legitimacy.rs file**

   Location: `crates/alalamien-engine/src/subsystems/legitimacy.rs`

   ```rust
   pub struct LegitimacyPhase;

   impl TickPhase for LegitimacyPhase {
       fn execute(&self, world: &mut World) {
           // Calculate legitimacy changes
           // Apply penalties/bonuses
       }
   }
   ```

2. **Add legitimacy fields to Nation component**

   In `crates/alalamien-engine/src/core/types.rs`:

   ```rust
   pub struct Nation {
       // ... existing fields ...
       pub legitimacy: f32,              // 0-100 scale
       pub war_exhaustion: f32,          // Cumulative casualty counter
       pub deficit_accumulated: f32,     // Running deficit total
       pub years_at_peace: u32,          // Tick counter for peace bonus
       pub alliance_burden: f32,         // Alliance commitment stress
   }
   ```

3. **Implement calculation functions**

   ```rust
   fn calculate_war_exhaustion(nation: &Nation, world: &World) -> f32;
   fn calculate_deficit_stress(nation: &Nation) -> f32;
   fn calculate_alliance_burden(nation: &Nation, world: &World) -> f32;
   fn calculate_peace_bonus(nation: &Nation) -> f32;
   fn apply_legitimacy_change(nation: &mut Nation, change: f32);
   ```

4. **Unit tests (8 tests minimum)**
   - `test_war_exhaustion_calculation`: Verify casualty-to-decay formula
   - `test_deficit_stress_application`: Deficit > levy decay
   - `test_alliance_burden_stacking`: Multiple alliances increase burden
   - `test_peace_bonus_recovery`: Peace restores legitimacy at +0.3/tick
   - `test_legitimacy_ceiling`: Cap at +2.0 recovery/tick
   - `test_legitimacy_floor`: Prevent negative legitimacy
   - `test_stable_nation_unchanged`: No stresses → legitimacy stable
   - `test_equilibrium_calculation`: War-exhausted nation reaches balance

**Deliverable:** 91 → 99 tests passing

---

### WEEK 2: Integration & Validation

**Objective:** Connect legitimacy to existing subsystems (Economy, Warfare, Alliance, Diplomacy).

#### Integration Points

1. **EconomyPhase → LegitimacyPhase**

   Capture deficit data into `nation.deficit_accumulated`:

   ```rust
   // In legitimacy.rs
   pub fn sync_deficits_from_economy(world: &mut World) {
       // Read total_deficit from each nation
       // Add to deficit_accumulated if deficit > 0
       // Reset if entered surplus (recovery)
   }
   ```

2. **WarfarePhase → LegitimacyPhase**

   Track casualties in `nation.war_exhaustion`:

   ```rust
   // During CombatPhase, track losses:
   nation.war_exhaustion += casualties as f32;

   // LegitimacyPhase reads war_exhaustion
   let current_wars = count_active_wars(nation);
   let exhaustion_decay = -0.5 * current_wars as f32 - 0.25 * nation.war_exhaustion / nation.population;
   ```

3. **AlliancePhase → LegitimacyPhase**

   Detect alliance crises:

   ```rust
   // If allied nation is at war and we haven't contributed:
   nation.alliance_burden += -0.3; // Crisis modifier

   // LegitimacyPhase reads alliance_burden
   ```

4. **DiplomacyPhase → LegitimacyPhase**

   Legitimacy affects alliance proposal likelihood:

   ```rust
   // Higher legitimacy = more attractive ally
   let legitimacy_factor = (nation.legitimacy / 100.0).min(1.0);
   alliance_proposal_score *= legitimacy_factor;
   ```

#### Tests (6 tests minimum)

- `test_economy_deficit_integration`: Deficit flows to legitimacy decay
- `test_warfare_casualty_integration`: Casualties tracked as war exhaustion
- `test_alliance_integration`: Multiple alliances increase burden correctly
- `test_diplomacy_legitimacy_factor`: Lower legitimacy reduces proposal score
- `test_cross_subsystem_equilibrium`: 100-tick stable cycle
- `test_combined_stressors`: War + deficit + alliance burden stack correctly

**Deliverable:** 99 → 105 tests passing

---

### WEEK 3: Validation, Documentation, Exit Criteria

**Objective:** Prove V0.5 is deterministic, performant, and production-ready.

#### Long-Running Simulation Tests (1000-ticks)

```rust
#[test]
fn test_legitimacy_1000_tick_cycle() {
    let mut world = World::new_v0_5_test();
    let mut rng = SeededRng::new(12345);

    let initial_hash = world.hash();

    for _ in 0..1000 {
        world.tick(&mut rng);
    }

    // Assertions:
    // 1. No NaN legitimacy values
    assert!(!world.nations.iter().any(|n| n.legitimacy.is_nan()));

    // 2. No nation has legitimacy > 100 or < initial_collapsed
    assert!(world.nations.iter().all(|n| n.legitimacy <= 100.0));

    // 3. Replay produces identical hash
    let mut replay_world = World::new_v0_5_test();
    for _ in 0..1000 {
        replay_world.tick(&mut SeededRng::new(12345));
    }
    assert_eq!(replay_world.hash(), initial_hash);

    // 4. Legitimacy changes are explainable
    // (log should show war_exhaust + deficit_stress + peace_bonus = total_change)
    verify_legitimacy_calculations(&world);
}
```

#### Performance Validation

- **Baseline (V0.4):** <15ms/tick for 200 nations
- **Target (V0.5):** <17ms/tick (allow +2ms for legitimacy calculations)
- **Measurement:** Run 100 ticks, measure average

```rust
#[test]
fn test_legitimacy_phase_performance() {
    let world = World::new_v0_5_large(); // 200 nations

    let start = std::time::Instant::now();
    for _ in 0..100 {
        world.tick(&mut rng);
    }
    let elapsed = start.elapsed();

    let avg_tick_ms = elapsed.as_millis() as f32 / 100.0;
    assert!(avg_tick_ms < 17.0, "LegitimacyPhase too slow: {}ms", avg_tick_ms);
}
```

#### Documentation Tasks

1. **V0.5_LEGITIMACY_SYSTEM.md** (similar to V0.4 guides)
   - System overview (1 page)
   - Formula breakdown (2 pages)
   - Integration guide (1 page)
   - Test result summary (1 page)

2. **Update ROADMAP.md**
   - Mark V0.5 complete
   - V0.6 section now active

3. **Code comments**
   - Every formula has 1-2 line explanation
   - Every integration point documented
   - Complex calculations have truth table comments

#### Exit Criteria

- ✅ All 110+ tests passing (0 failures, 0.35s runtime)
- ✅ 1000-tick determinism proof (exact replay match)
- ✅ <17ms/tick performance (profiled on reference machine)
- ✅ No NaN/Inf legitimacy values in any test
- ✅ Legitimacy formulas match documented equations exactly
- ✅ All integration points tested (economy, warfare, alliance, diplomacy)
- ✅ Code compiled with zero warnings
- ✅ Documentation complete (formulas explained to non-programmer)

**Deliverable:** 105 → 110+ tests passing, V0.5 marked COMPLETE

---

## Detailed Code Structure

### File: legitimacy.rs (~200 lines)

```rust
// crates/alalamien-engine/src/subsystems/legitimacy.rs

use crate::core::types::{Nation, World};
use crate::core::tick::TickPhase;

/// LegitimacyPhase: Final phase (phase 11)
/// Aggregates all stressors (war, deficit, alliance, peace)
/// Applies legitimacy changes to all nations
pub struct LegitimacyPhase;

impl TickPhase for LegitimacyPhase {
    fn execute(&self, world: &mut World) {
        for nation in &mut world.nations {
            let change = calculate_legitimacy_change(nation, world);
            apply_legitimacy_change(nation, change);
        }
    }
}

/// Calculate all stressor components
fn calculate_legitimacy_change(nation: &Nation, world: &World) -> f32 {
    let war_exhaustion = calculate_war_exhaustion(nation, world);
    let deficit_stress = calculate_deficit_stress(nation);
    let alliance_burden = calculate_alliance_burden(nation, world);
    let peace_bonus = calculate_peace_bonus(nation);

    war_exhaustion + deficit_stress + alliance_burden + peace_bonus
}

/// Stressor: Every active war and casualties erode legitimacy
/// Formula: -0.5 per war - 0.25 * (casualties / population)
fn calculate_war_exhaustion(nation: &Nation, world: &World) -> f32 {
    let active_wars = world.count_active_wars_for(nation.id);
    let casualty_ratio = nation.war_exhaustion / nation.population.max(1.0);

    let base_decay = -0.5 * active_wars as f32;
    let casualty_decay = -0.25 * casualty_ratio;

    (base_decay + casualty_decay).max(-2.0) // Cap at -2.0/tick
}

/// Stressor: Economic deficits stress ruler legitimacy
/// Formula: -0.75 * (total_deficit / 100) when deficit > 0
fn calculate_deficit_stress(nation: &Nation) -> f32 {
    if nation.total_deficit > 0.0 {
        let stress_ratio = nation.total_deficit / 100.0;
        (-0.75 * stress_ratio).max(-1.5) // Cap at -1.5/tick
    } else {
        0.0 // Positive balance doesn't provide bonus (only peace does)
    }
}

/// Stressor: Alliance commitments during crises erode legitimacy
/// Formula: -0.1 * alliance_count, -0.3 if allied nation at war
fn calculate_alliance_burden(nation: &Nation, world: &World) -> f32 {
    let alliance_count = world.count_alliances_for(nation.id);
    let base_burden = -0.1 * alliance_count as f32;

    let crisis_modifier = if world.has_allied_at_war(nation.id) {
        -0.3 // Additional penalty if ally needs support
    } else {
        0.0
    };

    (base_burden + crisis_modifier).max(-0.5) // Cap at -0.5/tick
}

/// Bonus: Peace allows legitimacy recovery
/// Formula: +0.3/tick when at peace
fn calculate_peace_bonus(nation: &Nation) -> f32 {
    if !nation.at_war {
        0.3 * (1.0 + nation.years_at_peace as f32 * 0.05).min(2.0)
    } else {
        nation.years_at_peace = 0;
        0.0
    }
}

/// Apply legitimacy change with floor/ceiling
fn apply_legitimacy_change(nation: &mut Nation, change: f32) {
    nation.legitimacy = (nation.legitimacy + change)
        .max(0.0)    // Floor at 0
        .min(100.0); // Ceiling at 100

    if !nation.at_war {
        nation.years_at_peace += 1;
    }
}

// Test functions...
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_war_exhaustion_calculation() {
        // Two active wars, 20% casualty ratio
        let nation = Nation {
            id: 1,
            at_war: true,
            war_exhaustion: 200.0,
            population: 1000.0,
            ..default()
        };
        let world = default_world_with_2_wars();

        let exhaust = calculate_war_exhaustion(&nation, &world);
        // Expected: -0.5 * 2 - 0.25 * (200/1000) = -1.0 - 0.05 = -1.05
        assert!((exhaust + 1.05).abs() < 0.01);
    }

    // ... more tests ...
}
```

### File: types.rs (Updated Fields)

```rust
// In crates/alalamien-engine/src/core/types.rs
// Add these fields to the Nation struct:

pub struct Nation {
    // ... existing fields ...

    // V0.5 Legitimacy Fields
    pub legitimacy: f32,              // 0-100 scale, ruler credibility
    pub war_exhaustion: f32,          // Cumulative casualties (triggers decay)
    pub deficit_accumulated: f32,     // Running total of deficit stress
    pub years_at_peace: u32,          // Ticks at peace (enables recovery)
    pub alliance_burden: f32,         // Active alliance count stress
}
```

### File: tick.rs (Update Pipeline)

```rust
// In crates/alalamien-engine/src/core/tick.rs
// Update TickPipeline to include LegitimacyPhase:

pub struct TickPipeline {
    economy: EconomyPhase,
    trade: TradePhase,
    warfare: WarfarePhase,
    logistics: LogisticsPhase,
    combat: CombatPhase,
    occupation: OccupationPhase,
    stability: StabilityPhase,
    alliance: AlliancePhase,
    diplomacy: DiplomacyPhase,
    demographics: DemographicsPhase,
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
        self.economy.execute(world);
        self.trade.execute(world);
        self.warfare.execute(world);
        self.logistics.execute(world);
        self.combat.execute(world);
        self.occupation.execute(world);
        self.stability.execute(world);
        self.alliance.execute(world);
        self.diplomacy.execute(world);
        self.demographics.execute(world);
        self.legitimacy.execute(world);  // NEW
    }
}
```

---

## Testing Strategy

### Unit Tests (20+ minimum)

**Category 1: Calculation (8 tests)**

- War exhaustion formula accuracy
- Deficit stress formula accuracy
- Alliance burden stacking
- Peace bonus scaling
- Legitimacy caps (0-100)
- Floor/ceiling enforcement
- Stable nation (no stressors)
- Equilibrium detection

**Category 2: Integration (6 tests)**

- Economy deficit → legitimacy decay
- Warfare casualties → exhaustion
- Alliance membership → burden
- Diplomacy scoring × legitimacy
- Peace timer → bonus eligibility
- Multi-stressor interaction

**Category 3: Long-run validation (4+ tests)**

- 1000-tick determinism
- No NaN/Inf values
- Performance <17ms/tick
- Stable rise/fall cycles
- Replay verification

### Determinism Proof

Every test must verify:

```rust
#[test]
fn test_name_determinism() {
    let seed = 12345;

    // Run 1
    let world1 = simulate_n_ticks(seed, 100);
    let hash1 = world1.hash();

    // Run 2 (identical setup)
    let world2 = simulate_n_ticks(seed, 100);
    let hash2 = world2.hash();

    // Must match
    assert_eq!(hash1, hash2, "Determinism broken!");
}
```

---

## Performance Targets

| Metric                      | V0.4 Baseline | V0.5 Target | Status                         |
| --------------------------- | ------------- | ----------- | ------------------------------ |
| Tick duration (200 nations) | <15ms         | <17ms       | ✓ +2ms acceptable              |
| Test suite runtime          | 0.34s         | 0.40s       | ✓ +0.06s acceptable            |
| Memory per nation           | ~1.2KB        | ~1.4KB      | ✓ +200 bytes legitimacy fields |
| Legitimacy calc time        | N/A           | <0.5ms      | ✓ O(n) operation               |

**Measurement method:**

```bash
time cargo test --release -- --nocapture
# Should complete in <0.5s
```

---

## Week-by-Week Checklist

### WEEK 1

- [ ] Create legitimacy.rs with skeleton
- [ ] Add legitimacy fields to Nation
- [ ] Implement 4 calculation functions
- [ ] Write 8 unit tests
- [ ] Verify 99 tests pass
- [ ] Code review: formulas match spec

### WEEK 2

- [ ] Integrate with EconomyPhase
- [ ] Integrate with WarfarePhase
- [ ] Integrate with AlliancePhase
- [ ] Integrate with DiplomacyPhase
- [ ] Write 6 integration tests
- [ ] Verify 105+ tests pass
- [ ] Profile: <17ms/tick target

### WEEK 3

- [ ] Write V0.5_LEGITIMACY_SYSTEM.md
- [ ] Run 1000-tick determinism test
- [ ] Complete performance validation
- [ ] Final code review
- [ ] Verify 110+ tests pass
- [ ] Mark V0.5 COMPLETE in ROADMAP.md
- [ ] Celebrate! 🎉

---

## Success Metrics

When V0.5 is complete, you can say:

> "I built a **legitimacy erosion system** that models ruler credibility across 5 stressors (war, economy, alliances, peace recovery). The system creates realistic geopolitical equilibrium: empires expand until costs mount, then stabilize or collapse. **100+ tests, 1000-tick determinism verified, <17ms/tick performance. Fully integrated with economy, warfare, alliance, and diplomacy subsystems.**"

---

## Failure Modes to Avoid

❌ **Don't:** Hardcode legitimacy values per nation
✅ **Do:** Calculate from formulas every tick

❌ **Don't:** Allow legitimacy > 100 or < 0
✅ **Do:** Enforce strict 0-100 ceiling/floor

❌ **Don't:** Skip determinism testing
✅ **Do:** Verify 1000-tick exact replay

❌ **Don't:** Let legitimacy sit unchanged for nations at peace
✅ **Do:** Apply +0.3/tick recovery bonus

---

## Next Steps After V0.5

Once V0.5 passes all exit criteria:

1. **Document lessons** for V0.6 (factions)
2. **Plan V0.6** (factions & events) — parallel structure
3. **Gather 1000-tick sample** for reproducibility proof
4. **Prepare hiring pitch:** "5 integrated subsystems, 100+ tests"

V0.6 will use same pattern: new phase, 20+ tests, determinism proof.

---

## References

- **ROADMAP.md** — Full project timeline
- **V0.4_Alliance_SYSTEM.md** — Similar detailed breakdown
- **V0.4_INTEGRATION_GUIDE.md** — How to integrate subsystems
- **crates/alalamien-engine/src/** — Current codebase

---

**Status:** Option 1 confirmed. Ready to begin Week 1 implementation. 🚀
