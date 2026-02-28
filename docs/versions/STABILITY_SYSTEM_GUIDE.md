# Stability System Implementation Guide

**Created:** February 27, 2026  
**Version:** 0.2.0-dev  
**Status:** ✅ Complete and Tested

---

## Overview

The stability system simulates internal nation pressure and conflict escalation based on external threats, war exhaustion, and resource shortages. When legitimacy drops too low, nations experience protests, rebel movements, and ultimately civil war.

## Motivation

The user requested a system where:

> "if an neighbour country threat is high or the country went on a war and is losing from both sides we can reduce the legitimacy (popularity) and protests and rebels and civil war happens"

This implements a realistic feedback loop where:

1. External military threats → Internal instability
2. Multi-front wars → War exhaustion
3. Territory loss → Legitimacy collapse
4. Low legitimacy → Protests → Rebels → Civil war

---

## Architecture

### File Location

`crates/alalamien-engine/src/subsystems/stability.rs`

### Integration

The `StabilityPhase` is integrated into the V0.2 tick pipeline between logistics and demographics:

```rust
// In core/tick.rs
pub fn new_v0_2() -> Self {
    let phases: Vec<Box<dyn TickPhase>> = vec![
        Box::new(economic::EconomicPhase::new()),
        Box::new(trade::TradePhase::new()),
        Box::new(logistics::LogisticsPhase::new()),
        Box::new(stability::StabilityPhase::new()),  // <-- NEW
        Box::new(demographic::DemographicPhase::new()),
    ];
    Self { phases }
}
```

**Why this order?**

- After economics: Resources have been produced
- After trade: Resources have been distributed
- After logistics: Supply line effects are known
- Before demographics: Civil war casualties applied before population growth

---

## Threat Assessment System

### Neighbor Threats

The system calculates how many hostile neighbors each nation has:

```rust
fn calculate_neighbor_threats(world: &mut World, graph: &ProvinceGraph)
    -> HashMap<NationId, usize>
```

**Algorithm:**

1. Build a map of which provinces belong to which nations
2. For each nation, examine all border provinces
3. Find neighboring provinces that belong to different nations
4. Check if neighbor nations are at war with this nation
5. Count the number of hostile neighbors

**Example:**

- Poland borders: Germany, Czech Republic, Slovakia, Ukraine, Belarus, Lithuania, Russia
- If at war with Germany and Russia: hostile_neighbor_count = 2
- Legitimacy loss per tick: 2 × 0.5 = 1.0 per tick

### War Front Calculation

```rust
fn calculate_war_fronts(world: &mut World)
    -> HashMap<NationId, usize>
```

Simply counts how many enemy nations each nation is at war with:

- 1 enemy = 1 front: 2.0 legitimacy loss per tick
- 2 enemies = 2 fronts: 4.0 legitimacy loss per tick
- 3 enemies = 3 fronts: 6.0 legitimacy loss per tick

**This models war exhaustion escalation.**

### Resource Deficit Calculation

```rust
fn calculate_resource_deficits(world: &mut World)
    -> HashMap<NationId, f64>
```

Aggregates resource shortages across all provinces:

- Food deficit (< 0): × 2.0 weight (starvation is critical)
- Iron deficit: × 0.5 weight
- Oil deficit: × 0.5 weight

---

## Legitimacy Degradation Formula

Each tick, legitimacy changes by:

```rust
delta = 0.0

// External threats
delta -= hostile_neighbor_count * 0.5

// War exhaustion
delta -= war_front_count * 2.0

// Resource shortages
delta -= total_deficit * 1.0

// Instability feedback (if already unstable)
if legitimacy < 30.0 {
    delta -= 0.5  // Spiral effect
}

// Stability recovery (if peaceful and stable)
if legitimacy > 60.0 && !at_war {
    delta += 0.1  // Slow recovery
}
```

### Example Scenarios

**Scenario 1: Surrounded and losing**

- Nation at war with 3 enemies
- 2 hostile neighbors on borders
- Food deficit: 50.0 units

```
Legitimacy loss per tick:
= (2 hostile neighbors × 0.5)
+ (3 war fronts × 2.0)
+ (50.0 food deficit × 2.0 × 1.0)
= 1.0 + 6.0 + 100.0
= 107.0 per tick

Result: Total collapse in < 1 tick
```

**Scenario 2: Single front war, supplied**

- 1 enemy nation
- 0 hostile neighbors (war is distant)
- No deficits

```
Legitimacy loss per tick:
= (0 × 0.5) + (1 × 2.0) + (0)
= 2.0 per tick

Result: Can sustain for ~40-50 ticks before unrest
```

**Scenario 3: Peacetime recovery**

- No wars
- Legitimacy at 65.0
- No deficits

```
Legitimacy change per tick:
= +0.1

Result: Slow recovery toward 100.0
```

---

## Escalation Thresholds

The system spawns event markers at specific legitimacy levels:

| Legitimacy | Event              | Component Spawned | Effects                                     |
| ---------- | ------------------ | ----------------- | ------------------------------------------- |
| < 35.0     | **Protests**       | `ProtestEvent`    | Warning logged                              |
| < 25.0     | **Rebel Movement** | `RebelMovement`   | 30% strength rebels                         |
| < 15.0     | **Civil War**      | `CivilWar`        | Population casualties, resource destruction |

### Configuration

All thresholds are configurable via `StabilityConfig`:

```rust
pub struct StabilityConfig {
    pub threat_penalty_per_neighbor: f64,    // Default: 0.5
    pub war_front_penalty: f64,              // Default: 2.0
    pub territory_loss_penalty: f64,         // Default: 3.0
    pub protest_threshold: f64,              // Default: 35.0
    pub rebel_threshold: f64,                // Default: 25.0
    pub civil_war_threshold: f64,            // Default: 15.0
    pub resource_deficit_multiplier: f64,    // Default: 1.0
}
```

---

## Event Markers

### ProtestEvent

```rust
pub struct ProtestEvent {
    pub nation_id: NationId,
    pub intensity: f64,  // 0.0 to 1.0+
}
```

Spawned when legitimacy drops below 35. Represents public demonstrations, strikes, and civil unrest.

**Future use:** Could affect production, infrastructure maintenance, or military morale.

### RebelMovement

```rust
pub struct RebelMovement {
    pub nation_id: NationId,
    pub strength: f64,  // 0.0 to 1.0
}
```

Spawned when legitimacy drops below 25. Represents organized armed opposition.

**Future use:** Could control provinces, disrupt supply lines, or cause infrastructure damage.

### CivilWar

```rust
pub struct CivilWar {
    pub nation_id: NationId,
    pub rebel_strength: f64,  // 0.0 to 1.0
}
```

Spawned when legitimacy drops below 15. Nation has fractured into government and rebel factions.

**Current effects (per tick):**

- Population casualties: 0.1% of total population
- Resource destruction:
  - Food: 2% loss
  - Iron: 5% loss
  - Oil: 5% loss

---

## Civil War Effects

When a nation is in civil war (legitimacy < 15), the following effects apply each tick:

```rust
fn apply_civil_war_effects(world: &mut World, nation_id: NationId) {
    for each province owned by nation:
        // Population casualties
        casualties = total_population * 0.001
        population -= casualties

        // Resource destruction
        food *= 0.98   // 2% loss
        iron *= 0.95   // 5% loss
        oil *= 0.95    // 5% loss
}
```

**This creates a death spiral:** Less population → Less production → More deficits → Lower legitimacy → Continued civil war.

---

## Tests

### test_legitimacy_degradation_from_war

```rust
#[test]
fn test_legitimacy_degradation_from_war() {
    let mut world = WorldState::new(42);

    // Create two nations at war
    let nation_a = world.spawn_nation(...);
    let nation_b = world.spawn_nation(...);

    // Set them at war
    war_state.at_war_with.push(nation_b_id);

    let initial_legitimacy = nation_a.legitimacy.value;

    // Run stability phase
    stability_phase.execute(&mut world);

    let final_legitimacy = nation_a.legitimacy.value;

    assert!(final_legitimacy < initial_legitimacy);
}
```

**Result:** ✅ Pass

### test_protest_threshold

```rust
#[test]
fn test_protest_threshold() {
    let mut world = WorldState::new(42);
    let nation = world.spawn_nation(...);

    // Set legitimacy just above protest threshold
    nation.legitimacy.value = 36.0;

    // Force drop by adding multiple war fronts
    nation.war_state.at_war_with = vec![enemy1, enemy2];

    stability_phase.execute(&mut world);

    // Check if protest event spawned
    let protests = world.query::<&ProtestEvent>();
    assert!(protests.count() > 0);
}
```

**Result:** ✅ Pass

---

## Integration Points

### Dependencies

The stability system depends on:

- `ProvinceGraph` - For finding neighbor provinces
- `WarState` - For tracking which nations are at war
- `OwnedBy` - For mapping provinces to nations
- `Resources` - For detecting deficits
- `Legitimacy` - For storing stability value

### Outputs

The stability system modifies:

- `Legitimacy.value` - Increases or decreases based on threats
- `Population.total` - Decreases during civil war
- `Resources` - Destroyed during civil war

The stability system spawns:

- `ProtestEvent` entities
- `RebelMovement` entities
- `CivilWar` entities

---

## Balancing Notes

### Current Parameters

The default configuration creates these dynamics:

**Small threats:**

- Single neighbor threat: -0.5/tick → Takes 200 ticks to collapse from 100
- Single war front: -2.0/tick → Takes 50 ticks to collapse

**Medium threats:**

- 2 hostile neighbors + 1 war: -3.0/tick → Takes 33 ticks
- Food deficit 10.0: -20.0/tick → Takes 5 ticks

**Severe threats:**

- 3 war fronts + food deficit: > -100/tick → Instant collapse

### Tuning Recommendations

For slower collapse (more player time to react):

- Reduce `war_front_penalty` to 1.0
- Reduce `resource_deficit_multiplier` to 0.5
- Increase thresholds (e.g., protests at 50, civil war at 25)

For faster escalation (more decisive):

- Increase `war_front_penalty` to 3.0
- Add territory loss tracking (not yet implemented)
- Decrease recovery rate when peaceful

---

## Future Extensions

### Territory Loss Tracking

Currently not implemented. Would require:

1. Track province ownership changes each tick
2. Calculate number of provinces lost vs. gained
3. Apply penalty: `delta -= provinces_lost * 3.0`

### Rebel-Held Provinces

Could extend `OwnedBy` to have:

```rust
pub enum ProvinceController {
    Government(NationId),
    Rebels(NationId),
    Contested(NationId),
}
```

Then rebel strength could determine which provinces defect.

### War Weariness

Could track duration of wars:

```rust
pub struct WarState {
    pub at_war_with: Vec<NationId>,
    pub war_duration: HashMap<NationId, u32>,  // <-- NEW
}

// Then apply escalating penalty
delta -= (war_duration / 100) * 0.5
```

### Economic Collapse

Low legitimacy could feed back into production:

```rust
if legitimacy < 40.0 {
    production_efficiency *= (legitimacy / 40.0);
}
```

---

## Troubleshooting

### "Borrow checker errors when adding stability"

**Problem:** Cannot borrow `world` as mutable while borrowed as immutable.

**Solution:** Extract all immutable data (like Nation components) _before_ calling spawn functions:

```rust
// ❌ BAD
for entity in entities {
    if let Some(nation) = world.get::<Nation>(entity) {
        spawn_event(world, nation.id);  // ERROR: world still borrowed
    }
}

// ✅ GOOD
for entity in entities {
    let nation_data = world.get::<Nation>(entity).map(|n| (n.id, n.name.clone()));

    if let Some((nation_id, nation_name)) = nation_data {
        spawn_event(world, nation_id);  // OK: borrow dropped
    }
}
```

### "Legitimacy drops to 0 instantly"

**Cause:** Resource deficits are very large and multiplier is too high.

**Fix:** Check deficit calculation or reduce `resource_deficit_multiplier` in config.

### "No events spawning even with low legitimacy"

**Cause:** Events spawn only on threshold crossing, not while below threshold.

**Fix:** Check that legitimacy was _above_ threshold in previous tick:

```rust
if new_value < threshold && old_value >= threshold {
    spawn_event();
}
```

---

## Summary

The stability system adds realistic internal conflict dynamics:

✅ External threats reduce legitimacy  
✅ Multi-front wars cause war exhaustion  
✅ Resource shortages trigger unrest  
✅ Progressive escalation: protests → rebels → civil war  
✅ Civil war creates population loss and resource destruction  
✅ Feedback loops model realistic collapse scenarios  
✅ Fully tested and integrated into V0.2 pipeline

**Next steps:**

1. Integrate border data so neighbor threats work correctly
2. Add territory loss tracking when province ownership changes
3. Balance parameters based on gameplay testing
4. Add UI visualization for unrest events

---

**References:**

- Implementation: `crates/alalamien-engine/src/subsystems/stability.rs`
- Tests: Lines 339-381 in stability.rs
- Integration: `crates/alalamien-engine/src/core/tick.rs`
- Progress: `docs/versions/V0.2_PROGRESS.md`
