# Spirits of Steel - v0.1 Development Guide

## Project Status

Your Spirits of Steel project foundation is now **fully scaffolded and ready for implementation**.

## What Was Created

### ✅ Folder Structure
```
strategy-game/
├── src/engine/          # Core simulation engine
│   ├── core/           # Type system, state, tick pipeline
│   ├── subsystems/     # Game systems (7 folders ready for v0.2+)
│   ├── instrumentation/# Logging & debugging
│   ├── logging/        # Centralized logger
│   └── utils/          # Math, constants, utilities
├── src/game/           # Game configs & scenarios
├── src/interface/      # UI stubs (CLI, ImGui, SDL)
├── tests/              # Unit, integration, determinism tests
├── tools/              # Profiler, scenario builder, inspector
├── docs/               # Architecture, roadmap, plans
└── assets/             # Already present (flags, data, icons)
```

### ✅ Core Type System
- **Entity** — Base class for all game objects
- **Nation** — Full statecraft actor with all attributes
- **Province** — Territory with resources, population, infrastructure
- **Resources** — 6 resource types with pricing model
- **Governance/Economy/Military** — 21 enum types for deep systemic gameplay

### ✅ Infrastructure
- **World State** — Container holding all nations, provinces, and relationships
- **Tick Pipeline** — 7-phase execution system (Demographic → Event)
- **Logger** — Singleton logging with file + console output
- **Math Utils** — Utility functions and constants
- **CMake Build System** — Ready for compilation

### ✅ Documentation
- **README.md** — Project overview
- **ARCHITECTURE.md** — System design
- **V0.1_PLAN.md** — Detailed v0.1 roadmap
- **.gitignore** — Git configuration

### ✅ Starting Point
- **main.cpp** — Boots world, creates test scenario, runs 12 ticks
- **test_entity_types.cpp** — Basic unit tests
- **Tool stubs** — Profiler, Scenario Builder, State Inspector

## Immediate Next Steps (For You)

### Step 1: Compile the Project
```bash
cd "E:\Ambisious money\strategy-game"
mkdir build
cd build
cmake .. -G "Visual Studio 17 2022"
cmake --build . --config Release
```

This should compile without errors (assuming dependencies are installed).

### Step 2: Run the Main Executable
```bash
./Release/SpiritsOfSteel.exe
```

You should see:
```
[2026-02-26 ...] [INFO] === Spirits of Steel Engine v0.1 ===
[2026-02-26 ...] [INFO] Initializing game world...
[2026-02-26 ...] [INFO] World state created
...
[2026-02-26 ...] [INFO] Running 12 ticks (1 year at 12 ticks/year)
[2026-02-26 ...] [INFO] ========== TICK 0 ==========
...
[2026-02-26 ...] [INFO] Simulation completed successfully
```

### Step 3: Implement Demographics (First Real System)
Edit `src/engine/core/tick/TickPipeline.cpp` — find `DemographicPhase::execute()`:

```cpp
void DemographicPhase::execute(GameTick currentTick) {
    // Access world state (will need to pass it to phases)
    // for each province:
    //   newPop = currentPop * (1 + birthRate - deathRate)
    //   province.population = newPop
}
```

**To implement this properly:**
1. Modify `TickPipeline` to hold reference to `WorldState`
2. Pass `WorldState*` to each `phase->execute()`
3. Implement the demographic calculation
4. Write unit test: "1M pop with 2.5% growth → should reach 1.25M in 10 years"

### Step 4: Implement Resource Production
Edit `EconomicPhase::execute()`:

```cpp
for each province:
    production = baseYield * infrastructureMultiplier * populationBonus
    province.currentResources[primaryResourceType] += production
```

### Step 5: Implement GDP Calculation
Create `src/engine/subsystems/economy/EconomyCalculations.h`:

```cpp
double calculateNationGDP(const Nation& nation, const WorldState& world) {
    double gdp = 0;
    for (ProvinceID provinceId : nation.provinces) {
        const Province* province = world.getProvince(provinceId);
        gdp += province->resourceProduction.getCurrentProduction() 
               * world.getGlobalResourcePrice().getPrice(
                   province->resourceProduction.primaryResource);
    }
    return gdp;
}
```

### Step 6: Test Determinism
Once systems are implemented:
```cpp
// tests/determinism/test_replay_consistency.cpp
WorldState world1;
world1.setSeed(12345);
for(120 ticks) world1.tick();
uint64_t hash1 = world1.hashState();

WorldState world2;
world2.setSeed(12345);
for(120 ticks) world2.tick();
uint64_t hash2 = world2.hashState();

assert(hash1 == hash2);
```

## Architecture Notes for Implementation

### No Global Mutable State
```cpp
// ❌ WRONG
static WorldState g_world;

// ✅ RIGHT
int main() {
    WorldState world;
    // pass by reference
}
```

### Logging is Exception
The Logger is a static singleton because logging itself shouldn't throw:
```cpp
SOS_LOG_INFO("Province " + std::to_string(id) + " population: " + std::to_string(pop));
```

### Constants Over Magic Numbers
```cpp
// ❌ BAD
newPop = pop * 1.025;

// ✅ GOOD
newPop = pop * (1.0 + Constants::DEFAULT_BIRTH_RATE);
```

### Every Value Logged
```cpp
void EconomicPhase::execute(GameTick tick) {
    for (auto nationId : world->getAllNationIDs()) {
        Nation* nation = world->getNation(nationId);
        double oldGDP = nation->gdp;
        nation->gdp = calculateGDP(*nation, *world);
        
        SOS_LOG_DEBUG("Nation " + nation->name + 
                      " GDP: " + std::to_string(oldGDP) + 
                      " → " + std::to_string(nation->gdp));
    }
}
```

## Key Files You'll Edit Most

| File | Purpose | v0.1 Task |
|------|---------|-----------|
| `TickPipeline.cpp` | 7 phases | Implement Demographics, Economic, Stability |
| `WorldState.h/.cpp` | Entity container | Already implemented |
| `Entity.h` | Type defs | ✅ Complete |
| `Nation.h` | Nation data | ✅ Complete |
| `Province.h` | Province data | ✅ Complete |
| `main.cpp` | Entry point | Add test scenario setup |
| `Logger.cpp` | Logging | ✅ Complete |

## Testing Strategy

### Unit Tests
```bash
ctest --verbose
```

### Manual Testing
```bash
./SpiritsOfSteel 2>&1 | tail -50
```

### Determinism Validation
Run same seed twice, compare logs/hashes.

## Git Workflow for v0.1

Commit after each system:

```bash
git add .
git commit -m "v0.1: Implement demographics system

- Population growth based on birth/death rates
- Tracks population movement between provinces
- All values logged for debugging
- Unit tests: growth_test, death_rate_test"

git commit -m "v0.1: Implement resource production

- Provinces produce based on type and infrastructure
- Infrastructure multiplier affects yield
- Economic phase updates province resources"

git commit -m "v0.1: Implement GDP calculation

- Nations calculate GDP from all provinces
- GDP = sum(production * resourcePrice)
- Logged per nation per tick"

git commit -m "v0.1: Validate determinism

- Same seed = identical outcome verified
- 120-tick simulation hash matches
- Ready for v0.2"
```

## Debugging Tips

### View Logs
```bash
cat spirits_of_steel.log | grep "ERROR\|CRITICAL"
```

### Low-Level Trace
Change in main.cpp:
```cpp
Logger::getInstance().setMinimumLevel(LogLevel::TRACE);
```

### Attach Debugger (MSVC)
```bash
Start → Debug Without Debugging (Ctrl+F5)
```

Or use:
```bash
devenv.exe /debugexe Release\AlalamienWar.exe
```

## Philosophy Reminder

This is **vibe coding + engineering rigor**:

- ✅ Every system should feel right (vibe)
- ✅ Every system should be testable (engineering)
- ✅ No unexplained behavior
- ✅ No rushed features
- ✅ One commit = one perfect system

## File Organization Summary

```
Headers (.h)        → src/engine/core/types/, src/engine/subsystems/*/
Implementations (.cpp) → Same folder as .h (except core/)
Tests (.cpp)        → tests/unit/, tests/integration/, tests/determinism/
Configs (.json)     → src/game/configs/
Scenarios (.json)   → src/game/scenarios/
Tools               → tools/*/
Docs (.md)          → docs/
```

---

**You now have a production-quality C++ foundation for your geopolitical sim.**

Next: Implement demographics and watch the world come alive! 🌍

