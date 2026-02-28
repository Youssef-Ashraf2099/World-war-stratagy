#!/usr/bin/env python3
"""
Extract populated places (cities) from Natural Earth shapefiles.

This provides major cities, capitals, and population centers
that can be used for capital identification and strategic targets.

Output format:
{
    "cities": [
        {
            "name": "Washington, D.C.",
            "country": "United States",
            "is_capital": true,
            "population": 5379184,
            "latitude": 38.9072,
            "longitude": -77.0369
        },
        ...
    ]
}
"""

import json
import sys
from pathlib import Path

try:
    import shapefile
except ImportError:
    print("ERROR: pyshp library not found")
    print("Install it with: pip install pyshp")
    sys.exit(1)


def extract_cities(shapefile_path: str, output_path: str):
    """
    Extract city data from Natural Earth populated places shapefile.
    
    Args:
        shapefile_path: Path to the .shp file (without extension)
        output_path: Where to save the JSON output
    """
    print(f"Reading shapefile: {shapefile_path}")
    
    try:
        sf = shapefile.Reader(shapefile_path)
    except Exception as e:
        print(f"ERROR: Could not read shapefile: {e}")
        return
    
    # Get field names
    field_names = [field[0] for field in sf.fields[1:]]
    print(f"Available fields: {field_names[:20]}...")  # Show first 20 fields
    
    cities = []
    capitals = []
    
    # Process each city record
    for shape, record in zip(sf.shapes(), sf.records()):
        rec_dict = dict(zip(field_names, record))
        
        # Extract key fields
        city_name = rec_dict.get('NAME') or rec_dict.get('name') or rec_dict.get('NAMEASCII')
        country_name = rec_dict.get('ADM0NAME') or rec_dict.get('adm0name') or rec_dict.get('SOV0NAME')
        
        # Population data
        population = rec_dict.get('POP_MAX') or rec_dict.get('pop_max') or rec_dict.get('POP_MIN') or 0
        if isinstance(population, str):
            try:
                population = int(population)
            except:
                population = 0
        
        # Capital status
        feature_class = rec_dict.get('FEATURECLA') or rec_dict.get('featurecla') or ''
        is_capital = 'Admin-0 capital' in str(feature_class) or 'capital' in str(feature_class).lower()
        
        # Get coordinates from shape
        if shape.points:
            point = shape.points[0]
            longitude = point[0]
            latitude = point[1]
        else:
            continue  # Skip if no coordinates
        
        # Get timezone if available
        timezone = rec_dict.get('TIMEZONE') or rec_dict.get('timezone')
        
        # Strategic importance (based on population and capital status)
        strategic_value = 10 if is_capital else 0
        if population > 10_000_000:
            strategic_value += 5
        elif population > 5_000_000:
            strategic_value += 3
        elif population > 1_000_000:
            strategic_value += 1
        
        if city_name and country_name:
            city_data = {
                "name": str(city_name),
                "country": str(country_name),
                "is_capital": bool(is_capital),
                "population": int(population) if population else 0,
                "latitude": round(latitude, 4),
                "longitude": round(longitude, 4),
                "strategic_value": strategic_value,
            }
            
            if timezone:
                city_data["timezone"] = str(timezone)
            
            cities.append(city_data)
            if is_capital:
                capitals.append(city_data)
    
    print(f"Extracted {len(cities)} cities")
    print(f"Found {len(capitals)} capital cities")
    
    # Sort by population
    cities.sort(key=lambda x: x['population'], reverse=True)
    
    print(f"\nTop 10 cities by population:")
    for i, city in enumerate(cities[:10], 1):
        cap_marker = " (CAPITAL)" if city['is_capital'] else ""
        print(f"  {i}. {city['name']}, {city['country']}: {city['population']:,}{cap_marker}")
    
    print(f"\nCapitals by country:")
    capitals.sort(key=lambda x: x['country'])
    for capital in capitals[:20]:  # Show first 20
        print(f"  - {capital['country']}: {capital['name']}")
    
    # Save to JSON
    output_data = {
        "cities": cities,
        "total_cities": len(cities),
        "capitals": capitals,
        "total_capitals": len(capitals)
    }
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)
    
    print(f"\nSaved city data to: {output_path}")


if __name__ == "__main__":
    assets_dir = Path(__file__).parent.parent / "assets" / "data"
    output_dir = Path(__file__).parent.parent / "src" / "game" / "scenarios"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    cities_shp = assets_dir / "ne_110m_populated_places"
    cities_output = output_dir / "cities.json"
    
    if cities_shp.with_suffix('.shp').exists():
        print("=" * 60)
        print("Extracting cities from Natural Earth populated places")
        print("=" * 60)
        extract_cities(str(cities_shp), str(cities_output))
    else:
        print(f"ERROR: Cities shapefile not found at {cities_shp}")
    
    print("\n" + "=" * 60)
    print("Done!")
    print("=" * 60)
