# AI Agent Context Guide

This repo is a Rust-based geopolitical simulation engine with real-world geodata. Use this file to get up to speed quickly.

## Quick Start (Read Order)

1. docs/TEST_COVERAGE_SUMMARY.md (test status at a glance)
2. docs/TEST_COVERAGE_REPORT.md (comprehensive test breakdown)
3. docs/versions/VERSION_INDEX.md
4. PROJECT_STATUS.md
5. docs/DOCUMENTATION_INDEX.md
6. docs/ROADMAP.md (feature roadmap)
7. docs/versions/V0.1_FINAL_SUMMARY.md (historical baseline)

## Current Status (as of March 2, 2026)

**Version: V0.6+**

- ✅ **220 tests passing** (100% pass rate, 1.75s execution)
- ✅ **11 subsystems implemented** (diplomacy, alliance, warfare, legitimacy, factions, events, AI, combat, economic, intervention, occupation)
- ✅ **3-layer test architecture** (unit/integration/advanced)
- ✅ **Comprehensive test coverage** (functional, performance, determinism, regression, edge cases)
- ✅ **Deterministic execution** (multi-seed reproducibility validated)
- ✅ **Real geodata integration** (177 nations from Natural Earth)
- ✅ **API server operational**
- ✅ **Desktop build functional**

## Key Code Entry Points

- **Engine core:** crates/alalamien-engine/src/
  - core/tick.rs (V0.6 tick pipeline with 11 subsystems)
  - core/province_graph.rs (adjacency graph)
  - core/world.rs (WorldState)
  - core/types.rs (ECS components: Nation, Province, Alliance, DiplomaticRelation, etc.)
  - core/deterministic.rs (deterministic RNG)
  - subsystems/ (11 subsystems: diplomacy, alliance, warfare, legitimacy, factions, events, AI advanced/basic, combat, economic, demographic, intervention, occupation)
- **Tests:** crates/alalamien-engine/src/tests/
  - unit/ (42 tests: core types, subsystems)
  - integration/ (24 tests: diplomatic, economic, warfare scenarios)
  - advanced/ (51 tests: chaos/fuzz, performance, determinism, quality metrics)
  - fixtures/ (TestWorldBuilder, test utilities)
- **API:** crates/alalamien-api/src/
- **Desktop:** crates/alalamien-desktop/src-tauri/
- **Geodata:** assets/data/ and crates/alalamien-engine/src/game/geodata.rs

## Common Commands

- **All Tests:** `cargo test --lib` (220 tests, ~1.75s)
- **Unit Tests:** `cargo test --lib tests::unit::`
- **Integration Tests:** `cargo test --lib tests::integration::`
- **Advanced Tests:** `cargo test --lib tests::advanced::`
- **Long Tests:** `cargo test --lib -- --ignored` (100k-tick determinism)
- **Run API:** `cargo run --package alalamien-api`
- **Build Desktop:** `cargo build --package alalamien-desktop`
- **With Output:** `cargo test --lib -- --nocapture`

## Test Architecture

**3-Layer Structure:**

1. **Unit Tests (42)** - Component-level validation
   - Core types (GDP, resources, legitimacy)
   - Subsystem phases (diplomacy, alliance, warfare, etc.)

2. **Integration Tests (24)** - Subsystem interactions
   - Alliance-warfare scenarios
   - Diplomatic scenarios (isolation, mediation, treaties)
   - Economic-military interactions
   - Multi-nation scenarios

3. **Advanced Tests (51)** - Quality assurance
   - Chaos/Fuzz (12 tests): random configs, stress testing
   - Determinism (6 tests): multi-seed reproducibility
   - Edge Cases (7 tests): boundary conditions
   - Performance (4 tests): scaling benchmarks
   - Quality Metrics (5 tests): game balance validation
   - Regression (7 tests): historical issue prevention
   - Subsystem Performance (8 tests): per-subsystem profiling

**Test Documentation:**

- docs/TEST_COVERAGE_REPORT.md (comprehensive analysis)
- docs/TEST_COVERAGE_SUMMARY.md (quick reference)
- docs/TEST_ARCHITECTURE_SUMMARY.md (test design)
- docs/TEST_FIXTURE_QUICK_REFERENCE.md (TestWorldBuilder guide)

## Notes

- All 220 tests pass with 100% success rate
- Determinism validated across multiple seeds and long runs
- Test execution is fast (<2s for full suite)
- Do not delete or rewrite version docs; append updates with dates

---

## V0.6+ Subsystems

### Core Systems

1. **Diplomacy** (14 tests)
   - Diplomatic relations tracking
   - Reputation system
   - Threat alignment
   - Alliance proposals

2. **Alliance Management** (12 tests)
   - Alliance formation & dissolution
   - Cohesion decay mechanics
   - Doctrine application (defensive, offensive)
   - Multi-alliance support

3. **Warfare** (Combat + Warfare subsystems)
   - War declarations
   - Battle resolution
   - Casualty calculation
   - War exhaustion

4. **Legitimacy & Stability** (14 tests)
   - Legitimacy tracking (0-100)
   - War exhaustion calculation
   - Economic deficit stress
   - Alliance burden
   - Stability thresholds

5. **Factions & Civil War** (10 tests)
   - Faction spawning from low legitimacy
   - Civil war mechanics
   - Army splitting (deterministic)
   - Province redistribution
   - Resource distribution

6. **Events System** (8 tests)
   - Random event generation
   - Event categories (economic, military, diplomatic, natural disasters, social)
   - Event duration & effects
   - Legitimacy impact

7. **AI Systems** (8 tests)
   - AI Advanced: strategic decision-making, alliance proposals, memory-based aggression
   - AI Basic: personality types (aggressive, defensive, balanced), emergency overrides

8. **Economic System** (4 tests)
   - Resource production
   - Production chains (iron→military, oil→logistics)
   - GDP calculation

9. **Demographics** (2 tests)
   - Population growth/decline
   - Food surplus calculation

10. **Intervention** (3 tests)
    - Neighbor detection
    - Intervention mechanics

11. **Occupation** (1 test)
    - Territory control mechanics

### Border Data & Province Graph

**Steps to extract and use border data:**

1. **Extract borders from shapefiles:**

```bash
# Requires: pip install pyshp
python scripts/extract_borders.py
```

This creates `src/game/scenarios/borders.json` with country adjacency data.

2. **Generate Rust border module:**

```bash
python scripts/load_borders.py
```

This creates `crates/alalamien-engine/src/game/borders.rs` with pre-computed border data.

3. **Integrate with ProvinceGraph:**

- Add `pub mod borders;` to `src/game/mod.rs`
- In `WorldState::from_geodata()`, after spawning nations, use border data to populate the `ProvinceGraph`
- Call `province_graph.add_border(province_a, province_b)` for each border relationship

**Why this matters for V0.2:**

- Trade routes (TradePhase) need to know which provinces are adjacent
- Logistics (LogisticsPhase) calculates supply lines based on graph connectivity
- Demographics can simulate migration along borders
- Blockades affect provinces based on their connectivity

### V0.2 Components Added

New types in `core/types.rs`:

- `OwnedBy` - Links provinces to nations
- `TradeRoute` - Represents resource flows
- `TradeRouteId` - Unique trade route identifier
- `ResourceDeficit` - Tracks shortages

New subsystems:

- `TradePhase` - Distributes resources via trade routes
- `LogisticsPhase` - Manages supply lines and attrition
- `StabilityPhase` - Legitimacy, protests, civil war escalation 🆕

**Status:**

- ✅ 220/220 tests passing
- ✅ 11-subsystem pipeline (Diplomacy, Alliance, Warfare, Legitimacy, Factions, Events, AI, Combat, Economic, Demographic, Intervention, Occupation)
- ✅ Comprehensive test coverage (unit, integration, advanced)
- ✅ Determinism validated (multi-seed reproducibility)
- ✅ Performance benchmarked (per-subsystem profiling)
- ⏳ Border data extraction complete, integration pending

### Diplomacy & Alliance System (V0.6)

The diplomacy system manages international relations:

1. **Diplomatic Relations:** Tracks reputation (-100 to +100) between nations
2. **Threat Alignment:** Shared enemies strengthen bonds
3. **Alliance Formation:** Based on alignment, reputation, and strategic needs
4. **Alliance Cohesion:** Decays over time, strengthened by shared wars
5. **Alliance Dissolution:** Automatic when cohesion drops below threshold

**Key files:**

- `crates/alalamien-engine/src/subsystems/diplomacy.rs`
- `crates/alalamien-engine/src/subsystems/alliance.rs`
- `crates/alalamien-engine/src/subsystems/alliance_dataset.rs`

### Warfare & Combat System (V0.6)

The warfare system manages military conflicts:

1. **War Declarations:** Track aggressor vs defender
2. **Battle Resolution:** Deterministic combat outcomes
3. **Casualty Calculation:** Based on military strength
4. **War Exhaustion:** Affects legitimacy over time
5. **Alliance Involvement:** Allies can join wars

**Key files:**

- `crates/alalamien-engine/src/subsystems/warfare.rs`
- `crates/alalamien-engine/src/subsystems/combat.rs`

### Legitimacy & Stability System (V0.6)

The legitimacy system models internal nation pressure:

1. **Legitimacy Tracking:** 0-100 range with stability thresholds
2. **War Exhaustion:** Active wars drain legitimacy
3. **Economic Stress:** Deficits reduce legitimacy
4. **Alliance Burden:** Too many alliances strain legitimacy
5. **Faction Spawning:** Low legitimacy triggers factions
6. **Civil War:** Critical legitimacy (<15) triggers civil war

**Key files:**

- `crates/alalamien-engine/src/subsystems/legitimacy.rs`
- `crates/alalamien-engine/src/subsystems/factions.rs`

### Events System (V0.6)

The events system adds dynamic unpredictability:

1. **Event Categories:** Economic, military, diplomatic, natural disasters, social
2. **Probability-Based Triggering:** Uses deterministic RNG
3. **Event Effects:** Immediate and duration-based
4. **Stacking Modifiers:** Multiple events can affect a nation
5. **Legitimacy Impact:** Events can boost or harm stability

**Key files:**

- `crates/alalamien-engine/src/subsystems/events.rs`

### AI Systems (V0.6)

The AI systems control nation behavior:

1. **AI Advanced:** Strategic decision-making, alliance proposals, threat assessment, memory-based behavior
2. **AI Basic:** Personality archetypes (aggressive, defensive, balanced), emergency overrides
3. **Deterministic:** All AI decisions use deterministic RNG for reproducibility

**Key files:**

- `crates/alalamien-engine/src/subsystems/ai_advanced.rs`
- `crates/alalamien-engine/src/subsystems/ai_basic.rs`

---

**Current V0.6 Tick Pipeline:**

Phase execution order in `core/tick.rs`:

1. DiplomacyPhase (relations, reputation, threat alignment)
2. AlliancePhase (cohesion, dissolution)
3. AIAdvancedPhase (strategic decisions)
4. AIBasicPhase (personality-driven behavior)
5. WarfarePhase (war progression)
6. CombatPhase (battle resolution)
7. LegitimacyPhase (stability, war exhaustion)
8. FactionCivilWarPhase (faction spawning, civil war)
9. EventPhase (random events)
10. EconomicPhase (resource production)
11. DemographicPhase (population changes)
12. InterventionPhase (mediation)
13. OccupationPhase (territory control)

This order ensures logical causality and determinism.

## Project Summary

This is a world-scale strategy game simulation engine. The core loop is deterministic and data-driven, using Natural Earth geodata to seed nations and their initial stats. The engine is built around an ECS architecture (bevy_ecs), with modular subsystems for population, resources, and economic output. API and desktop targets are thin wrappers around the same engine.

## How The Game Works (Conceptual Model)

- World is composed of Nations and Provinces.
- Each Province has population, resources, infrastructure, and an owner Nation.
- Each tick executes ordered phases:
  1. Demographics: population changes based on food surplus
  2. Economics: resources and GDP update based on production and infrastructure
- Stability constraints prevent NaNs and runaway growth.
- Determinism is required: same seed and same actions produce the same world state hash.

## Roadmap (High-Level)

The detailed roadmap lives in docs/versions/ and docs/ROADMAP.md. Use this as the guiding summary for future work.

### v0.1 (Completed ✅)

- Core ECS simulation engine
- Deterministic tick pipeline
- Population and resource systems
- Province adjacency graph
- 500-tick stability test
- Save/load + state hashing
- API server and desktop wrapper
- Real geodata integration (177 nations)

### v0.2-v0.5 (Completed ✅)

- Trade routes & logistics
- Stability system (legitimacy)
- Economic production chains
- Diplomatic relations
- Alliance management
- Military units & combat
- War declarations & warfare
- Events system

### v0.6 (Current ✅)

- ✅ Comprehensive test architecture (220 tests)
- ✅ 11 subsystems fully implemented
- ✅ Factions & civil war mechanics
- ✅ AI systems (advanced & basic)
- ✅ Intervention & mediation
- ✅ Occupation mechanics
- ✅ Performance benchmarking
- ✅ Test coverage documentation

### v0.7+ (Planned)

- Save/load system tests
- Vassal state mechanics
- Nuclear weapon mechanics
- Espionage systems
- API endpoint validation
- Frontend map visualization
- Multiplayer synchronization

### v1.0 (Target)

- Full gameplay loop: diplomacy, economy, warfare, and events
- AI nation behavior and strategy (DONE)
- Save/load for long campaigns
- Polished UI and stability targets
- 300+ tests with comprehensive coverage
