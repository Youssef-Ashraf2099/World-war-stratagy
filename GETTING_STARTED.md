# 🎮 Alalamien War - v0.1 Complete Setup

## ✅ What Has Been Created

### 1. **Rust Simulation Engine** (`crates/alalamien-engine/`)

**Core Systems:**

- ✅ Entity Component System (using bevy_ecs)
- ✅ World State Management
- ✅ Tick Pipeline Architecture
- ✅ Deterministic Random Number Generation
- ✅ State Serialization (Save/Load)
- ✅ Metrics & Instrumentation

**Subsystems:**

- ✅ Demographic System (population growth based on food)
- ✅ Economic System (resource production & consumption)

**Core Types:**

- `Nation` - Player-controllable entities
- `Province` - Territory with resources
- `Resources` - Food, Iron, Oil, Rare Earths, Water, Trade Ports
- `Population` - With growth rates
- `Legitimacy` - Stability metric (0-100)
- `GDP` - Economic power

### 2. **REST API Server** (`crates/alalamien-api/`)

**Endpoints:**

```
GET  /health              → Health check
GET  /world/state         → World summary (tick, nations, provinces)
POST /world/tick          → Advance simulation by N ticks
GET  /nations             → List all nations
GET  /nations/{id}        → Get specific nation details
GET  /provinces           → List all provinces
GET  /provinces/{id}      → Get specific province details
GET  /metrics             → Performance metrics
```

**Features:**

- CORS enabled for frontend
- Thread-safe state management
- Test scenario pre-loaded

### 3. **React Frontend** (`frontend/`)

**UI Components:**

- Header with tick counter and controls
- Nations list (with legitimacy/GDP)
- Provinces list (with resources)
- Map view (canvas placeholder)

**State Management:**

- Zustand store
- Auto-refresh every 2 seconds
- API client with TypeScript types

**Tech Stack:**

- React 18 + TypeScript
- Vite (build tool)
- Zustand (state)
- Axios (API)
- Ready for D3.js & MapLibre GL

### 4. **Desktop App** (`crates/alalamien-desktop/`)

**Features:**

- Tauri wrapper (Native performance)
- Embedded API server
- Single window application
- Cross-platform (Windows, Linux, macOS)

---

## 🚀 Getting Started

### Prerequisites

1. **Rust 1.75+**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js 20+**
   - Download: https://nodejs.org/

3. **Tauri Prerequisites** (for desktop app)
   - Windows: WebView2 (usually pre-installed)
   - Linux: `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
   - macOS: Xcode Command Line Tools

### Quick Start

**Option 1: Simple Startup (Recommended)**

Windows:

```cmd
.\start.bat
```

Linux/Mac:

```bash
chmod +x start.sh
./start.sh
```

This will:

1. Install frontend dependencies
2. Build the Rust engine
3. Launch the desktop app

**Option 2: Development Mode (For iteration)**

Terminal 1 - API Server:

```bash
cargo run -p alalamien-api
```

Terminal 2 - Frontend Dev Server:

```bash
cd frontend
npm install
npm run dev
```

Then open http://localhost:5173

---

## 📂 Project Structure

```
Alalamien-War/
│
├── crates/                          # Rust workspace
│   ├── alalamien-engine/            # 🦀 Core simulation
│   │   ├── src/
│   │   │   ├── core/                # Foundation
│   │   │   │   ├── types.rs         # Nation, Province, Resources
│   │   │   │   ├── world.rs         # World state with ECS
│   │   │   │   ├── tick.rs          # Tick pipeline
│   │   │   │   ├── deterministic.rs # Seeded RNG
│   │   │   │   └── state.rs         # Save/load system
│   │   │   ├── subsystems/          # Game systems
│   │   │   │   ├── demographic.rs   # Population dynamics
│   │   │   │   └── economic.rs      # Resource production
│   │   │   ├── instrumentation/     # Metrics & logging
│   │   │   └── utils/               # Math utilities
│   │   └── Cargo.toml
│   │
│   ├── alalamien-api/               # 🌐 REST API
│   │   ├── src/
│   │   │   ├── handlers.rs          # HTTP endpoints
│   │   │   ├── state.rs             # Shared state
│   │   │   └── main.rs              # Server entry
│   │   └── Cargo.toml
│   │
│   └── alalamien-desktop/           # 🖥️  Tauri app
│       ├── src/
│       │   ├── main.rs              # Tauri entry
│       │   └── embedded_server.rs   # Background API server
│       ├── tauri.conf.json          # Tauri configuration
│       └── Cargo.toml
│
├── frontend/                        # ⚛️  React UI
│   ├── src/
│   │   ├── components/              # UI components
│   │   │   ├── Header.tsx           # Top bar with controls
│   │   │   ├── NationsList.tsx      # Nations sidebar
│   │   │   ├── ProvincesList.tsx    # Provinces sidebar
│   │   │   └── MapView.tsx          # Map canvas
│   │   ├── store/
│   │   │   └── gameStore.ts         # Zustand state
│   │   ├── api/
│   │   │   └── client.ts            # API client
│   │   ├── App.tsx                  # Main app
│   │   └── main.tsx                 # Entry point
│   ├── package.json
│   ├── vite.config.ts
│   └── tsconfig.json
│
├── assets/                          # Game assets
│   ├── data/                        # Geospatial shapefiles
│   ├── flags/
│   └── icons/
│
├── docs/                            # Documentation
│   ├── ARCHITECTURE.md
│   ├── ROADMAP.md
│   ├── V0.1_PLAN.md
│   └── ...
│
├── Cargo.toml                       # Rust workspace root
├── README.md                        # Project overview
├── BUILD_GUIDE.md                   # Detailed build guide
├── MIGRATION_COMPLETE.md            # Migration summary
└── start.bat / start.sh             # Quick start scripts
```

---

## 🧪 Testing the System

### 1. Test the API

```bash
# Start API server
cargo run -p alalamien-api

# In another terminal:
curl http://localhost:3000/health
curl http://localhost:3000/world/state
curl http://localhost:3000/nations
curl http://localhost:3000/provinces
```

### 2. Advance the Simulation

```bash
# Advance by 1 tick
curl -X POST http://localhost:3000/world/tick \
  -H "Content-Type: application/json" \
  -d '{"ticks": 1}'

# Advance by 100 ticks
curl -X POST http://localhost:3000/world/tick \
  -H "Content-Type: application/json" \
  -d '{"ticks": 100}'
```

### 3. Run Rust Tests

```bash
# Run all tests
cargo test

# Run specific package tests
cargo test -p alalamien-engine

# Run with output
cargo test -- --nocapture
```

### 4. Test Determinism

```rust
// This is tested automatically in the engine
let world1 = WorldState::new(42);
let world2 = WorldState::new(42);
// Same seed = same results guaranteed
```

---

## 🎯 What Works Right Now

### Current Features

✅ **Simulation:**

- 2 nations (player & AI)
- 3 provinces with different resources
- Population growth based on food availability
- Resource production based on infrastructure
- Deterministic tick execution

✅ **API:**

- Full REST API with 8 endpoints
- CORS enabled
- JSON responses
- Thread-safe state management

✅ **Frontend:**

- Real-time display of world state
- Nations list with legitimacy & GDP
- Provinces list with population & resources
- Map visualization (basic canvas)
- Tick advancement buttons

✅ **Desktop App:**

- Single-window application
- Embedded server
- No external dependencies at runtime

---

## 🛠️ Development Workflow

### Iteration Cycle

1. **Make changes to engine** (`crates/alalamien-engine/src/`)
2. **Run tests**: `cargo test -p alalamien-engine`
3. **Test in API**: `cargo run -p alalamien-api`
4. **View in frontend**: Frontend auto-reloads

### Adding a New Subsystem

1. Create `crates/alalamien-engine/src/subsystems/your_system.rs`
2. Implement `TickPhase` trait
3. Add to `subsystems/mod.rs`
4. Add to tick pipeline in `core/tick.rs`

Example:

```rust
pub struct YourPhase;

impl TickPhase for YourPhase {
    fn name(&self) -> &str { "YourPhase" }

    fn execute(&mut self, world: &mut World) {
        // Your logic here
    }
}
```

### Adding an API Endpoint

1. Add handler to `crates/alalamien-api/src/handlers.rs`
2. Add route to `lib.rs` router
3. Update frontend API client if needed

---

## 📊 Performance Targets (v0.1)

- ✅ 60 ticks/second minimum
- ✅ No memory leaks
- ✅ No NaN/Infinity values
- ✅ Deterministic replay
- ✅ State save/load under 100ms

---

## 🐛 Common Issues

### "Command not found: cargo"

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### "npm: command not found"

Install Node.js from https://nodejs.org/

### Port 3000 already in use

**Windows:**

```cmd
netstat -ano | findstr :3000
taskkill /PID <PID> /F
```

**Linux/Mac:**

```bash
lsof -ti:3000 | xargs kill -9
```

### Frontend shows "Cannot connect to API"

Make sure API server is running:

```bash
cargo run -p alalamien-api
```

### Tauri build fails on Linux

Install required dependencies:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

---

## 📈 Next Steps (Beyond v0.1)

### Immediate Enhancements

1. **MapLibre GL Integration**
   - Replace canvas with real map
   - Use your geospatial data (`assets/data/*.shp`)
   - Clickable provinces

2. **D3.js Visualizations**
   - GDP growth charts
   - Population trends
   - Resource production graphs
   - Legitimacy over time

3. **More Test Scenarios**
   - 5-10 nations
   - Border conflicts
   - Resource scarcity

### v0.2 Goals (From Roadmap)

- Economic dependency system
- Trade routes (graph overlay)
- Resource deficits causing instability
- Blockade mechanics
- Price abstraction

### v0.3 Goals

- War as logistics
- Supply lines
- Attrition mechanics
- Occupation system

---

## 📚 Documentation

- **[README.md](README.md)** - Project overview
- **[BUILD_GUIDE.md](BUILD_GUIDE.md)** - Detailed build instructions
- **[MIGRATION_COMPLETE.md](MIGRATION_COMPLETE.md)** - Migration summary
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System architecture
- **[docs/ROADMAP.md](docs/ROADMAP.md)** - Development roadmap
- **[docs/V0.1_PLAN.md](docs/V0.1_PLAN.md)** - v0.1 milestone plan

---

## 🎓 Learning Resources

### Rust + bevy_ecs

- https://bevyengine.org/learn/book/
- https://docs.rs/bevy_ecs/

### Tauri

- https://tauri.app/start/

### React + TypeScript

- https://react.dev/
- https://www.typescriptlang.org/

### D3.js & MapLibre

- https://d3js.org/
- https://maplibre.org/

---

## ✅ Final Checklist

Before starting development:

- [ ] Rust installed and working
- [ ] Node.js installed
- [ ] Run `.\start.bat` or `./start.sh`
- [ ] See the desktop app open
- [ ] Click "Advance 10 Ticks" and see numbers change
- [ ] Open http://localhost:3000/health in browser
- [ ] Run `cargo test` successfully

---

## 🎉 You're Ready!

The foundation is complete. The architecture is solid. The simulation runs.

**Now build your empire! 👑**

---

_For questions or issues, refer to BUILD_GUIDE.md or the docs/ folder_
