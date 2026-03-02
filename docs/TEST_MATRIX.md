# Test Matrix - V0.6+

**Generated:** March 2, 2026  
**Purpose:** Comprehensive mapping of test coverage, subsystems exercised, and validation targets

---

## 📋 Test Matrix Overview

This document maps each test to:

- **Validation Target** - What behavior/property is being tested
- **Subsystems Exercised** - Which engine components are involved
- **Expected Failure Modes** - How the test would fail if broken
- **Performance Class** - Execution speed category

---

## 🧪 Unit Tests (42 tests)

### Core Types Tests (5 tests)

| Test                        | Validates                          | Subsystems             | Failure If Broken               | Perf |
| --------------------------- | ---------------------------------- | ---------------------- | ------------------------------- | ---- |
| `test_nation_id_uniqueness` | NationId generates unique UUIDs    | Core/Types             | Duplicate IDs, nation conflicts | Fast |
| `test_gdp_component`        | GDP creation & updates             | Core/Types, Economic   | NaN GDP, invalid growth rates   | Fast |
| `test_resources_operations` | Resource arithmetic (add/subtract) | Core/Types             | Resource duplication bugs       | Fast |
| `test_legitimacy_bounds`    | Legitimacy stays 0-100             | Core/Types, Legitimacy | Values <0 or >100               | Fast |
| `test_legitimacy_stability` | Stability index calculations       | Core/Types, Legitimacy | Incorrect thresholds            | Fast |

### Core World Tests (2 tests)

| Test                   | Validates                  | Subsystems | Failure If Broken     | Perf |
| ---------------------- | -------------------------- | ---------- | --------------------- | ---- |
| `test_world_creation`  | World initializes properly | Core/World | Crashes on spawn      | Fast |
| `test_nation_spawning` | Nations added to world     | Core/World | Entity spawn failures | Fast |

### Subsystem Unit Tests (35 tests)

#### Diplomacy (7 tests)

| Test                                     | Validates                      | Subsystems         | Failure If Broken         | Perf |
| ---------------------------------------- | ------------------------------ | ------------------ | ------------------------- | ---- |
| `test_diplomacy_phase_creation`          | DiplomacyPhase constructs      | Diplomacy          | Phase doesn't exist       | Fast |
| `test_diplomatic_relation_default`       | Relations initialize correctly | Diplomacy          | Wrong default values      | Fast |
| `test_friendly_hostile_checks`           | Reputation thresholds          | Diplomacy          | Incorrect diplomacy logic | Fast |
| `test_reputation_bounds`                 | Reputation stays -100 to +100  | Diplomacy          | Out-of-bounds reputation  | Fast |
| `test_reputation_modification`           | Reputation changes work        | Diplomacy          | Frozen relations          | Fast |
| `test_shared_enemies_increase_alignment` | Common foes boost relations    | Diplomacy, Warfare | No threat alignment       | Fast |
| `test_diplomacy_world_integration`       | Full world execution           | Diplomacy, All     | Pipeline crashes          | Fast |

#### Legitimacy (6 tests)

| Test                                 | Validates                | Subsystems      | Failure If Broken     | Perf |
| ------------------------------------ | ------------------------ | --------------- | --------------------- | ---- |
| `test_legitimacy_respects_ceiling`   | Max legitimacy = 100     | Legitimacy      | Values exceed 100     | Fast |
| `test_legitimacy_respects_floor`     | Min legitimacy = 0       | Legitimacy      | Negative legitimacy   | Fast |
| `test_legitimacy_default`            | Default initialization   | Legitimacy      | Wrong starting values | Fast |
| `test_legitimacy_is_stable`          | Stability flag works     | Legitimacy      | Always unstable       | Fast |
| `test_legitimacy_modification_chain` | Sequential changes stack | Legitimacy      | Order-dependent bugs  | Fast |
| `test_legitimacy_world_integration`  | Full world execution     | Legitimacy, All | Pipeline crashes      | Fast |

#### Alliance, Warfare, Factions, Events (8 tests)

| Test                              | Validates                | Subsystems    | Failure If Broken  | Perf |
| --------------------------------- | ------------------------ | ------------- | ------------------ | ---- |
| `test_alliance_module_exists`     | AlliancePhase constructs | Alliance      | Module missing     | Fast |
| `test_alliance_world_integration` | Alliance execution       | Alliance, All | Pipeline crashes   | Fast |
| `test_warfare_phase_creation`     | WarfarePhase constructs  | Warfare       | Module missing     | Fast |
| `test_simple_war_state_tracking`  | War state management     | Warfare       | Wrong war tracking | Fast |
| `test_factions_module_exists`     | FactionPhase constructs  | Factions      | Module missing     | Fast |
| `test_factions_world_integration` | Faction execution        | Factions, All | Pipeline crashes   | Fast |
| `test_events_module_exists`       | EventPhase constructs    | Events        | Module missing     | Fast |
| `test_events_world_integration`   | Event execution          | Events, All   | Pipeline crashes   | Fast |

---

## 🔗 Integration Tests (37 tests)

### Alliance-Warfare Interaction (3 tests)

| Test                                     | Validates                        | Subsystems                   | Failure If Broken    | Perf |
| ---------------------------------------- | -------------------------------- | ---------------------------- | -------------------- | ---- |
| `test_alliances_provide_support`         | Allies help in wars              | Alliance, Warfare            | No alliance effect   | Med  |
| `test_shared_enemy_strengthens_alliance` | Common threats boost cohesion    | Alliance, Diplomacy, Warfare | No threat bonding    | Med  |
| `test_alliance_cascading_wars`           | Defensive alliances trigger wars | Alliance, Warfare, Diplomacy | No defensive trigger | Med  |

### Civil War Cascade (5 tests)

| Test                                           | Validates                       | Subsystems                    | Failure If Broken | Perf |
| ---------------------------------------------- | ------------------------------- | ----------------------------- | ----------------- | ---- |
| `test_nation_collapses_into_factions`          | Zero legitimacy → civil war     | Legitimacy, Factions          | No collapse       | Med  |
| `test_civil_war_factions_fight_each_other`     | Factions war each other         | Factions, Warfare             | No internal war   | Med  |
| `test_no_double_collapse`                      | Prevent duplicate faction spawn | Factions                      | Infinite factions | Med  |
| `test_faction_inherits_proportional_resources` | Resource distribution           | Factions, Economic            | Unfair splits     | Med  |
| `test_legitimacy_spiral_reaches_crisis`        | Compounding stress → collapse   | Legitimacy, Warfare, Economic | No death spiral   | Med  |

### Diplomatic Scenarios (6 tests)

| Test                                       | Validates                     | Subsystems              | Failure If Broken      | Perf |
| ------------------------------------------ | ----------------------------- | ----------------------- | ---------------------- | ---- |
| `test_alliance_dissolution_under_pressure` | Stress breaks alliances       | Alliance, Legitimacy    | Alliances too stable   | Med  |
| `test_diplomatic_isolation_effects`        | Isolation harms stability     | Diplomacy, Legitimacy   | No isolation penalty   | Med  |
| `test_neutral_mediator_role`               | Third-party mediation         | Diplomacy, Intervention | No mediation           | Med  |
| `test_peace_treaty_impact`                 | Treaties end wars             | Diplomacy, Warfare      | Peace treaties ignored | Med  |
| `test_reputation_recovery_trajectory`      | Reputation recovers over time | Diplomacy               | Frozen reputation      | Med  |
| `test_coalition_formation_scenario`        | Multi-nation coalitions form  | Alliance, Diplomacy     | No coalitions          | Med  |

### Economic-Military Interaction (2 tests)

| Test                                     | Validates                   | Subsystems        | Failure If Broken  | Perf |
| ---------------------------------------- | --------------------------- | ----------------- | ------------------ | ---- |
| `test_economic_capacity_limits_military` | Economy constrains military | Economic, Warfare | Unlimited military | Med  |
| `test_military_action_affects_economy`   | Wars drain economy          | Warfare, Economic | No war costs       | Med  |

### Economic Scenarios (4 tests)

| Test                                   | Validates             | Subsystems           | Failure If Broken    | Perf |
| -------------------------------------- | --------------------- | -------------------- | -------------------- | ---- |
| `test_economic_crisis_triggers_unrest` | Deficit → instability | Economic, Legitimacy | No economic pressure | Med  |
| `test_economic_recovery_after_war`     | Post-war recovery     | Economic, Legitimacy | No recovery          | Med  |
| `test_resource_bottleneck_scenarios`   | Scarcity handling     | Economic             | Resource bugs        | Med  |
| `test_trade_network_formation`         | Trade links form      | Economic, Diplomacy  | No trade             | Med  |

### Game Depth Scenarios (8 tests)

| Test                                   | Validates                     | Subsystems                    | Failure If Broken  | Perf |
| -------------------------------------- | ----------------------------- | ----------------------------- | ------------------ | ---- |
| `test_war_erodes_legitimacy_over_time` | War → legitimacy decay        | Warfare, Legitimacy           | No war exhaustion  | Med  |
| `test_peace_restores_legitimacy`       | Peace → legitimacy recovery   | Legitimacy                    | No recovery        | Med  |
| `test_deficit_causes_legitimacy_drain` | Economic stress → instability | Economic, Legitimacy          | No deficit penalty | Med  |
| `test_combined_stress_compounds`       | Multiple stressors stack      | Warfare, Economic, Legitimacy | No compounding     | Med  |
| `test_diplomatic_world_is_dynamic`     | Relations evolve              | Diplomacy                     | Static world       | Med  |
| `test_average_legitimacy_responds`     | Global metrics change         | Legitimacy, All               | Frozen state       | Med  |
| `test_war_exhaustion_increments`       | War exhaustion grows          | Warfare                       | No exhaustion      | Med  |
| `test_war_exhaustion_decays_at_peace`  | Exhaustion recovers           | Warfare                       | No decay           | Med  |

### Multi-Nation Scenarios (6 tests)

| Test                                        | Validates            | Subsystems           | Failure If Broken   | Perf |
| ------------------------------------------- | -------------------- | -------------------- | ------------------- | ---- |
| `test_5_nation_world_stability`             | 5-nation stability   | All                  | System crashes      | Med  |
| `test_10_nation_world_long_run`             | 10-nation long-term  | All                  | Stability breaks    | Slow |
| `test_global_legitimacy_and_gdp_drift`      | Metrics stay valid   | Economic, Legitimacy | NaN/infinite values | Med  |
| `test_coalition_formation_scenario`         | Coalition mechanics  | Alliance, Diplomacy  | No coalitions       | Med  |
| `test_5_nation_world_stability_and_metrics` | Consistency checks   | All                  | Nation vanishing    | Med  |
| `test_10_nation_world_long_run_dynamics`    | Diplomatic evolution | Diplomacy, Alliance  | Frozen relations    | Slow |

### Warfare-Diplomacy Interaction (4 tests)

| Test                                      | Validates                  | Subsystems              | Failure If Broken      | Perf |
| ----------------------------------------- | -------------------------- | ----------------------- | ---------------------- | ---- |
| `test_war_exhaustion_grows_via_pipeline`  | War exhaustion integration | Warfare                 | No exhaustion tracking | Med  |
| `test_war_exhaustion_decays_at_peace`     | Peace recovery             | Warfare                 | No decay               | Med  |
| `test_war_nation_loses_legitimacy_faster` | War vs peace divergence    | Warfare, Legitimacy     | No difference          | Med  |
| `test_war_diplomacy_world_remains_stable` | Combined stability         | Warfare, Diplomacy, All | System crashes         | Med  |

---

## 🚀 Advanced Tests (51 tests)

### Chaos & Fuzz Testing (12 tests)

| Test                                        | Validates                             | Subsystems         | Failure If Broken         | Perf |
| ------------------------------------------- | ------------------------------------- | ------------------ | ------------------------- | ---- |
| `test_random_seed_variance`                 | Different seeds produce valid outputs | Core, All          | Determinism broken        | Med  |
| `test_rapid_fire_ticks`                     | High-frequency execution              | Core/Tick          | Race conditions           | Med  |
| `test_batch_tick_equivalence`               | Batch = individual ticks              | Core/Tick          | Tick order bugs           | Med  |
| `test_multi_seed_comparison`                | Multi-seed reproducibility            | Core/Deterministic | Non-deterministic         | Med  |
| `test_sequential_execution_stability`       | Sequential consistency                | Core/Tick          | Order-dependent bugs      | Med  |
| `test_world_doesnt_crash_on_zero_resources` | Zero resource handling                | Economic           | Division by zero          | Fast |
| `test_extreme_nation_counts`                | 1 vs 50 nation scalability            | All                | Scaling issues            | Slow |
| `test_random_nation_counts_stability`       | Variable nation stability             | All                | Count-dependent bugs      | Med  |
| `test_stress_test_many_nations`             | 50-nation stress                      | All                | Memory/performance issues | Slow |
| + 3 additional edge cases                   | Boundary conditions                   | All                | Edge case crashes         | Med  |

### Determinism Tests (6 tests)

| Test                                    | Validates              | Subsystems         | Failure If Broken       | Perf |
| --------------------------------------- | ---------------------- | ------------------ | ----------------------- | ---- |
| `test_determinism_1000_ticks_v0_35`     | 1k-tick reproduction   | Core/Tick, All     | Non-deterministic       | Slow |
| `test_determinism_500_ticks`            | 500-tick consistency   | Core/Tick, All     | Drift over time         | Med  |
| `test_100_tick_determinism`             | 100-tick exact match   | Core/Tick, All     | Early divergence        | Fast |
| `test_multi_seed_consistency`           | Seed consistency       | Core/Deterministic | RNG broken              | Fast |
| `test_determinism_same_seed_same_ticks` | Seed→Ticks determinism | Core/Tick          | State drift             | Fast |
| + 1 additional test                     | Historical format      | Core/State         | Version incompatibility | Med  |

### Edge Cases (7 tests)

| Test                           | Validates            | Subsystems         | Failure If Broken    | Perf |
| ------------------------------ | -------------------- | ------------------ | -------------------- | ---- |
| `test_single_nation_stability` | 1-nation world       | All                | Minimum count bug    | Fast |
| `test_zero_seed_handling`      | Seed=0 handling      | Core/Deterministic | Zero seed crash      | Fast |
| `test_max_seed_handling`       | Max u64 seed         | Core/Deterministic | Overflow bugs        | Fast |
| `test_rapid_ticks_stress`      | 1000-tick rapid fire | Core/Tick          | Performance collapse | Med  |
| `test_many_nations_stability`  | High nation count    | All                | Scaling issues       | Med  |
| `test_long_game_session`       | 1000+ tick marathon  | All                | Memory leaks         | Slow |
| `test_max_seed_handling`       | Boundary conditions  | Core/Deterministic | Edge crashes         | Fast |

### Performance Benchmarks (4 tests)

| Test                               | Validates            | Subsystems | Failure If Broken     | Perf |
| ---------------------------------- | -------------------- | ---------- | --------------------- | ---- |
| `test_performance_single_nation`   | 1-nation baseline    | All        | Baseline regression   | Med  |
| `test_performance_5_nations`       | 5-nation throughput  | All        | Linear scaling broken | Med  |
| `test_performance_10_nations`      | 10-nation throughput | All        | Quadratic scaling     | Med  |
| `test_performance_scaling_10_vs_5` | Scaling metric       | All        | Non-linear growth     | Med  |

### Quality Metrics (5 tests)

| Test                                | Validates              | Subsystems  | Failure If Broken | Perf |
| ----------------------------------- | ---------------------- | ----------- | ----------------- | ---- |
| `test_economy_remains_stable`       | Economic stability     | Economic    | Runaway inflation | Med  |
| `test_no_infinite_loops`            | Deadlock prevention    | All         | Hangs             | Fast |
| `test_game_feels_dynamic`           | Event generation       | Events      | Static world      | Med  |
| `test_population_does_not_collapse` | Population persistence | Demographic | Mass extinction   | Med  |
| `test_game_fairness_no_favoritism`  | Fair treatment         | All         | Bias bugs         | Med  |

### Regression Tests (7 tests)

| Test                                    | Validates                | Subsystems | Failure If Broken   | Perf      |
| --------------------------------------- | ------------------------ | ---------- | ------------------- | --------- |
| `test_v0_6_pipeline_exists`             | Pipeline integrity       | Core/Tick  | Pipeline broken     | Fast      |
| `test_world_state_persists_ticks`       | State persistence        | Core/World | State loss          | Med       |
| `test_nation_spawning_preserves_state`  | Spawn safety             | Core/World | State corruption    | Fast      |
| `test_multiple_nations_coexist`         | Multi-nation coexistence | All        | Nation conflicts    | Fast      |
| `test_multiple_executions_additive`     | Cumulative ticks         | Core/Tick  | Tick reset bugs     | Fast      |
| `test_determinism_same_seed_same_ticks` | Historical determinism   | Core/Tick  | Regression          | Fast      |
| `test_no_panic_on_extended_execution`   | 1M-tick stability        | All        | Memory/panic issues | Very Slow |

### Subsystem Performance (8 tests)

| Test                                    | Validates           | Subsystems | Failure If Broken      | Perf |
| --------------------------------------- | ------------------- | ---------- | ---------------------- | ---- |
| `test_diplomacy_subsystem_performance`  | Diplomacy timing    | Diplomacy  | Performance regression | Med  |
| `test_legitimacy_subsystem_performance` | Legitimacy timing   | Legitimacy | Performance regression | Med  |
| `test_warfare_subsystem_performance`    | Combat timing       | Warfare    | Performance regression | Med  |
| `test_economic_subsystem_performance`   | Economic timing     | Economic   | Performance regression | Med  |
| `test_alliance_subsystem_performance`   | Alliance timing     | Alliance   | Performance regression | Med  |
| `test_events_subsystem_performance`     | Events timing       | Events     | Performance regression | Med  |
| `test_combined_subsystem_scaling`       | Combined scaling    | All        | Non-linear growth      | Med  |
| + 1 additional test                     | Subsystem profiling | All        | Bottlenecks            | Med  |

---

## 🧬 Inline Tests (106 tests)

### Core Engine (32 tests)

#### Deterministic RNG (3 tests)

| Test                        | Validates                 | Expected Failure    | Perf |
| --------------------------- | ------------------------- | ------------------- | ---- |
| `test_deterministic_output` | Same seed = same sequence | Different outputs   | Fast |
| `test_custom_range`         | Range clamping works      | Out-of-range values | Fast |
| `test_f64_range`            | Float range generation    | Range violations    | Fast |

#### Province Graph (6 tests)

| Test                      | Validates               | Expected Failure  | Perf |
| ------------------------- | ----------------------- | ----------------- | ---- |
| `test_add_border`         | Border addition         | Border not added  | Fast |
| `test_remove_border`      | Border removal          | Border persists   | Fast |
| `test_get_neighbors`      | Neighbor queries        | Wrong neighbors   | Fast |
| `test_border_count`       | Border counting         | Wrong counts      | Fast |
| `test_multiple_neighbors` | Multi-neighbor handling | Missing neighbors | Fast |
| `test_find_isolated`      | Isolation detection     | Wrong isolation   | Fast |

#### State Management (1 test)

| Test                          | Validates                 | Expected Failure       | Perf |
| ----------------------------- | ------------------------- | ---------------------- | ---- |
| `test_state_hash_determinism` | State hashing consistency | Non-deterministic hash | Fast |

#### Tick Pipeline (15 tests)

| Test                                     | Validates              | Expected Failure       | Perf |
| ---------------------------------------- | ---------------------- | ---------------------- | ---- |
| `test_pipeline_execution`                | Phase execution order  | Wrong order            | Fast |
| `test_v0_5_pipeline_legitimacy_phase`    | Legitimacy integration | Phase missing          | Fast |
| `test_v0_4_alliance_dissolution`         | Alliance lifecycle     | Alliance bugs          | Med  |
| `test_determinism_500_ticks`             | 500-tick consistency   | State drift            | Med  |
| `test_determinism_500_ticks_v0_2_replay` | Legacy format          | Format incompatibility | Med  |
| `test_1000_ticks_v0_4_with_alliances`    | Extended alliance test | Long-term bugs         | Slow |
| `test_determinism_1000_ticks_v0_35`      | 1k-tick determinism    | Non-deterministic      | Slow |
| + 8 additional pipeline tests            | Tick mechanics         | Pipeline bugs          | Med  |

### Subsystems (103 tests)

Complete breakdown of all inline subsystem tests across:

- AI Advanced (5 tests) - Strategic AI behavior
- AI Basic (3 tests) - Personality archetypes
- Alliance (6 tests) - Alliance mechanics
- Alliance Dataset (4 tests) - Data integrity
- Combat (4 tests) - Battle resolution
- Demographic (2 tests) - Population dynamics
- Diplomacy (7 tests) - Diplomatic relations
- Economic (4 tests) - Production & resources
- Events (8 tests) - Random events
- Factions (10 tests) - Civil war mechanics
- Intervention (3 tests) - Mediation
- Legitimacy (14 tests) - Stability mechanics
- Occupation (1 test) - Territory control

_(Detailed breakdowns available in source files)_

---

## 📊 Test Coverage Summary

### By Validation Type

| Type                       | Count | Purpose                     |
| -------------------------- | ----- | --------------------------- |
| **Functional Correctness** | 168   | Validates behavior logic    |
| **Performance Benchmarks** | 12    | Tracks execution speed      |
| **Determinism Validation** | 15    | Ensures reproducibility     |
| **Regression Prevention**  | 18    | Guards against bugs         |
| **Edge Case Protection**   | 14    | Boundary condition safety   |
| **Integration Scenarios**  | 12    | Multi-subsystem interaction |

### By Performance Class

| Class         | Count | Execution Time              |
| ------------- | ----- | --------------------------- |
| **Fast**      | ~150  | <10ms                       |
| **Medium**    | ~70   | 10-50ms                     |
| **Slow**      | ~13   | 50-200ms                    |
| **Very Slow** | 3     | >200ms (ignored by default) |

### By Subsystem Coverage

| Subsystem  | Unit | Integration | Advanced | Total |
| ---------- | ---- | ----------- | -------- | ----- |
| Diplomacy  | 7    | 9           | 2        | 18    |
| Legitimacy | 6    | 8           | 2        | 16    |
| Alliance   | 2    | 7           | 2        | 11    |
| Warfare    | 2    | 8           | 2        | 12    |
| Factions   | 2    | 5           | 1        | 8     |
| Economic   | 4    | 6           | 2        | 12    |
| Events     | 2    | 2           | 2        | 6     |
| AI         | 8    | 2           | 1        | 11    |
| Core       | 9    | 0           | 38       | 47    |

---

## 🎯 Test Quality Metrics

### Coverage Priorities

**Critical Path Tests (Must Never Break):**

- All determinism tests
- Core type bounds (legitimacy, resources, GDP)
- World creation & tick pipeline
- State persistence

**High-Value Integration Tests:**

- Civil war cascade (end-to-end complex mechanic)
- Game depth scenarios (behavioral validation)
- Alliance cascading wars (multi-subsystem interaction)

**Performance-Critical Tests:**

- Subsystem performance suite
- Scaling tests (5 vs 10 nations)
- Long-run stability (1000+ ticks)

### Test Effectiveness

**Strong Tests (High Signal):**

- `test_civil_war_cascade` - Validates entire faction spawn pipeline
- `test_combined_stress_compounds` - Multi-factor interaction
- `test_legitimacy_spiral_reaches_crisis` - End-to-end death spiral
- `test_alliance_cascading_wars` - Defensive alliance trigger

**Smoke Tests (Catch Most Breaks):**

- `test_world_creation` - Basic engine viability
- `test_100_tick_determinism` - Fast determinism validation
- `test_5_nation_world_stability` - Multi-nation baseline

---

## 🔮 Future Test Recommendations

### Gaps to Fill

1. **Save/Load System** (15+ tests needed)
   - Round-trip serialization
   - Version migration
   - Partial state recovery

2. **Espionage Mechanics** (10+ tests needed)
   - Intelligence gathering
   - Covert operations
   - Counter-intelligence

3. **Nuclear Weapons** (8+ tests needed)
   - Deterrence mechanics
   - MAD scenarios
   - Fallout effects

4. **Vassal States** (8+ tests needed)
   - Protectorate formation
   - Puppet state mechanics
   - Liberation scenarios

### Test Infrastructure Improvements

- Property-based testing (proptest integration)
- Mutation testing (verify test quality)
- Code coverage metrics (cargo-llvm-cov)
- Test dependency graph visualization

---

**Matrix Version:** 1.0  
**Last Updated:** March 2, 2026  
**Total Tests Mapped:** 236  
**Maintainer:** Development Team
