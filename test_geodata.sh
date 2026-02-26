#!/bin/bash
# Test script to verify geodata loading

set -e

echo "Testing geodata extraction and loading..."
echo ""

# Run the extraction script
echo "1. Running extraction script..."
cd "$(dirname "$0")"
python scripts/extract_nations.py

echo ""
echo "2. Verifying nations.json exists..."
if [ -f "src/game/scenarios/nations.json" ]; then
    echo "✓ nations.json found"
    NATION_COUNT=$(python -c "import json; print(len(json.load(open('src/game/scenarios/nations.json'))))")
    echo "✓ Contains $NATION_COUNT nations"
else
    echo "✗ nations.json not found!"
    exit 1
fi

echo ""
echo "3. Checking JSON validity..."
python -c "
import json
import sys
try:
    with open('src/game/scenarios/nations.json') as f:
        data = json.load(f)
    print(f'✓ Valid JSON with {len(data)} nations')
    if data:
        first = data[0]
        print(f'✓ Sample nation: {first[\"name\"]} (pop: {first[\"population\"]}, gdp: {first[\"gdp\"]})')
except Exception as e:
    print(f'✗ Error: {e}')
    sys.exit(1)
"

echo ""
echo "4. Cargo check (verify compilation)..."
cargo check --workspace --quiet

echo ""
echo "✓ All tests passed!"
echo ""
echo "Summary:"
echo "- Extracted $NATION_COUNT real nations from Natural Earth data"
echo "- Generated src/game/scenarios/nations.json"
echo "- All Rust code compiles successfully"
echo ""
echo "Next: Run 'cargo run --package alalamien-api' to start with real world data"
