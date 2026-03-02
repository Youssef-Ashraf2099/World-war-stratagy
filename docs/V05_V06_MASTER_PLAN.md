# V0.5 + V0.6 MASTER IMPLEMENTATION PLAN

**Document:** PATH C Execution Guide  
**Scope:** V0.5 Legitimacy + V0.6 Factions & Events  
**Duration:** 6 weeks (V0.5 4 weeks, V0.6 2-3 weeks) + 1 week hardening  
**Target Tests:** 135+ total (95 after V0.5, 135 after V0.6)  
**Success Metric:** 100K-tick deterministic validation with all systems active

---

## 📋 QUICK REFERENCE: V0.5 + V0.6 SCOPE

### V0.5: Legitimacy System (3-4 weeks)

```
NEW FILES:
  crates/alalamien-engine/src/subsystems/legitimacy.rs

NEW TYPES:
  struct Legitimacy { current: f64, stress: f64, exhaustion: f64 }
  component on Nation struct

NEW PHASE:
  LegitimacyPhase (execute at phase 11, after Demographics)

INTEGRATION:
  - EconomyPhase: resource stress → legitimacy loss
  - WarfarePhase: military losses → exhaustion
  - DiplomacyPhase: war initiation → legitimacy penalty
  - AlliancePhase: obligation failure → legitimacy penalty

TESTS:
  20+ new tests covering decay, recovery, collapse, interactions
  10K-tick validation with legitimacy active

FORMULA:
  legitimacy_change =
    (-0.5 to -1.5 × active_wars) +
    (-0.2 to -0.5 per deficit type) +
    (-0.2 if alliance obligations refused) +
    (+0.3 if at peace)

  Result clamped to [0, 100]
  When = 0: nation_collapses()
```

### V0.6: Factions & World Events (2-3 weeks)

```
NEW FILES:
  crates/alalamien-engine/src/subsystems/factions.rs
  crates/alalamien-engine/src/subsystems/world_events.rs

NEW TYPES:
  struct Faction { provinces, legitimacy, military, population, allies }
  enum EventType { Famine, Plague, GeneralBorn, Coup, ... }
  struct WorldEvent { event_type, target, magnitude, duration }

NEW PHASES:
  FactionCivilWarPhase (execute at phase 8, before Alliance)
  WorldEventPhase (execute at phase 12, last phase)

FACTIONS:
  - Collapse → splits into 2-4 factions
  - Civil war mechanics: battles, legitimacy loss, recovery
  - Reunification when victor reaches 70+ legitimacy
  - External intervention (allies support factions)

EVENTS:
  - 15+ event types (environmental, military, economic, diplomatic, rare)
  - 0.5% base probability per tick modified by context
  - Deterministic (seeded RNG, same seed = same events)
  - Integration with all subsystems

TESTS:
  25+ new tests covering factions, events, interactions
  10K-tick validation with events + factions
  Event distribution verification

FORMULA:
  event_probability = 0.005 × context_modifier

  modifiers:
    × 1.5 if legitimacy < 30
    × 0.7 if legitimacy > 75
    × 1.75 if at war
    × 1.4 if famine active
    × 1.3 if just won war
```

---

## 🏗 FOLDER STRUCTURE (After V0.5 + V0.6)

```
crates/alalamien-engine/src/
├── subsystems/
│   ├── alliance.rs              (V0.4)
│   ├── alliance_dataset.rs      (V0.4)
│   ├── diplomacy.rs             (V0.4)
│   ├── economy.rs               (V0.3)
│   ├── trade.rs                 (V0.2)
│   ├── warfare.rs               (V0.3)
│   ├── logistics.rs             (V0.3)
│   ├── occupation.rs            (V0.3)
│   ├── stability.rs             (V0.1)
│   ├── demographics.rs          (V0.1)
│   ├── legitimacy.rs            (V0.5) ← NEW
│   ├── factions.rs              (V0.6) ← NEW
│   ├── world_events.rs          (V0.6) ← NEW
│   └── mod.rs
│
├── types.rs
├── core/
│   ├── tick.rs (updated with new phases)
│   ├── types.rs (add Faction, Event)
│   └── ...
│
└── lib.rs
```

---

## 📝 V0.5 DETAILED IMPLEMENTATION PLAN

### STEP 1: Create LegitimacyPhase Skeleton (Week 1, Day 1-2)

**File:** `crates/alalamien-engine/src/subsystems/legitimacy.rs`

```rust
use bevy_ecs::prelude::*;
use crate::core::types::Nation;

pub struct LegitimacyPhase {
    // No state needed, pure calculations
}

impl LegitimacyPhase {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&mut self, world: &mut World) {
        // Implementation here (see steps below)
    }
}

impl crate::core::tick::TickPhase for LegitimacyPhase {
    fn name(&self) -> &str {
        "Legitimacy"
    }

    fn execute(&mut self, world: &mut World) {
        Self::execute(self, world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legitimacy_placeholder() {
        // Start with placeholder
    }
}
```

**Action:** Let me scaffold this file in code

### STEP 2: Add Legitimacy Component to Nation (Week 1, Day 2-3)

**File:** `crates/alalamien-engine/src/core/types.rs` (update Nation struct)

```rust
pub struct Nation {
    pub id: NationId,
    pub name: String,
    pub color: [u8; 3],
    pub is_ai_controlled: bool,
    pub capital_province_id: ProvinceId,

    // NEW FOR V0.5:
    pub legitimacy: f64,              // 0-100
    pub war_exhaustion: f64,          // 0-100, cumulative
    pub resource_stress: f64,         // 0-100, composite
    pub alliance_obligation_strain: f64, // 0-100
}

impl Nation {
    // Helper methods
    pub fn is_at_crisis(&self) -> bool {
        self.legitimacy < 25.0
    }

    pub fn is_collapsed(&self) -> bool {
        self.legitimacy <= 0.0
    }

    pub fn is_stable(&self) -> bool {
        self.legitimacy > 75.0
    }
}
```

### STEP 3: Implement Legitimacy Calculation (Week 1, Day 3-5)

**In legitimacy.rs, create calculation logic:**

```rust
impl LegitimacyPhase {
    pub fn execute(&mut self, world: &mut World) {
        let mut query = world.query::<(&mut Nation, &WarState, &EconomyState, Option<&Alliance>)>();

        let mut to_process = Vec::new();
        for (entity, (nation, war, economy, _)) in query.iter_with(world) {
            to_process.push((entity, nation.id, world.current_tick()));
        }

        for (entity, nation_id, current_tick) in to_process {
            if let Some(mut nation) = world.get::<&mut Nation>(entity) {
                // Calculate legitimacy change
                let war_penalty = Self::calculate_war_exhaustion(&nation, war);
                let stress_penalty = Self::calculate_resource_stress(&nation, economy);
                let obligation_penalty = Self::calculate_alliance_strain(&nation); // Stub
                let recovery_bonus = Self::calculate_peace_recovery(&nation);

                let change = war_penalty + stress_penalty + obligation_penalty + recovery_bonus;
                nation.legitimacy = (nation.legitimacy + change).clamp(0.0, 100.0);

                // Check collapse
                if nation.legitimacy == 0.0 {
                    nation.is_collapsed = true;
                }
            }
        }
    }

    fn calculate_war_exhaustion(nation: &Nation, war: &WarState) -> f64 {
        if war.is_at_war {
            let base = -0.8;
            let multi_front_penalty = if war.active_fronts > 1 {
                -0.2 * (war.active_fronts as f64 - 1.0)
            } else {
                0.0
            };
            base + multi_front_penalty
        } else {
            0.0
        }
    }

    fn calculate_resource_stress(nation: &Nation, economy: &EconomyState) -> f64 {
        let mut penalty = 0.0;

        if economy.food_balance < -20.0 {
            penalty -= 0.2;
        }
        if economy.iron_balance < -50.0 {
            penalty -= 0.3;
        }
        if economy.oil_balance < -30.0 {
            penalty -= 0.25;
        }

        penalty.clamp(-2.0, 0.0) // Cap at -2.0/tick
    }

    fn calculate_alliance_strain(_: &Nation) -> f64 {
        // Will implement in V0.5 week 2 when connecting to AlliancePhase
        0.0
    }

    fn calculate_peace_recovery(nation: &Nation) -> f64 {
        if !nation.is_at_war {
            0.3 // Base recovery at peace
        } else {
            0.0
        }
    }
}
```

### STEP 4: Integration with Other Systems (Week 2)

**In each subsystem, add legitimacy awareness:**

**economy.rs:**

```rust
// In EconomyPhase::execute()
// If deficit detected, flag for legitimacy calculation
```

**warfare.rs:**

```rust
// In WarfarePhase::execute()
// Set war.active_fronts from number of simultaneous wars
```

**diplomacy.rs:**

```rust
// When war starts/ends, update WarState
```

**alliance.rs:**

```rust
// Track: Did member refuse obligation?
// Trigger obligation_strain penalty
```

### STEP 5: Write Tests (Week 1-2, iterate)

```rust
#[test]
fn test_legitimacy_base_decay() {
    // No wars, no stress
    // Assert: legitimacy +0.3/tick at peace
}

#[test]
fn test_legitimacy_war_exhaustion() {
    // Nation at war 30 ticks
    // Assert: -0.8/tick ≈ -24 legitimacy loss
}

#[test]
fn test_legitimacy_resource_stress() {
    // Food deficit -50, iron deficit -100
    // Assert: -0.5/tick penalty
}

#[test]
fn test_legitimacy_collapse_condition() {
    // Drive to 0
    // Assert: nation.is_collapsed() = true
}

#[test]
fn test_legitimacy_crisis_penalties() {
    // Legitimacy < 25
    // Assert: Military effectiveness -50%
    // Assert: Production -40%
}

#[test]
fn test_1000_ticks_legitimacy() {
    // Simulation with legitimacy active
    // Assert: No crashes, state valid
}
```

### STEP 6: Documentation (Week 3)

Create `V0.5_LEGITIMACY_SPEC.md` covering:

- Mechanics overview
- Formula breakdown
- Integration points
- All test descriptions
- Performance notes

---

## 📝 V0.6 DETAILED IMPLEMENTATION PLAN

### STEP 1: Create Faction Types & System (Week 5, Day 1-3)

**File:** `crates/alalamien-engine/src/subsystems/factions.rs`

```rust
use bevy_ecs::prelude::*;
use crate::core::types::{NationId, ProvinceId, AllianceId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FactionId(pub u32);

pub struct Faction {
    pub faction_id: FactionId,
    pub parent_nation: NationId,      // Original nation
    pub provinces: Vec<ProvinceId>,
    pub legitimacy: f64,              // Independent legitimacy
    pub military_strength: f64,
    pub population: f64,
    pub morale: f64,                  // 0-100
    pub founded_tick: u32,
    pub allies: Vec<EntityId>,        // Other factions or external nations
    pub is_in_civil_war: bool,
}

impl Faction {
    pub fn new(id: FactionId, parent: NationId, tick: u32) -> Self {
        Self {
            faction_id: id,
            parent_nation: parent,
            provinces: Vec::new(),
            legitimacy: 15.0,
            military_strength: 50.0,
            population: 0.0,
            morale: 50.0,
            founded_tick: tick,
            allies: Vec::new(),
            is_in_civil_war: false,
        }
    }

    pub fn can_reunify(&self) -> bool {
        self.legitimacy > 70.0
    }
}

pub struct FactionCivilWarPhase {
    // No state needed
}

impl FactionCivilWarPhase {
    pub fn new() -> Self {
        Self {}
    }

    pub fn execute(&mut self, world: &mut World) {
        // Civil war resolution logic (see below)
    }
}
```

### STEP 2: Implement Collapse → Faction Splitting (Week 5, Day 2-4)

```rust
impl FactionCivilWarPhase {
    pub fn execute(&mut self, world: &mut World) {
        // 1. Check for nations that just collapsed
        let mut collapses = Vec::new();
        {
            let mut query = world.query::<(&Nation, &ProvinceOwnership)>();
            for (entity, (nation, provinces)) in query.iter_with(world) {
                if nation.legitimacy == 0.0 && !nation.is_factionalized {
                    collapses.push((entity, nation.id, provinces.clone()));
                }
            }
        }

        // 2. Create factions for each collapse
        for (nation_entity, nation_id, provinces) in collapses {
            Self::split_nation_into_factions(world, nation_entity, nation_id, provinces);
        }

        // 3. Resolve ongoing civil wars
        Self::resolve_civil_wars(world);

        // 4. Check for reunification
        Self::check_reunification(world);
    }

    fn split_nation_into_factions(
        world: &mut World,
        nation_entity: Entity,
        nation_id: NationId,
        provinces: Vec<ProvinceId>
    ) {
        let num_factions = if provinces.len() < 2 { 2 } else { rand(2..=4) };
        let provinces_per_faction = provinces.len() / num_factions;

        for i in 0..num_factions {
            let faction_provinces = provinces
                .iter()
                .skip(i * provinces_per_faction)
                .take(provinces_per_faction)
                .copied()
                .collect::<Vec<_>>();

            let faction = Faction::new(
                FactionId(i as u32),
                nation_id,
                world.current_tick()
            );

            // Spawn faction entity
            world.spawn(faction);

            // Update province ownership
            for prov_id in faction_provinces {
                // Update ProvinceOwnership to point to faction
            }
        }

        // Mark original nation as factionalized
        if let Some(mut nation) = world.get::<&mut Nation>(nation_entity) {
            nation.is_factionalized = true;
        }
    }

    fn resolve_civil_wars(world: &mut World) {
        // Factions damage each other
        // Winner gains provinces
        // Loser loses legitimacy + population
    }

    fn check_reunification(world: &mut World) {
        // Look for faction with legitimacy > 70
        // Merge other faction's resources into this faction
        // Reform nation
    }
}
```

### STEP 3: Create World Events System (Week 5-6, Day 3-10)

**File:** `crates/alalamien-engine/src/subsystems/world_events.rs`

```rust
use bevy_ecs::prelude::*;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum EventType {
    // Environmental
    Famine,
    Plague,
    DroughtBreak,
    ResourceDiscovery,
    NaturalDisaster,

    // Military
    GeneralBorn,
    TraitorDiscovered,
    SuccessfulCoup,
    FailedCoup,

    // Economic
    TradeBreakthrough,
    TradeCrisis,
    GoldRush,
    Bankruptcy,

    // Diplomatic
    PeaceProposal,
    AllianceDissenters,
    LandDisputeFlares,
    HeiressMarriage,

    // Faction
    RevolutionBrewing,
    SplinterGroup,

    // Rare
    ValidProphecy,
    HumanitysBestBorn,
    DynastyEnds,
    AstronomicalEvent,
}

#[derive(Clone, Debug)]
pub struct WorldEvent {
    pub event_id: u32,
    pub event_type: EventType,
    pub target: EventTarget,
    pub magnitude: f64,           // 0.0-1.0
    pub duration_remaining: u32,
    pub triggered_at_tick: u32,
}

#[derive(Clone, Debug)]
pub enum EventTarget {
    World,
    Nation(NationId),
    Province(ProvinceId),
    Alliance(AllianceId),
}

pub struct WorldEventPhase {
    active_events: VecDeque<WorldEvent>,
    event_rng: StdRng,
}

impl WorldEventPhase {
    pub fn new(seed: u64) -> Self {
        Self {
            active_events: VecDeque::new(),
            event_rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn execute(&mut self, world: &mut World) {
        // 1. Tick down active events
        self.tick_active_events();

        // 2. Generate new events
        self.generate_new_events(world);

        // 3. Apply event effects
        self.apply_event_effects(world);

        // 4. Remove expired events
        self.cleanup_expired_events();
    }

    fn generate_new_events(&mut self, world: &World) {
        let base_probability = 0.005; // 0.5%

        let mut query = world.query::<&Nation>();
        for nation in query.iter(world) {
            // Calculate modifiers
            let mut prob = base_probability;

            if nation.legitimacy < 30.0 {
                prob *= 1.5;
            } else if nation.legitimacy > 75.0 {
                prob *= 0.7;
            }

            // Roll probability
            if self.event_rng.gen::<f64>() < prob {
                let event_type = self.choose_event_type(nation);
                let event = WorldEvent {
                    event_id: 0, // Assign ID
                    event_type,
                    target: EventTarget::Nation(nation.id),
                    magnitude: self.event_rng.gen::<f64>(),
                    duration_remaining: 30,
                    triggered_at_tick: world.current_tick(),
                };
                self.active_events.push_back(event);
            }
        }
    }

    fn choose_event_type(&mut self, _: &Nation) -> EventType {
        // Weighted random selection
        match self.event_rng.gen_range(0..100) {
            0..=10 => EventType::Famine,
            11..=15 => EventType::Plague,
            16..=25 => EventType::GeneralBorn,
            26..=35 => EventType::TradeBreakthrough,
            36..=45 => EventType::SuccessfulCoup,
            // ... etc
            _ => EventType::Famine,
        }
    }

    fn apply_event_effects(&mut self, world: &mut World) {
        for event in self.active_events.iter() {
            match (&event.event_type, &event.target) {
                (EventType::Famine, EventTarget::Nation(nation_id)) => {
                    Self::apply_famine(world, *nation_id);
                },
                (EventType::Plague, EventTarget::Nation(nation_id)) => {
                    Self::apply_plague(world, *nation_id);
                },
                (EventType::GeneralBorn, EventTarget::Nation(nation_id)) => {
                    Self::apply_general_born(world, *nation_id);
                },
                // ... more event types
                _ => {}
            }
        }
    }

    fn apply_famine(world: &mut World, nation_id: NationId) {
        if let Some(mut economy) = world.get::<&mut EconomyState>(nation_id) {
            economy.food_production *= 0.5; // -50%
        }
    }

    fn apply_plague(world: &mut World, nation_id: NationId) {
        // Affects demographics and legitimacy
        // TODO: Implement cascade logic
    }

    fn apply_general_born(world: &mut World, nation_id: NationId) {
        if let Some(mut war) = world.get::<&mut WarState>(nation_id) {
            war.military_effectiveness *= 1.25; // +25%
        }
    }

    fn tick_active_events(&mut self) {
        for event in self.active_events.iter_mut() {
            event.duration_remaining = event.duration_remaining.saturating_sub(1);
        }
    }

    fn cleanup_expired_events(&mut self) {
        self.active_events.retain(|e| e.duration_remaining > 0);
    }
}

impl crate::core::tick::TickPhase for WorldEventPhase {
    fn name(&self) -> &str {
        "WorldEvents"
    }

    fn execute(&mut self, world: &mut World) {
        Self::execute(self, world);
    }
}
```

### STEP 4: Integration & Testing (Week 6, Day 10-14)

Write 25+ tests covering:

- Faction splitting
- Civil war resolution
- Event generation probability
- Event effects (famine, plague, general, coup, etc)
- Event determinism (same seed = same events)
- Complex interactions (famine → plague cascade)
- 10K-tick integration test

### STEP 5: Documentation (Week 7)

Create `V0.6_FACTIONS_EVENTS_SPEC.md`

---

## 🧪 HARDENING WEEK (Week 8)

### Critical Tests

```rust
#[test]
fn test_100k_ticks_with_factions_and_events() {
    // MOTHER TEST
    let mut world = WorldState::new(42);
    let mut pipeline = TickPipeline::V06();

    for _ in 0..100_000 {
        pipeline.execute(&mut world);
    }

    // Assert: No crashes, all values valid
    assert!(world.is_valid_state());
}

#[test]
fn test_determinism_100k_ticks() {
    let hash1 = run_simulation(42, 100_000);
    let hash2 = run_simulation(42, 100_000);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_multiple_collapses_no_corruption() {
    // 5+ nations collapse in different ticks
    // Assert: All handled, no data corruption
}

#[test]
fn test_event_distribution_probability() {
    // Run 100K ticks, count events
    // Assert: Frequency matches 0.5% base × modifiers
}
```

---

## 🚀 EXECUTION CHECKLIST

### V0.5 Week 1-2

- [ ] Create legitimacy.rs with skeleton
- [ ] Add legitimacy fields to Nation
- [ ] Implement war exhaustion calculation
- [ ] Implement resource stress calculation
- [ ] Implement peace recovery
- [ ] Write 10 unit tests
- [ ] Get 100 compile + tests passing

### V0.5 Week 3

- [ ] Integrate with EconomyPhase
- [ ] Integrate with WarfarePhase
- [ ] Integrate with DiplomacyPhase
- [ ] Integrate with AlliancePhase
- [ ] Write 10 integration tests
- [ ] Run 1000-tick validation test
- [ ] Get 110+ tests passing

### V0.5 Week 4

- [ ] Fine-tune formulas based on 1000-tick runs
- [ ] Balance legitimacy decay rates
- [ ] Write documentation
- [ ] Exit criteria validation
- [ ] get 115+ tests passing

### V0.6 Week 5-6

- [ ] Create factions.rs
- [ ] Implement collapse → faction splitting
- [ ] Implement civil war mechanics
- [ ] Create world_events.rs
- [ ] Implement 15 event types
- [ ] Integrate events with subsystems
- [ ] Write 25+ event/faction tests
- [ ] Get 130+ tests passing

### V0.6 Week 7

- [ ] Edge case handling
- [ ] Complex interaction tests
- [ ] 10K-tick test passing
- [ ] Documentation
- [ ] Get 135+ tests passing

### HARDENING Week 8

- [ ] 100K-tick determinism test
- [ ] Memory profiling
- [ ] Performance ceiling check
- [ ] Balance validation
- [ ] Documentation updates
- [ ] Final fixes
- [ ] Exit criteria validation

---

## 💡 SUCCESS INDICATORS

**After V0.5:**

- Legitimacy system complete
- 115+ tests passing
- Can simulate empires rising and falling naturally
- No crashes on 10K-tick runs

**After V0.6:**

- Factions working correctly
- Events firing at correct probability
- 135+ tests passing
- Can see plague cascades, coups, general births
- 10K-tick stable

**After V0.6-HARDENING:**

- 100K-tick identical runs prove determinism
- All subsystems working together
- Research-grade stability
- Ready for V1.0 release

---

**Ready?** Let me scaffold V0.5 implementation immediately. Which part should we start with first?
