#!/bin/bash

echo "========================================"
echo "Alalamien War - Quick Start Script"
echo "========================================"
echo ""

echo "[1/3] Installing frontend dependencies..."
cd frontend
if [ ! -d "node_modules" ]; then
    npm install
    if [ $? -ne 0 ]; then
        echo "ERROR: Failed to install frontend dependencies"
        exit 1
    fi
else
    echo "Frontend dependencies already installed"
fi
cd ..

echo ""
echo "[2/3] Building Rust engine..."
cargo build
if [ $? -ne 0 ]; then
    echo "ERROR: Failed to build Rust project"
    exit 1
fi

echo ""
echo "[3/3] Starting desktop application..."
echo "This will open the Alalamien War window"
echo ""
cd crates/alalamien-desktop
cargo tauri dev
