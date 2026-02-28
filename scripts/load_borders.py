#!/usr/bin/env python3
"""
Load border data into the engine and populate the province graph.

Usage:
    python scripts/load_borders.py

This will:
1. Read borders.json
2. Match borders to existing nations
3. Generate Rust code to initialize the province graph
"""

import json
from pathlib import Path


def load_borders(borders_path: str, nations_path: str, output_path: str):
    """
    Load border data and generate Rust initialization code.
    
    Args:
        borders_path: Path to borders.json
        nations_path: Path to nations.json (to match nation names)
        output_path: Where to save generated Rust code
    """
    # Load borders
    with open(borders_path, 'r', encoding='utf-8') as f:
        borders_data = json.load(f)
    
    # Load nations to get name mapping
    with open(nations_path, 'r', encoding='utf-8') as f:
        nations_data = json.load(f)

    if isinstance(nations_data, list):
        nation_names = {n['name'] for n in nations_data}
    else:
        nation_names = {n['name'] for n in nations_data.get('nations', [])}
    
    print(f"Loaded {len(borders_data['borders'])} borders")
    print(f"Loaded {len(nation_names)} nations")
    
    # Filter borders to only include nations we have
    valid_borders = []
    for border in borders_data['borders']:
        if border['country_a'] in nation_names and border['country_b'] in nation_names:
            valid_borders.append(border)
    
    print(f"Matched {len(valid_borders)} borders to existing nations")
    
    # Generate Rust code
    rust_code = f"""// Auto-generated border data from Natural Earth
// Generated from: {Path(borders_path).name}
// Total borders: {len(valid_borders)}

use std::collections::HashMap;

/// Load pre-computed nation adjacency data
pub fn get_nation_borders() -> Vec<(&'static str, &'static str)> {{
    vec![
"""
    
    for border in valid_borders:
        rust_code += f'        ("{border["country_a"]}", "{border["country_b"]}"),\n'
    
    rust_code += """    ]
}

/// Build a map from nation name to list of neighboring nations
pub fn build_nation_adjacency_map() -> HashMap<String, Vec<String>> {
    let borders = get_nation_borders();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    
    for (country_a, country_b) in borders {
        map.entry(country_a.to_string())
            .or_insert_with(Vec::new)
            .push(country_b.to_string());
        
        map.entry(country_b.to_string())
            .or_insert_with(Vec::new)
            .push(country_a.to_string());
    }
    
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_count() {
        let borders = get_nation_borders();
        assert!(!borders.is_empty(), "Should have border data");
    }

    #[test]
    fn test_adjacency_map() {
        let map = build_nation_adjacency_map();
        assert!(!map.is_empty(), "Adjacency map should not be empty");
        
        // Test that borders are bidirectional
        for (country, neighbors) in &map {
            for neighbor in neighbors {
                let neighbor_list = map.get(neighbor).unwrap();
                assert!(
                    neighbor_list.contains(country),
                    "Border {} <-> {} should be bidirectional",
                    country,
                    neighbor
                );
            }
        }
    }
}
"""
    
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write(rust_code)
    
    print(f"Generated Rust code at: {output_path}")
    print(f"\nSample borders:")
    for i, border in enumerate(valid_borders[:10], 1):
        print(f"  {i}. {border['country_a']} <-> {border['country_b']}")


if __name__ == "__main__":
    project_root = Path(__file__).parent.parent
    
    borders_path = project_root / "src" / "game" / "scenarios" / "borders.json"
    borders_fallback_path = project_root / "src" / "game" / "scenarios" / "borders_from_countries.json"
    nations_path = project_root / "src" / "game" / "scenarios" / "nations.json"
    output_path = project_root / "crates" / "alalamien-engine" / "src" / "game" / "borders.rs"
    
    if not borders_path.exists():
        print(f"ERROR: {borders_path} not found")
        print("Run extract_borders.py first to generate border data")
        exit(1)
    
    if not nations_path.exists():
        print(f"ERROR: {nations_path} not found")
        exit(1)

    # If primary borders file is empty, use extracted fallback
    with open(borders_path, 'r', encoding='utf-8') as f:
        primary_borders = json.load(f)
    if not primary_borders.get('borders'):
        if borders_fallback_path.exists():
            print(f"Primary borders file is empty, using fallback: {borders_fallback_path.name}")
            borders_path = borders_fallback_path
        else:
            print("ERROR: borders.json is empty and no fallback borders file found")
            exit(1)
    
    load_borders(str(borders_path), str(nations_path), str(output_path))
    
    print("\n" + "=" * 60)
    print("Next steps:")
    print("1. Add 'pub mod borders;' to src/game/mod.rs")
    print("2. Use borders::build_nation_adjacency_map() in WorldState initialization")
    print("3. When spawning provinces, use borders data to call ProvinceGraph::add_border()")
    print("=" * 60)
