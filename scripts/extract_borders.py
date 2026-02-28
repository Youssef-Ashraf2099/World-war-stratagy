#!/usr/bin/env python3
"""
Extract border/adjacency data from Natural Earth shapefiles.

This script reads Natural Earth boundary line shapefiles and generates
a JSON file containing which countries/provinces share borders.

Output format:
{
    "borders": [
        {"country_a": "United States", "country_b": "Canada"},
        {"country_a": "France", "country_b": "Germany"},
        ...
    ]
}
"""

import json
import sys
from pathlib import Path
from collections import defaultdict

try:
    import shapefile
except ImportError:
    print("ERROR: pyshp library not found")
    print("Install it with: pip install pyshp")
    sys.exit(1)


def extract_country_borders(shapefile_path: str, output_path: str):
    """
    Extract country adjacency from Natural Earth boundary lines shapefile.
    
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
    print(f"Available fields: {field_names}")
    
    borders = []
    border_set = set()  # To avoid duplicates
    
    # Process each border line record
    for record in sf.records():
        rec_dict = dict(zip(field_names, record))
        
        # The boundary line shapefile has two countries for each border
        # Fields might be named differently in your version, adjust as needed
        country_a = None
        country_b = None
        
        # Try different possible field names
        for field_a, field_b in [
            ('name_left', 'name_right'),
            ('adm0_left', 'adm0_right'),
            ('name_en_left', 'name_en_right'),
            ('sovereignt', 'sov_a3'),
        ]:
            if field_a in rec_dict and field_b in rec_dict:
                country_a = rec_dict.get(field_a)
                country_b = rec_dict.get(field_b)
                if country_a and country_b:
                    break
        
        if country_a and country_b and country_a != country_b:
            # Normalize the pair (alphabetically) to avoid duplicates
            pair = tuple(sorted([str(country_a), str(country_b)]))
            if pair not in border_set:
                border_set.add(pair)
                borders.append({
                    "country_a": pair[0],
                    "country_b": pair[1]
                })
    
    print(f"Extracted {len(borders)} unique border relationships")
    
    # Save to JSON
    output_data = {
        "borders": borders,
        "count": len(borders)
    }
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)
    
    print(f"Saved border data to: {output_path}")
    
    # Print some statistics
    country_counts = defaultdict(int)
    for border in borders:
        country_counts[border["country_a"]] += 1
        country_counts[border["country_b"]] += 1
    
    print(f"\nTop 10 countries by number of neighbors:")
    sorted_countries = sorted(country_counts.items(), key=lambda x: x[1], reverse=True)
    for i, (country, count) in enumerate(sorted_countries[:10], 1):
        print(f"  {i}. {country}: {count} neighbors")


def extract_province_borders_from_countries(countries_shapefile: str, output_path: str):
    """
    Generate province-level borders by analyzing country polygon adjacency.
    
    For each country, we can generate internal provinces and figure out which
    countries are adjacent based on polygon geometry.
    """
    print(f"Reading countries shapefile: {countries_shapefile}")
    
    try:
        sf = shapefile.Reader(countries_shapefile)
    except Exception as e:
        print(f"ERROR: Could not read shapefile: {e}")
        return
    
    field_names = [field[0] for field in sf.fields[1:]]
    
    adjacency_map = defaultdict(set)
    
    # Build a spatial index of country bounding boxes
    country_boxes = []
    for idx, (shape, record) in enumerate(zip(sf.shapes(), sf.records())):
        rec_dict = dict(zip(field_names, record))
        country_name = rec_dict.get('NAME') or rec_dict.get('ADMIN') or rec_dict.get('name')
        
        if country_name:
            bbox = shape.bbox  # [minx, miny, maxx, maxy]
            country_boxes.append({
                'index': idx,
                'name': country_name,
                'bbox': bbox,
                'shape': shape
            })
    
    print(f"Processing {len(country_boxes)} countries for adjacency...")
    
    # Check for overlapping bounding boxes (potential neighbors)
    for i, country_a in enumerate(country_boxes):
        for country_b in country_boxes[i+1:]:
            # Check if bounding boxes overlap or touch
            if boxes_intersect(country_a['bbox'], country_b['bbox']):
                # They might be neighbors - add to adjacency list
                adjacency_map[country_a['name']].add(country_b['name'])
                adjacency_map[country_b['name']].add(country_a['name'])
    
    # Convert to border list
    borders = []
    seen = set()
    for country, neighbors in adjacency_map.items():
        for neighbor in neighbors:
            pair = tuple(sorted([country, neighbor]))
            if pair not in seen:
                seen.add(pair)
                borders.append({
                    "country_a": pair[0],
                    "country_b": pair[1]
                })
    
    print(f"Extracted {len(borders)} potential border relationships")
    
    output_data = {
        "borders": borders,
        "count": len(borders),
        "note": "Generated from country polygon adjacency (bounding box overlap)"
    }
    
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)
    
    print(f"Saved border data to: {output_path}")


def boxes_intersect(box_a, box_b):
    """Check if two bounding boxes intersect or touch."""
    # box format: [minx, miny, maxx, maxy]
    return not (box_a[2] < box_b[0] or  # a is left of b
                box_a[0] > box_b[2] or  # a is right of b
                box_a[3] < box_b[1] or  # a is below b
                box_a[1] > box_b[3])    # a is above b


if __name__ == "__main__":
    # Paths
    assets_dir = Path(__file__).parent.parent / "assets" / "data"
    output_dir = Path(__file__).parent.parent / "src" / "game" / "scenarios"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Try extracting from boundary lines first
    boundary_lines_shp = assets_dir / "ne_110m_admin_0_boundary_lines_land"
    borders_output = output_dir / "borders.json"
    
    if boundary_lines_shp.with_suffix('.shp').exists():
        print("=" * 60)
        print("Extracting borders from boundary lines shapefile")
        print("=" * 60)
        extract_country_borders(str(boundary_lines_shp), str(borders_output))
    else:
        print(f"WARNING: Boundary lines shapefile not found at {boundary_lines_shp}")
    
    # Also try from countries polygons
    countries_shp = assets_dir / "ne_110m_admin_0_countries"
    borders_alt_output = output_dir / "borders_from_countries.json"
    
    if countries_shp.with_suffix('.shp').exists():
        print("\n" + "=" * 60)
        print("Extracting borders from countries polygons (alternative method)")
        print("=" * 60)
        extract_province_borders_from_countries(str(countries_shp), str(borders_alt_output))
    
    print("\n" + "=" * 60)
    print("Done! Check the output files for border data.")
    print("=" * 60)
