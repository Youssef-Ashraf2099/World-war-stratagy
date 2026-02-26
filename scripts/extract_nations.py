#!/usr/bin/env python3
"""
Spirits of Steel - Natural Earth Shapefile Extraction Script
Converts Natural Earth shapefiles to JSON format for game loading
"""

import json
import sys
from pathlib import Path

try:
    from osgeo import ogr
    GDAL_AVAILABLE = True
except ImportError:
    GDAL_AVAILABLE = False

try:
    from dbfread import DBF
    DBFREAD_AVAILABLE = True
except ImportError:
    DBFREAD_AVAILABLE = False


def extract_with_gdal(shapefile_path, output_json_path):
    """Extract shapefile attributes using GDAL/OGR"""
    if not GDAL_AVAILABLE:
        print("ERROR: GDAL required for this method")
        return False

    print(f"Loading shapefile: {shapefile_path}")
    driver = ogr.GetDriverByName('ESRI Shapefile')

    if not driver:
        print("ERROR: ESRI Shapefile driver not found")
        return False

    datasource = driver.Open(str(shapefile_path))
    if not datasource:
        print(f"ERROR: Could not open {shapefile_path}")
        return False

    layer = datasource.GetLayer(0)
    layer_def = layer.GetLayerDefn()

    # Get available field names
    field_names = []
    for i in range(layer_def.GetFieldCount()):
        field_names.append(layer_def.GetFieldDefn(i).GetName())

    print(f"Found {len(field_names)} attributes")
    print(f"Attributes: {', '.join(field_names[:10])}...")

    nations = []
    feature_count = layer.GetFeatureCount()
    print(f"Processing {feature_count} features...")

    for feature_num in range(feature_count):
        feature = layer.GetFeature(feature_num)
        if not feature:
            continue

        nation = {
            'id': feature_num + 1,
            'name': feature.GetField('NAME') or 'Unknown',
        }

        # Add population if available
        if 'POP_EST' in field_names:
            pop = feature.GetField('POP_EST')
            nation['population'] = int(pop) if pop else 0

        # Add GDP if available
        if 'GDP_MD_EST' in field_names:
            gdp = feature.GetField('GDP_MD_EST')
            nation['gdp'] = float(gdp) * 1e6 if gdp else 0

        # Add continent
        if 'CONTINENT' in field_names:
            nation['continent'] = feature.GetField('CONTINENT') or 'Unknown'

        # Add ISO code
        if 'ISO_A2' in field_names:
            nation['code'] = feature.GetField('ISO_A2') or 'XX'

        # Add formal name if available
        if 'FORMAL_EN' in field_names:
            formal = feature.GetField('FORMAL_EN')
            if formal:
                nation['formal_name'] = formal

        nations.append(nation)

    datasource = None

    # Write JSON
    print(f"Writing {len(nations)} nations to {output_json_path}")
    with open(output_json_path, 'w', encoding='utf-8') as f:
        json.dump(nations, f, indent=2, ensure_ascii=False)

    print(f"✓ Successfully extracted {len(nations)} nations")
    return True


def extract_from_csv(csv_path, output_json_path):
    """Fallback: Extract from manually exported CSV"""
    import csv

    print(f"Loading CSV: {csv_path}")
    nations = []

    try:
        with open(csv_path, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)

            for row_num, row in enumerate(reader, 1):
                nation = {
                    'id': row_num,
                    'name': row.get('NAME', 'Unknown'),
                }

                # Add population
                pop = row.get('POP_EST', 0)
                nation['population'] = int(float(pop)) if pop else 0

                # Add GDP (convert from millions to actual)
                gdp = row.get('GDP_MD_EST', 0)
                nation['gdp'] = float(gdp) * 1e6 if gdp else 0

                # Add continent
                nation['continent'] = row.get('CONTINENT', 'Unknown')

                # Add code
                nation['code'] = row.get('ISO_A2', 'XX')

                nations.append(nation)

        print(f"Extracted {len(nations)} nations from CSV")

        with open(output_json_path, 'w', encoding='utf-8') as f:
            json.dump(nations, f, indent=2, ensure_ascii=False)

        print(f"✓ Wrote {len(nations)} nations to {output_json_path}")
        return True

    except Exception as e:
        print(f"ERROR reading CSV: {e}")
        return False


def extract_from_dbf(dbf_path, output_json_path):
    """Extract from shapefile's DBF file directly"""
    if not DBFREAD_AVAILABLE:
        print("ERROR: dbfread not available. Install with: pip install dbfread")
        return False

    print(f"Loading DBF file: {dbf_path}")

    try:
        table = DBF(str(dbf_path), encoding='utf-8', ignore_missing_memofile=True)

        print(f"Found {len(table)} records")
        print(f"Columns: {', '.join(table.field_names)}")

        nations = []
        for idx, record in enumerate(table, 1):
            nation = {
                'id': idx,
                'name': record.get('NAME', 'Unknown'),
            }

            # Add population
            pop = record.get('POP_EST', 0)
            nation['population'] = int(pop) if pop else 0

            # Add GDP (convert from millions to actual)
            gdp = record.get('GDP_MD', record.get('GDP_MD_EST', 0))
            nation['gdp'] = float(gdp) * 1e6 if gdp else 0

            # Add continent
            nation['continent'] = record.get('CONTINENT', 'Unknown')

            # Add ISO code
            nation['code'] = record.get('ISO_A2', 'XX')

            # Add formal name if available
            formal = record.get('FORMAL_EN')
            if formal:
                nation['formal_name'] = formal

            # Add economy type
            economy = record.get('ECONOMY')
            if economy:
                nation['economy'] = economy

            # Add income group
            income = record.get('INCOME_GRP')
            if income:
                nation['income_group'] = income

            nations.append(nation)

        print(f"Extracted {len(nations)} nations from DBF")

        # Ensure output directory exists
        output_json_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_json_path, 'w', encoding='utf-8') as f:
            json.dump(nations, f, indent=2, ensure_ascii=False)

        print(f"✓ Wrote {len(nations)} nations to {output_json_path}")
        return True

    except Exception as e:
        print(f"ERROR reading DBF: {e}")
        import traceback
        traceback.print_exc()
        return False


def main():
    print("=" * 60)
    print("Alalamien War - Shapefile Extraction Tool")
    print("=" * 60)
    print()

    # Define paths
    project_root = Path(__file__).parent.parent
    shapefile = project_root / "assets" / "data" / "ne_110m_admin_0_countries.shp"
    dbf_file = project_root / "assets" / "data" / "ne_110m_admin_0_countries.dbf"
    output_json = project_root / "src" / "game" / "scenarios" / "nations.json"

    # Try GDAL first
    if GDAL_AVAILABLE and shapefile.exists():
        print("Method: GDAL/OGR (native shapefile parsing)\n")
        success = extract_with_gdal(str(shapefile), str(output_json))
        if success:
            print("\n✓ Extraction complete!")
            print(f"Output: {output_json}")
            return 0

    # Try DBF extraction
    if DBFREAD_AVAILABLE and dbf_file.exists():
        print("Method: DBF direct read (shapefile attribute table)\n")
        success = extract_from_dbf(dbf_file, output_json)
        if success:
            print("\n✓ Extraction complete!")
            print(f"Output: {output_json}")
            return 0

    # Fallback to CSV
    csv_file = project_root / "assets" / "data" / "countries.csv"
    if csv_file.exists():
        print("Method: CSV fallback (manually exported)\n")
        success = extract_from_csv(str(csv_file), str(output_json))
        if success:
            print("\n✓ Extraction complete!")
            print(f"Output: {output_json}")
            return 0

    print("\nERROR: Could not find shapefile, DBF, or CSV")
    print(f"Expected shapefile: {shapefile}")
    print(f"Expected DBF: {dbf_file}")
    print(f"Expected CSV: {csv_file}")
    print("\nTo fix:")
    print("1. Install GDAL: pip install gdal")
    print("2. Install dbfread: pip install dbfread")
    print("3. OR export shapefile to CSV manually using QGIS:")
    print("   - Open ne_110m_admin_0_countries.shp in QGIS")
    print("   - Right-click layer → Export → Save as CSV")
    print("   - Save to: assets/data/countries.csv")
    return 1


if __name__ == '__main__':
    sys.exit(main())
