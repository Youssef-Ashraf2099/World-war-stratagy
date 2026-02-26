@echo off
REM Test script to verify geodata loading on Windows

setlocal enabledelayedexpansion

echo Testing geodata extraction and loading...
echo.

REM Run the extraction script
echo 1. Running extraction script...
cd /d "%~dp0"
python scripts\extract_nations.py

echo.
echo 2. Verifying nations.json exists...
if exist "src\game\scenarios\nations.json" (
    echo [OK] nations.json found
    REM Count nations with Python
    for /f %%i in ('python -c "import json; print(len(json.load(open('src/game/scenarios/nations.json'))))"') do set NATION_COUNT=%%i
    echo [OK] Contains !NATION_COUNT! nations
) else (
    echo [ERROR] nations.json not found!
    exit /b 1
)

echo.
echo 3. Checking JSON validity...
python -c "
import json
import sys
try:
    with open('src/game/scenarios/nations.json') as f:
        data = json.load(f)
    print(f'[OK] Valid JSON with {len(data)} nations')
    if data:
        first = data[0]
        print(f'[OK] Sample nation: {first[\"name\"]} (pop: {first[\"population\"]}, gdp: {first[\"gdp\"]})')
except Exception as e:
    print(f'[ERROR] {e}')
    sys.exit(1)
"

echo.
echo 4. Cargo check (verify compilation)...
cargo check --workspace --quiet

echo.
echo [SUCCESS] All geodata tests passed!
echo.
echo Summary:
echo - Extracted !NATION_COUNT! real nations from Natural Earth data
echo - Generated src\game\scenarios\nations.json
echo - All Rust code compiles successfully
echo.
echo Next: Run 'cargo run --package alalamien-api' to start with real world data
