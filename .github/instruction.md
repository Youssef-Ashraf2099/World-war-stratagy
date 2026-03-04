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

## Current Status (as of March 4, 2026)

**Version: V0.7 Complete ✅ | V0.8 In Progress — Phase 0 + 0.5 Done ✅**

### V0.7 Pre-UI Hardening (Complete)

- ✅ **383 tests passing** (100% pass rate, 0 failures)
- ✅ **6 integrated subsystems with notifications** (warfare, nuclear, vassalage, espionage, economic, alliances)
- ✅ **19-phase deterministic tick pipeline**
- ✅ **Full notification system** across all subsystems
- ✅ **Comprehensive test architecture** (unit/integration/system/regression)
- ✅ **Real geodata integration** (177 nations from Natural Earth)
- ✅ **REST API server operational**
- ✅ **Headless simulation engine** ready for UI layer

### V0.8 UI Implementation — Phase Progress

#### Phase 0 ✅ (Complete) — Main Menu

- ✅ **Bevy 0.14.2** native desktop renderer running
- ✅ **Main menu UI** — 5 buttons: NEW GAME, LOAD GAME, SETTINGS, CREDITS, QUIT
- ✅ **Modern minimal dark theme** — `srgb(0.1, 0.1, 0.18)` base, cyan title, ocean-blue accents
- ✅ **Audio system** — hover.mp3 + click.mp3 (MP3 via `bevy_audio` mp3 feature)
- ✅ **Window icon** — gameIcon.ico applied via `WinitWindows` + `include_bytes!` (PostStartup)
- ✅ **Background gradient effects** — cyan top, red bottom, border accent lines

#### Phase 0.5 ✅ (Complete) — Multi-Screen Navigation & SOLID Architecture

- ✅ **Credits screen** — "Youssef Ashraf" credit, BACK TO MENU button
- ✅ **Load Game placeholder** — Phase 1 notice, BACK TO MENU button
- ✅ **State machine** — `AppState { Menu, LoadGame, Credits, Loading, Game, Settings }`
- ✅ **OnEnter/OnExit lifecycle** — `ScreenUI` marker component, `despawn_recursive` cleanup
- ✅ **SOLID architecture** — single-responsibility .rs files (navigation, ui_manager, button, background, icon)
- ⏸️ **Settings screen** — explicitly postponed to future phase

#### Phase 1 📋 (Next) — Map Rendering

- 📋 **Vector-based map rendering** (infinite zoom, Web Mercator projection)
- 📋 Real geodata nations rendered on canvas

#### Phase 2 📋 — Four-Panel HUD

- 📋 **Four-panel HUD layout** (top bar, left panel, right log, bottom controls)

#### Phase 3 📋 — API Bridge

- 📋 **REST API bridge** (HTTP client querying alalamien-api)

#### Phase 4 📋 — Developer Tools

- 📋 **Developer tools crate** (separate `alalamien-dev-tools/`, feature-gated)

#### Phase 5 📋 — VFX & Polish

- 📋 **VFX & Polish** (particles, animations, smooth transitions)
- 📋 **Target: 60+ FPS** with 100+ provinces visible

## Key Code Entry Points

- **UI (V0.8+):** crates/alalamien-ui/src/
  - `main.rs` — App setup, state registration, system scheduling
  - `icon.rs` — Window icon via `WinitWindows` NonSend resource (PostStartup)
  - `audio.rs` — Audio asset loading (hover.mp3, click.mp3)
  - `states.rs` — `MenuState` resource
  - `ui/menu.rs` — Main menu layout and buttons
  - `ui/credits.rs` — Credits screen ("Youssef Ashraf")
  - `ui/load_game.rs` — Load Game placeholder
  - `ui/background.rs` — Gradient background effects, `animate_background`
  - `systems/navigation.rs` — Pure state transition logic (SRP)
  - `systems/ui_manager.rs` — `ScreenUI` marker, `cleanup_ui`, `setup_*_ui` (SRP)
  - `systems/button.rs` — Hover visual + audio feedback only (SRP)
  - `systems/loading.rs` — ESC/SPACE → back to Menu from Loading state
  - `systems/animation.rs` — Menu title animation
  - `components/` — UI component types
  - `resources/` — Resource types
  - **Asset path:** `"../../assets"` (workspace root, relative to crate)
  - **Icon:** `include_bytes!("../../../assets/images/gameIcon.ico")` embedded at compile time

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

- **All Tests:** `cargo test --lib` (383 tests, 100% pass rate)
- **Unit Tests:** `cargo test --lib tests::unit::`
- **Integration Tests:** `cargo test --lib tests::integration::`
- **System Tests:** `cargo test --lib tests::system::`
- **Regression Tests:** `cargo test --lib tests::regression::`
- **Long Tests:** `cargo test --lib -- --ignored` (100k-tick determinism)
- **Run API:** `cargo run --package alalamien-api`
- **Build UI (V0.8+):** `cargo run --package alalamien-ui`
- **Dev Tools (V0.8+):** `cargo run --package alalamien-ui --features dev-tools`
- **With Output:** `cargo test --lib -- --nocapture`

## Test Architecture

**4-Layer V0.7 Structure (383 Tests):**

1. **Unit Tests** - Component-level validation
   - Core types (GDP, resources, legitimacy, notifications)
   - Subsystem phases (warfare, nuclear, vassalage, espionage, economic, alliances)
   - Notification delivery and filtering

2. **Integration Tests** - Cross-subsystem interactions
   - Alliance-warfare scenarios
   - Nuclear deterrence mechanics
   - Vassal-overlord relationships
   - Espionage operations and counter-intelligence
   - Economic sanctions and trade

3. **System Tests** - End-to-end simulation validation
   - 1000+ tick stability runs
   - Multi-nation conflict scenarios
   - Notification propagation across all subsystems
   - Performance benchmarks (100-500 nations)

4. **Regression Tests** - Historical issue prevention
   - Previous bug reproductions
   - Edge case coverage
   - Determinism verification across versions

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

## V0.7 Integrated Subsystems (6 Core Systems with Notifications)

**Architecture:** 19-phase deterministic tick pipeline with unified notification system across all subsystems.

### 1. **Warfare Subsystem** ⚔️

- War declarations and peace treaties
- Battle resolution with casualty calculation
- War exhaustion mechanics
- Occupation and territory control
- Multi-front war management
- **Notifications:** War declared, battle won/lost, peace treaty signed, territory occupied

### 2. **Nuclear Deterrence Subsystem** ☢️

- Nuclear arsenal tracking
- Launch mechanics and targeting
- Deterrence doctrine enforcement
- Retaliation triggers
- Fallout and long-term effects
- **Notifications:** Nuclear launch detected, retaliation triggered, fallout spread

### 3. **Vassalage Subsystem** 👑

- Overlord-vassal relationships
- Tribute collection and enforcement
- Independence movements
- Protection obligations
- Intervention mechanics
- **Notifications:** Vassal acquired, tribute paid/refused, independence declared

### 4. **Espionage Subsystem** 🕵️

- Spy network establishment
- Intelligence gathering operations
- Sabotage and assassination missions
- Counter-intelligence measures
- Imperfect information gameplay
- **Notifications:** Spy detected, sabotage succeeded, intelligence gathered

### 5. **Economic Subsystem** 💰

- Resource production and trade
- Production chains (iron→military, oil→logistics)
- GDP calculation and economic sanctions
- Trade route management
- Deficit tracking and economic stress
- **Notifications:** Trade embargo imposed, economic sanctions applied, resource shortage

### 6. **Alliance Subsystem** 🤝

- Alliance formation and dissolution
- Cohesion decay mechanics
- Multi-alliance support
- War obligations and mutual defense
- Diplomatic relations and reputation
- **Notifications:** Alliance formed, alliance dissolved, war obligation triggered, reputation changed

### Notification System

**Unified Architecture:**

- All 6 subsystems generate notifications during their tick phase
- Notifications contain: type, severity, tick, nation_id, message, metadata
- Filterable by nation, subsystem, severity, time range
- Persistent across save/load cycles
- API endpoints: `GET /notifications/recent`, `GET /notifications/nation/:id`

### Key Subsystem Files (V0.7)

**Warfare:**

- `crates/alalamien-engine/src/subsystems/warfare.rs`

**Nuclear:**

- `crates/alalamien-engine/src/subsystems/nuclear.rs`

**Vassalage:**

- `crates/alalamien-engine/src/subsystems/vassalage.rs`

**Espionage:**

- `crates/alalamien-engine/src/subsystems/espionage.rs`

**Economic:**

- `crates/alalamien-engine/src/subsystems/economic.rs`

**Alliance:**

- `crates/alalamien-engine/src/subsystems/alliance.rs`

**Notifications:**

- `crates/alalamien-engine/src/core/notification.rs`

---

**V0.7 Tick Pipeline (19 Phases):**

The engine executes 19 ordered phases per tick, ensuring deterministic causality:

1. **Early Setup Phases** - Initialize tick context
2. **Diplomatic Phase** - Update relations, reputation
3. **Alliance Phase** - Cohesion decay, dissolution checks
4. **Economic Phase** - Resource production, trade
5. **Military Buildup** - Army recruitment, positioning
6. **Warfare Phase** - War declarations, battle resolution
7. **Nuclear Phase** - Launch decisions, deterrence
8. **Occupation Phase** - Territory control updates
9. **Vassalage Phase** - Tribute collection, independence checks
10. **Espionage Phase** - Intelligence ops, sabotage
11. **Notification Generation** - Subsystems emit events
12. **Stability Checks** - Legitimacy updates
13. **Population Phase** - Demographics, migration
14. **Late Economic** - GDP recalculation
15. **Event Resolution** - Random events processed
16. **AI Decision** - Strategic planning
17. **Cleanup Phase** - Remove expired entities
18. **State Persistence** - Snapshot preparation
19. **Final Validation** - Consistency checks

This order ensures no circular dependencies and maintains determinism across all 383 tests.

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

### v0.2-v0.6 (Completed ✅)

- Trade routes & logistics
- Stability system (legitimacy)
- Economic production chains
- Diplomatic relations
- Alliance management
- Military units & combat
- War declarations & warfare
- Events system
- Factions & civil war mechanics
- AI systems (advanced & basic)
- Intervention & mediation
- Occupation mechanics

### v0.7 (Completed ✅ - March 3, 2026)

- ✅ **Pre-UI Hardening Phase**
- ✅ **383 tests passing** (100% pass rate)
- ✅ **6 integrated subsystems** (warfare, nuclear, vassalage, espionage, economic, alliances)
- ✅ **Unified notification system** across all subsystems
- ✅ **19-phase tick pipeline** with deterministic execution
- ✅ **Nuclear deterrence system** (launch, retaliation, fallout)
- ✅ **Vassal state mechanics** (overlord-vassal relationships, tribute, independence)
- ✅ **Espionage operations** (intelligence gathering, sabotage, counter-intel)
- ✅ **Comprehensive test coverage** (unit/integration/system/regression)
- ✅ **API endpoint validation**
- ✅ **Headless simulation** ready for UI integration

### v0.8 (In Progress — Phase 0 + 0.5 Complete ✅)

**UI Implementation Phase - See `docs/V0.8_UI_IMPLEMENTATION_ROADMAP.md`**

- ✅ **Bevy Engine 0.14.2** native desktop renderer running
- ✅ **Main menu** — 5-button layout, dark theme, cyan/red accent palette
- ✅ **Audio system** — MP3 hover/click sounds via `bevy_audio` mp3 feature
- ✅ **Window icon** — gameIcon.ico via `WinitWindows` API
- ✅ **Background effects** — gradient accent layers, border lines
- ✅ **Multi-screen navigation** — Menu ↔ Credits ↔ LoadGame ↔ Loading state machine
- ✅ **SOLID architecture** — SRP enforced across navigation.rs, ui_manager.rs, button.rs, background.rs, icon.rs
- ✅ **OnEnter/OnExit lifecycle** — ScreenUI marker + despawn_recursive cleanup
- ⏸️ **Settings screen** — postponed
- 📋 **Vector-based map rendering** (infinite zoom, Web Mercator projection)
- 📋 **Four-panel HUD** (top bar, left panel, right log, bottom controls)
- 📋 **REST API bridge** (HTTP client querying alalamien-api)
- 📋 **Developer tools** (separate `alalamien-dev-tools/` crate, feature-gated)
- 📋 **VFX & Polish** (particles, animations, smooth transitions)
- 📋 **Target: 60+ FPS** with 100+ provinces visible

### v0.9+ (Planned)

- Technological research trees
- Cultural influence mechanics
- Advanced AI strategic planning
- Save/load for long campaigns
- Multiplayer synchronization

### v1.0 (Target)

- Full gameplay loop: diplomacy, economy, warfare, events, UI
- Polished native desktop application
- Complete documentation
- 400+ tests with comprehensive coverage
- Production-ready for portfolio/demo
