#!/usr/bin/env python3
"""
Quick test to verify nations.json is valid and loadable
"""

import json
import sys
from pathlib import Path

def main():
    project_root = Path(__file__).parent.parent
    nations_json = project_root / "src" / "game" / "scenarios" / "nations.json"
    
    print(f"Testing: {nations_json}")
    print("=" * 60)
    
    try:
        with open(nations_json, 'r', encoding='utf-8') as f:
            nations = json.load(f)
        
        print(f"✓ JSON is valid")
        print(f"✓ Loaded {len(nations)} nations")
        print()
        
        # Test queries
        print("Sample Nations:")
        print("-" * 60)
        
        # Find USA
        usa = next((n for n in nations if n['code'] == 'US'), None)
        if usa:
            print(f"United States:")
            print(f"  Population: {usa['population']:,}")
            print(f"  GDP: ${usa['gdp']:,.0f}")
            print(f"  Continent: {usa['continent']}")
            print()
        
        # Find China
        china = next((n for n in nations if n['code'] == 'CN'), None)
        if china:
            print(f"China:")
            print(f"  Population: {china['population']:,}")
            print(f"  GDP: ${china['gdp']:,.0f}")
            print(f"  Continent: {china['continent']}")
            print()
        
        # Find Germany
        germany = next((n for n in nations if n['code'] == 'DE'), None)
        if germany:
            print(f"Germany:")
            print(f"  Population: {germany['population']:,}")
            print(f"  GDP: ${germany['gdp']:,.0f}")
            print(f"  Continent: {germany['continent']}")
            print()
        
        # Statistics
        print("Statistics:")
        print("-" * 60)
        total_pop = sum(n['population'] for n in nations)
        total_gdp = sum(n['gdp'] for n in nations if n['gdp'] > 0)
        continents = set(n['continent'] for n in nations)
        
        print(f"Total Population: {total_pop:,}")
        print(f"Total GDP: ${total_gdp:,.0f}")
        print(f"Continents: {', '.join(sorted(continents))}")
        print()
        
        # Top 10 by population
        print("Top 10 Nations by Population:")
        print("-" * 60)
        by_pop = sorted(nations, key=lambda n: n['population'], reverse=True)[:10]
        for i, nation in enumerate(by_pop, 1):
            print(f"{i:2d}. {nation['name']:30s} {nation['population']:>15,}")
        print()
        
        # Top 10 by GDP
        print("Top 10 Nations by GDP:")
        print("-" * 60)
        by_gdp = sorted([n for n in nations if n['gdp'] > 0], 
                       key=lambda n: n['gdp'], reverse=True)[:10]
        for i, nation in enumerate(by_gdp, 1):
            print(f"{i:2d}. {nation['name']:30s} ${nation['gdp']:>15,.0f}")
        
        print()
        print("=" * 60)
        print("✓ All tests passed!")
        print(f"✓ Ready for C++ GeoLoader integration")
        return 0
        
    except Exception as e:
        print(f"✗ ERROR: {e}")
        import traceback
        traceback.print_exc()
        return 1

if __name__ == '__main__':
    sys.exit(main())
