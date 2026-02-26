# Alalamien War v0.1 - Build and Run Guide

## Prerequisites

### Required Software

1. **Rust** (1.75+)

   ```bash
   # Install via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Node.js** (20+)

   ```bash
   # Download from https://nodejs.org/
   # Or use nvm: nvm install 20
   ```

3. **Tauri CLI**
   ```bash
   cargo install tauri-cli --version "^2.0.0"
   ```

## Quick Start

### 1. Install Frontend Dependencies

```bash
cd frontend
npm install
cd ..
```

### 2. Option A: Run Desktop App (Recommended)

This starts both the API server and frontend in one window:

```bash
cd crates/alalamien-desktop
cargo tauri dev
```

The desktop app will:

- Start the simulation engine
- Start the API server on http://localhost:3000
- Open the frontend UI automatically

### 3. Option B: Run Separately (For Development)

**Terminal 1 - API Server:**

```bash
cargo run -p alalamien-api
```

**Terminal 2 - Frontend:**

```bash
cd frontend
npm run dev
```

Then open http://localhost:5173 in your browser.

## Running Tests

```bash
# Run all tests
cargo test

# Run specific package tests
cargo test -p alalamien-engine
cargo test -p alalamien-api

# Run with output
cargo test -- --nocapture
```

## Building for Production

### Desktop App

```bash
cd crates/alalamien-desktop
cargo tauri build
```

The built app will be in `crates/alalamien-desktop/target/release/bundle/`

### API Server Only

```bash
cargo build --release -p alalamien-api
# Binary: target/release/alalamien-api
```

## Project Structure

```
.
├── crates/
│   ├── alalamien-engine/    # Core simulation engine
│   │   ├── src/
│   │   │   ├── core/        # Types, world state, tick pipeline
│   │   │   ├── subsystems/  # Demographic, economic systems
│   │   │   ├── instrumentation/  # Metrics, logging
│   │   │   └── utils/       # Math utilities
│   │   └── Cargo.toml
│   │
│   ├── alalamien-api/       # REST API server
│   │   ├── src/
│   │   │   ├── handlers.rs  # HTTP endpoints
│   │   │   ├── state.rs     # API state management
│   │   │   └── main.rs      # Entry point
│   │   └── Cargo.toml
│   │
│   └── alalamien-desktop/   # Tauri desktop app
│       ├── src/
│       │   ├── main.rs      # Tauri entry
│       │   └── embedded_server.rs
│       ├── tauri.conf.json
│       └── Cargo.toml
│
├── frontend/                # React + TypeScript UI
│   ├── src/
│   │   ├── components/      # UI components
│   │   ├── store/           # Zustand state
│   │   ├── api/             # API client
│   │   └── App.tsx
│   └── package.json
│
├── assets/                  # Game assets
├── docs/                    # Documentation
└── Cargo.toml              # Workspace root
```

## Available API Endpoints

When the server is running:

- `GET /health` - Health check
- `GET /world/state` - Get world state summary
- `POST /world/tick` - Advance simulation
- `GET /nations` - List all nations
- `GET /nations/{id}` - Get specific nation
- `GET /provinces` - List all provinces
- `GET /provinces/{id}` - Get specific province
- `GET /metrics` - Performance metrics

## Development Tips

### Fast Iteration

```bash
# Watch mode for engine only
cargo watch -p alalamien-engine -x test

# Frontend hot reload is automatic with vite
```

### Debugging

1. Enable debug logging:

   ```bash
   RUST_LOG=debug cargo run -p alalamien-api
   ```

2. Check the browser console for frontend logs

3. API health: http://localhost:3000/health

### Performance

The engine is designed for determinism, not maximum speed in v0.1.
Current target: 60 ticks/second minimum.

Use metrics endpoint to monitor:

```bash
curl http://localhost:3000/metrics
```

## Troubleshooting

### "Cannot find module" errors (Frontend)

```bash
cd frontend
rm -rf node_modules
npm install
```

### Port already in use

```bash
# Kill process on port 3000 (API)
# Windows:
netstat -ano | findstr :3000
taskkill /PID <PID> /F

# Linux/Mac:
lsof -ti:3000 | xargs kill -9
```

### Tauri build fails

Make sure you have the required system dependencies:

- **Windows:** WebView2 (usually pre-installed on Windows 10+)
- **Linux:** `webkit2gtk`, `libappindicator3`, etc.
- **macOS:** Xcode Command Line Tools

## Next Steps

See [V0.1_PLAN.md](./V0.1_PLAN.md) for the development roadmap.

Current v0.1 implementation includes:

- ✅ Core types (Nation, Province, Resources)
- ✅ World state with ECS
- ✅ Tick pipeline
- ✅ Basic demographic system
- ✅ Basic economic system
- ✅ REST API
- ✅ React frontend
- ✅ Desktop app wrapper

Next priorities:

- [ ] Deterministic random events
- [ ] Advanced population dynamics
- [ ] GDP calculation system
- [ ] Legitimacy calculation
- [ ] MapLibre GL integration
- [ ] D3.js visualizations
