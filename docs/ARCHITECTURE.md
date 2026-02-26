# Alalamien War - Architecture Document

## Overview

This document outlines the architecture for **Alalamien War v0.1**, a deterministic geopolitical simulation engine.

## Design Principles

1. **Deterministic Systems** — Every outcome is traceable to variables
2. **No Magic Numbers** — All values derived from simulation state
3. **Separated Concerns** — Engine core independent from UI
4. **Testable Design** — Each system can be tested in isolation
5. **Incremental Versioning** — Perfect implementation per version

## Directory Structure

```
src/
├── engine/
│   ├── core/          # Core simulation foundation
│   ├── subsystems/    # Game systems (economy, warfare, etc)
│   ├── instrumentation/  # Debugging & monitoring
│   ├── logging/       # Centralized logging
│   └── utils/         # Shared utilities
├── game/              # Game config & scenarios
├── interface/         # UI layers (CLI, SDL, ImGui)
└── main.cpp           # Entry point
```

## Core Type System

### Entity Hierarchy

```
Entity (base)
├── Nation
└── Province
```

### Key Types

- **Nation** — Player-controllable geopolitical actor
- **Province** — Territory with resources, population, and infrastructure
- **TradeRoute** — Economic connection between nations
- **Alliance** — Multi-member treaty with obligations
- **SocialBloc** — Population stratification (Workers, Elites, Military, Rural)

### Resource Types

- Food
- Iron
- Oil
- Rare Earths
- Water
- Trade Ports

## Tick Pipeline

Each game tick executes phases in strict order:

1. **Demographic** — Population growth, death, migration
2. **Economic** — Production, consumption, GDP calculation
3. **Diplomatic** — Relation decay, alliance management
4. **Logistics** — Resource movement, trade routes
5. **Military** — Army upkeep, supply lines, combat
6. **Stability** — Revolts, ideology shifts, legitimacy
7. **Event** — Random/seeded event triggers

## v0.1 Milestone

**Goal:** Establish core simulation foundation with basic nation-province interaction.

### Completed

- [x] Type definitions (Nation, Province, Resources)
- [x] World state container
- [x] Tick pipeline architecture
- [x] Logging system
- [x] Math utilities
- [x] Entry point with test scenario

### In Progress (v0.1)

- [ ] Province resource production
- [ ] Basic population dynamics
- [ ] Nation GDP calculation
- [ ] Legitimacy calculation
- [ ] Deterministic random number generation

### Deferred to v0.2+

- Full economy simulation
- Trade system
- Military system
- Diplomacy engine
- Event system
- UI/Graphics

## Compilation & Execution

### Prerequisites

- C++17 or later
- CMake 3.20+
- SDL2
- Eigen3
- nlohmann/json
- EnTT

### Build

```bash
cd spirits-of-steel
mkdir build
cd build
cmake ..
cmake --build . --config Release
```

### Run Main Game

```bash
./AlalamienWar
```

### Run Tests

```bash
ctest
```

## Determinism Strategy

All randomness uses **seeded PRNG**:
- Same seed + same replay = identical outcome
- Enables replay/crash investigation
- Required for multiplayer & AI debugging

## Next Steps

1. Implement province resource production
2. Add population dynamics
3. Create basic legitimacy system
4. Build simple economy (GDP calculation)
5. Create initial test scenario with 3-5 nations

See `ROADMAP.md` for full development plan.
