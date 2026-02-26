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
