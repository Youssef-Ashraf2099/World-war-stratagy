# Spirits of Steel - Quick Reference Card (v0.1)

## One-Minute Overview

**What:** Deterministic geopolitical simulation engine  
**Where:** `E:\Ambisious money\strategy-game\`  
**Status:** Foundation complete, ready for implementation  
**Next:** Implement demographics system  

---

## Quick Commands

### Compile
```bash
cd E:\Ambisious money\strategy-game
mkdir build && cd build
cmake .. -G "Visual Studio 17 2022"
cmake --build . --config Release
```

### Run Game
```bash
./Release/AlalamienWar.exe
```

### Run Tests
```bash
ctest --verbose
```

### View Logs
```bash
cat spirits_of_steel.log | grep ERROR
```

---

## File Locations (What to Edit)

| Task | File | Location |
|------|------|----------|
| Implement Demographics | `TickPipeline.cpp` | `src/engine/core/tick/` |
| Add Economy | `EconomyPhase` | `src/engine/core/tick/` |
| Add Resources | `Resources.h` | `src/engine/core/types/` |
| Create Scenario | `main.cpp` | `src/` |
| Add Tests | `test_*.cpp` | `tests/unit/` |
| Change Constants | `Constants.h` | `src/engine/utils/` |
| Configure Game | `*.json` | `src/game/configs/` |

---

## Core Types (What You Work With)

### Nation
```cpp
Nation nation(0);
nation.name = "TestLand";
nation.population = 50000000;
nation.legitimacy = 65.0;
nation.governanceType = GovernanceType::Democracy;
```

### Province
```cpp
Province province(0);
province.name = "TestProvince";
province.population = 1000000;
province.ownerNation = 0;
province.resourceProduction.baseYield = 100.0;
```

### Resources
```cpp
ResourceStack resources;
resources[ResourceType::Food] = 1000.0;
resources[ResourceType::Iron] = 500.0;
double total = resources.total();
```

---

## Tick Pipeline (What Happens Each Turn)

```
Tick N:
  1. Demographic  → Population changes
  2. Economic     → Resource production
  3. Diplomatic   → Relations decay
  4. Logistics    → Trade routes
  5. Military     → Army upkeep
  6. Stability    → Legitimacy changes
  7. Event        → Random triggers
```

---

## Important Constants (in `Constants.h`)

```cpp
TICKS_PER_YEAR = 12         // 12 months per year
DEFAULT_BIRTH_RATE = 0.03   // 3% population growth
DEFAULT_DEATH_RATE = 0.012  // 1.2% death rate
DEFAULT_TAX_RATE = 0.15     // 15% tax collection
```

---

## Logging (How to Debug)

### Log a Message
```cpp
SOS_LOG_INFO("Province created: " + province.name);
SOS_LOG_ERROR("Nation has negative population!");
SOS_LOG_DEBUG("Trade route updated");
```

### Configure Logging
```cpp
Logger::getInstance().setMinimumLevel(LogLevel::TRACE);
Logger::getInstance().setLogFile("mylog.log");
```

### View Logs
```bash
# All messages
tail -50 spirits_of_steel.log

# Only errors
grep ERROR spirits_of_steel.log

# Only debug
grep DEBUG spirits_of_steel.log
```

---

## Common Tasks

### Create a Nation
```cpp
Nation nation(1);
nation.name = "MyNation";
nation.totalPopulation = 50000000;
nation.gdp = 1000000.0;
world.addNation(nation);
```

### Create a Province
```cpp
Province province(0);
province.name = "MyProvince";
province.ownerNation = 1;
province.population = 1000000;
world.addProvince(province);
```

### Run Simulation
```cpp
WorldState world;
TickPipeline pipeline;

// Register phases
pipeline.registerPhase(std::make_shared<DemographicPhase>());
pipeline.registerPhase(std::make_shared<EconomicPhase>());

// Run ticks
for (int i = 0; i < 120; ++i) {
    pipeline.tick(world.getCurrentTick());
    world.advanceTick();
}
```

### Test Determinism
```cpp
// Run 1
world1.setSeed(12345);
for(120) world1.tick();
uint64_t hash1 = world1.hashState();

// Run 2
world2.setSeed(12345);
for(120) world2.tick();
uint64_t hash2 = world2.hashState();

// hash1 == hash2  ✅
```

---

## Next Steps (In Order)

1. **Compile** (5 min) → Verify build works
2. **Test** (5 min) → Run `ctest`
3. **Run game** (2 min) → See it work
4. **Implement demographics** (2-3 hrs) → Population grows
5. **Implement economy** (2-3 hrs) → Resources produced
6. **Implement GDP** (1-2 hrs) → Nations calculate GDP
7. **Implement legitimacy** (2 hrs) → Nations gain/lose legitimacy
8. **Create scenario** (1 hr) → Test with 3 nations
9. **Validate determinism** (1 hr) → Verify reproducibility

**Total:** ~15-20 hours to complete v0.1

---

## Key Files Map

```
src/
├── main.cpp                 ← Entry point (where to add scenarios)
├── engine/
│   ├── core/
│   │   ├── types/          ← Data structures (Nation, Province, Resources)
│   │   ├── state/          ← World container
│   │   ├── tick/           ← Game loop phases
│   │   └── deterministic/  ← PRNG & hashing
│   ├── logging/            ← Debug logging system
│   ├── utils/              ← Math utilities & constants
│   └── subsystems/         ← Future systems (v0.2+)
├── game/
│   ├── configs/            ← JSON configuration files
│   └── scenarios/          ← Pre-built scenarios
└── interface/              ← UI layer (v0.3+)

tests/
├── unit/                   ← Unit tests
├── integration/            ← Integration tests
├── determinism/            ← Determinism validation
└── regression/             ← Known outcome tests

docs/
├── README.md              ← Start here
├── ARCHITECTURE.md        ← System design
├── V0.1_PLAN.md           ← v0.1 roadmap
├── DEVELOPMENT_GUIDE.md   ← How to continue
└── PROJECT_INITIALIZED.md ← Full status
```

---

## Common Errors & Fixes

| Error | Fix |
|-------|-----|
| `undefined reference to Logger` | Add `src/engine/logging/Logger.cpp` to CMakeLists.txt |
| `cannot find WorldState` | Add `#include "engine/core/state/WorldState.h"` |
| `Phase not executing` | Verify phase is registered in `TickPipeline` |
| `Determinism fails` | Check for uninitialized variables or timing issues |
| `Population goes negative` | Use `Math::clamp()` to prevent negative values |
| `Log file not created` | Call `setLogFile()` before logging |

---

## Design Reminders

✅ **Deterministic** — Same seed = same outcome  
✅ **No magic numbers** — Use `Constants::`  
✅ **Logged** — Every change is logged  
✅ **Testable** — Each system can be tested alone  
✅ **Documented** — Comments explain why, not what  
✅ **Typed** — Use enums, not strings  

---

## v0.1 Success Criteria

- [x] Foundation structure created
- [ ] Compiles without warnings
- [ ] Main loop runs 120 ticks
- [ ] Demographics implemented
- [ ] Economy implemented
- [ ] Legitimacy implemented
- [ ] 3-nation scenario runs
- [ ] Determinism validated
- [ ] Tests pass

---

## Helpful Documents

- **Getting Started:** `README.md`
- **System Design:** `ARCHITECTURE.md`
- **What To Build:** `V0.1_PLAN.md`
- **How To Build It:** `DEVELOPMENT_GUIDE.md`
- **Project Status:** `PROJECT_INITIALIZED.md`
- **Original GDD:** `strategy game.md`
- **Original Roadmap:** `ROADMAP.md`

---

## Remember

> *"We ship fewer features per release, but they're perfect."*

Focus on **one system at a time**. Finish it completely. Test it thoroughly. **Then move to the next.**

Good luck! 🎮

