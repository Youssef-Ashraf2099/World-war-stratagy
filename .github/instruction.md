# AI Agent Context Guide

This repo is a Rust-based geopolitical simulation engine with real-world geodata. Use this file to get up to speed quickly.

## Quick Start (Read Order)

1. docs/versions/V0.1_FINAL_SUMMARY.md
2. docs/versions/V0.1_TARGETS_REVIEW.md
3. docs/versions/V0.1_COMPLETION_CERTIFICATE.md
4. docs/versions/VERSION_INDEX.md
5. PROJECT_STATUS.md
6. docs/DOCUMENTATION_INDEX.md

## Current Status (as of Feb 27, 2026)

- v0.1 is 100% complete (10/10 original targets met)
- 34/34 engine tests passing
- 500-tick stability test added and passing
- Province adjacency graph implemented and integrated
- API server works and loads 177 nations
- Desktop build produces a Windows executable

## Key Code Entry Points

- Engine core: crates/alalamien-engine/src/
  - core/tick.rs (tick pipeline + 500-tick tests)
  - core/province_graph.rs (adjacency graph)
  - core/world.rs (WorldState)
  - core/types.rs (ECS components)
  - subsystems/ (demographic + economic systems)
- API: crates/alalamien-api/src/
- Desktop: crates/alalamien-desktop/src-tauri/
- Geodata: assets/data/ and crates/alalamien-engine/src/game/geodata.rs

## Common Commands

- Engine tests: cargo test --package alalamien-engine
- Run API: cargo run --package alalamien-api
- Build desktop: cargo build --package alalamien-desktop

## Notes

- API world state currently has nations but no provinces; /provinces returns empty.
- Determinism is verified with 500-tick state hashing tests.
- Do not delete or rewrite version docs; append updates with dates.

---

## V0.2 Development (In Progress)

### Border Data Extraction

V0.2 adds trade, logistics, and economic dependencies. To support this, you need to populate the `ProvinceGraph` with real border data from Natural Earth.

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

- ✅ 40/40 tests passing
- ✅ 5-phase pipeline (Economy → Trade → Logistics → Stability → Demographics)
- ✅ Stability system with legitimacy mechanics
- ⏳ Border data extraction complete, integration pending

### Trade System (V0.2)

The trade system allows provinces of the same nation to share resources:

1. If a province has food deficit (< 0), it tries to pull from neighboring provinces with surplus
2. Uses ProvinceGraph to find neighbors
3. Runs multiple diffusion passes to balance resources
4. Affects legitimacy when imports fail

**Key files:**

- `crates/alalamien-engine/src/subsystems/trade.rs`
- `crates/alalamien-engine/src/subsystems/logistics.rs`

### Compile Issues Fixed

Recent fixes:

- Added `OwnedBy` component (was missing)
- Fixed type annotations in resource operations
- Removed unused imports (Legitimacy, Infrastructure in tests)
- Fixed HashMap type inference issues

### Next V0.2 Tasks

According to ROADMAP.md, V0.2 should implement:

- [x] Trade routes (graph overlay)
- [x] Resource deficits
- [ ] Price abstraction (simple scalar, not a market)
- [x] Starvation penalties
- [x] Stability system (legitimacy, protests, civil war) 🆕
- [ ] Production chains (Iron → Military Capacity, Oil → Logistics Range remaining)
- [ ] Blockade simulation
- [ ] Deterministic replay verification
- [ ] Border data integration into ProvinceGraph

**Stability System (NEW):**

The stability system models internal nation pressure based on:

- Hostile neighbor count (0.5 legitimacy loss per neighbor)
- Active war fronts (2.0 legitimacy loss per front)
- Resource deficits (food shortages especially)

Legitimacy thresholds trigger escalating unrest:

- < 35: Protests spawn
- < 25: Rebel movements form
- < 15: Civil war erupts (population casualties + resource destruction)

**Key files:**

- `crates/alalamien-engine/src/subsystems/stability.rs`
- Event markers: `ProtestEvent`, `RebelMovement`, `CivilWar`

**Logical Phase Order (FIXED):**

Current V0.2 pipeline in `core/tick.rs`:

```
tick():
    1. EconomicPhase      // Resource production
    2. TradePhase         // Resource distribution
    3. LogisticsPhase     // Supply lines, attrition
    4. StabilityPhase     // Legitimacy, protests, civil war
    5. DemographicPhase   // Population changes
```

This order is critical for determinism and logical causality.

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

### v0.1 (Completed)

- Core ECS simulation engine
- Deterministic tick pipeline
- Population and resource systems
- Province adjacency graph
- 500-tick stability test
- Save/load + state hashing
- API server and desktop wrapper
- Real geodata integration (177 nations)

### v0.2 (Planned)

- Province generation from geodata
- Frontend map visualization (web or desktop)
- Basic UI for inspecting nations and provinces in the map refrence inside the assets
- Improved resource balancing and visualization

### v0.3 (Planned)

- Diplomacy: alliances, trade, treaties
- Economy expansion: trade routes, logistics
- Events system: random events and global crises

### v0.4 (Planned)

- Military units and combat resolution
- War declarations and conflict outcomes
- Supply and attrition mechanics

### v1.0 (Target)

- Full gameplay loop: diplomacy, economy, warfare, and events
- AI nation behavior and strategy
- Save/load for long campaigns
- Polished UI and stability targets
