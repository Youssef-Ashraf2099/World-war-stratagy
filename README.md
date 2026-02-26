# Alalamien War

**A deterministic geopolitical simulation engine**

## Project Status

**Current Version:** v0.1 (Development)

Alalamien War is a grand strategy simulation focusing on systemic pressure, geopolitical dynamics, and deterministic state evolution. This is not a traditional war game—it's a statecraft simulator where conflict is one tool among many.

## Architecture

- **Engine:** Rust + bevy_ecs (deterministic simulation core)
- **API Layer:** Axum + Tokio (state exposure)
- **Frontend:** TypeScript + React + D3.js + MapLibre GL
- **Platform:** Desktop (Tauri wrapper)

## Project Structure

```
crates/
├── alalamien-engine/   # Core simulation engine
├── alalamien-api/      # REST API server
└── alalamien-desktop/  # Tauri desktop wrapper
frontend/               # TypeScript/React UI
assets/                 # Game assets, geospatial data
docs/                   # Architecture & design docs
```

## Quick Start

### Prerequisites

- Rust 1.75+ ([Install Rust](https://rustup.rs/))
- Node.js 20+ ([Install Node](https://nodejs.org/))
- Tauri CLI: `cargo install tauri-cli --version "^2.0.0"`

### Build

```bash
# Build the engine
cargo build --release

# Run tests
cargo test

# Start development desktop app
cd crates/alalamien-desktop
cargo tauri dev
```

### Development

```bash
# Run API server only
cargo run -p alalamien-api

# Run frontend dev server
cd frontend
npm install
npm run dev
```

## v0.1 Milestone Goals

✅ Type definitions (Nation, Province, Resources)
✅ World state container with bevy_ecs
✅ Tick pipeline architecture
⏳ Deterministic simulation loop
⏳ Basic logging & instrumentation
⏳ Province resource production
⏳ Simple population dynamics

See [docs/V0.1_PLAN.md](docs/V0.1_PLAN.md) for detailed roadmap.

## Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Development Guide](docs/DEVELOPMENT_GUIDE.md)
- [Roadmap](docs/ROADMAP.md)
- [Quick Reference](docs/QUICK_REFERENCE.md)

## Design Philosophy

1. **Deterministic Systems** — Every outcome traceable to variables
2. **No Magic Numbers** — All values derived from simulation state
3. **Separated Concerns** — Engine core independent from UI
4. **Testable Design** — Each system tested in isolation
5. **Incremental Versioning** — Perfect implementation per version

## License

MIT
