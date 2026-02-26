//! Geospatial data loading from Natural Earth shapefiles

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Nation data loaded from Natural Earth shapefile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NationData {
    /// Unique ID
    pub id: u32,
    /// Nation name
    pub name: String,
    /// Estimated population
    pub population: u64,
    /// GDP (USD)
    pub gdp: f64,
    /// Continent
    pub continent: String,
    /// ISO 2-letter code
    pub code: String,
    /// Formal name (if different from name)
    #[serde(default)]
    pub formal_name: Option<String>,
    /// Economic classification
    #[serde(default)]
    pub economy: Option<String>,
    /// Income group classification
    #[serde(default)]
    pub income_group: Option<String>,
}

impl NationData {
    /// Load all nations from JSON file
    pub fn load_all(json_path: &Path) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let json_str = std::fs::read_to_string(json_path)?;
        let nations: Vec<NationData> = serde_json::from_str(&json_str)?;
        Ok(nations)
    }

    /// Get nation by ISO code
    pub fn find_by_code<'a>(nations: &'a [Self], code: &str) -> Option<&'a Self> {
        nations.iter().find(|n| n.code == code)
    }

    /// Get nation by name
    pub fn find_by_name<'a>(nations: &'a [Self], name: &str) -> Option<&'a Self> {
        nations.iter().find(|n| n.name.eq_ignore_ascii_case(name))
    }

    /// Get all nations in a continent
    pub fn by_continent<'a>(nations: &'a [Self], continent: &str) -> Vec<&'a Self> {
        nations
            .iter()
            .filter(|n| n.continent.eq_ignore_ascii_case(continent))
            .collect()
    }

    /// Get top N nations by population
    pub fn top_by_population(nations: &[Self], count: usize) -> Vec<Self> {
        let mut sorted = nations.to_vec();
        sorted.sort_by_key(|n| std::cmp::Reverse(n.population));
        sorted.into_iter().take(count).collect()
    }

    /// Get top N nations by GDP
    pub fn top_by_gdp(nations: &[Self], count: usize) -> Vec<Self> {
        let mut sorted = nations.to_vec();
        sorted.sort_by(|a, b| b.gdp.partial_cmp(&a.gdp).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nation_data_structure() {
        let nation = NationData {
            id: 1,
            name: "United States of America".to_string(),
            population: 328239523,
            gdp: 21433226000000.0,
            continent: "North America".to_string(),
            code: "US".to_string(),
            formal_name: Some("United States of America".to_string()),
            economy: Some("1. Developed region: G7".to_string()),
            income_group: Some("1. High income: OECD".to_string()),
        };

        assert_eq!(nation.code, "US");
        assert_eq!(nation.population, 328239523);
        assert!(nation.gdp > 21e12);
    }

    #[test]
    fn test_continent_filtering() {
        let nations = vec![
            NationData {
                id: 1,
                name: "USA".to_string(),
                population: 1000,
                gdp: 1e12,
                continent: "North America".to_string(),
                code: "US".to_string(),
                formal_name: None,
                economy: None,
                income_group: None,
            },
            NationData {
                id: 2,
                name: "China".to_string(),
                population: 2000,
                gdp: 1e12,
                continent: "Asia".to_string(),
                code: "CN".to_string(),
                formal_name: None,
                economy: None,
                income_group: None,
            },
        ];

        let north_america = NationData::by_continent(&nations, "North America");
        assert_eq!(north_america.len(), 1);
        assert_eq!(north_america[0].code, "US");
        
        let top = NationData::top_by_population(&nations, 1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].code, "CN");
    }
}
