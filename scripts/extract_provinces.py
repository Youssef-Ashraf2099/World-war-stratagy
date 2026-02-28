#!/usr/bin/env python3
"""
Extract province/state data from Natural Earth admin_1 shapefiles.

This provides sub-national administrative divisions (states, provinces, regions)
that can be used to create provinces in the game.

Output format:
{
    "provinces": [
        {
            "name": "California",
            "country": "United States",
            "type": "State",
            "population": 39538223,
            "latitude": 37.0,
            "longitude": -120.0
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


def extract_provinces(shapefile_path: str, output_path: str):
    """
    Extract province/state data from Natural Earth admin_1 shapefile.
    
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
    
    provinces = []
    
    # Process each province record
    for shape, record in zip(sf.shapes(), sf.records()):
        rec_dict = dict(zip(field_names, record))
        
        # Extract key fields
        province_name = rec_dict.get('name') or rec_dict.get('NAME') or rec_dict.get('gn_name')
        country_name = rec_dict.get('admin') or rec_dict.get('ADMIN') or rec_dict.get('sovereignt')
        province_type = rec_dict.get('type') or rec_dict.get('TYPE') or rec_dict.get('type_en') or 'Province'
        
        # Population data
        population = rec_dict.get('pop_est') or rec_dict.get('POP_EST') or rec_dict.get('pop') or 0
        if isinstance(population, str):
            try:
                population = int(population)
            except:
                population = 0
        
        # Calculate centroid from bounding box
        bbox = shape.bbox  # [minx, miny, maxx, maxy]
        longitude = (bbox[0] + bbox[2]) / 2.0
        latitude = (bbox[1] + bbox[3]) / 2.0
        
        # Get area
        area = rec_dict.get('area_sqkm') or rec_dict.get('AREA_SQKM') or 0
        if isinstance(area, str):
            try:
                area = float(area)
            except:
                area = 0
        
        # ISO code if available
        iso_code = rec_dict.get('iso_3166_2') or rec_dict.get('ISO_3166_2') or rec_dict.get('code_local')
        
        if province_name and country_name:
            province_data = {
                "name": str(province_name),
                "country": str(country_name),
                "type": str(province_type),
                "population": int(population) if population else 0,
                "area_sqkm": float(area) if area else 0,
                "latitude": round(latitude, 4),
                "longitude": round(longitude, 4),
            }
            
            if iso_code:
                province_data["iso_code"] = str(iso_code)
            
            provinces.append(province_data)
    
    print(f"Extracted {len(provinces)} provinces/states")
    
    # Group by country for statistics
    by_country = {}
    for p in provinces:
        country = p['country']
        by_country.setdefault(country, []).append(p)
    
    print(f"\nProvinces per country (top 10):")
    sorted_countries = sorted(by_country.items(), key=lambda x: len(x[1]), reverse=True)
    for i, (country, provs) in enumerate(sorted_countries[:10], 1):
        print(f"  {i}. {country}: {len(provs)} provinces")
    
    # Save to JSON
    output_data = {
        "provinces": provinces,
        "count": len(provinces),
        "countries": len(by_country)
    }
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)
    
    print(f"\nSaved province data to: {output_path}")


if __name__ == "__main__":
    assets_dir = Path(__file__).parent.parent / "assets" / "data"
    output_dir = Path(__file__).parent.parent / "src" / "game" / "scenarios"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    provinces_shp = assets_dir / "ne_110m_admin_1_states_provinces"
    provinces_output = output_dir / "provinces.json"
    
    if provinces_shp.with_suffix('.shp').exists():
        print("=" * 60)
        print("Extracting provinces from Natural Earth admin_1 shapefile")
        print("=" * 60)
        extract_provinces(str(provinces_shp), str(provinces_output))
    else:
        print(f"ERROR: Provinces shapefile not found at {provinces_shp}")
    
    print("\n" + "=" * 60)
    print("Done!")
    print("=" * 60)
