@echo off
echo ========================================
echo Alalamien War - Quick Start Script
echo ========================================
echo.

echo [1/3] Installing frontend dependencies...
cd frontend
if not exist node_modules (
    call npm install
    if errorlevel 1 (
        echo ERROR: Failed to install frontend dependencies
        pause
        exit /b 1
    )
) else (
    echo Frontend dependencies already installed
)
cd ..

echo.
echo [2/3] Building Rust engine...
cargo build
if errorlevel 1 (
    echo ERROR: Failed to build Rust project
    pause
    exit /b 1
)

echo.
echo [3/3] Starting desktop application...
echo This will open the Alalamien War window
echo.
cd crates\alalamien-desktop
cargo tauri dev

pause
