//! Province adjacency graph for border relationships

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::core::types::ProvinceId;

/// Province adjacency graph storing border relationships
#[derive(Debug, Clone, Default, Serialize, Deserialize, Resource)]
pub struct ProvinceGraph {
    /// Adjacency list: province_id -> set of neighboring province_ids
    adjacency: HashMap<ProvinceId, HashSet<ProvinceId>>,
}

impl ProvinceGraph {
    /// Create a new empty province graph
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    /// Add a bidirectional border between two provinces
    pub fn add_border(&mut self, province_a: ProvinceId, province_b: ProvinceId) {
        // Add A -> B
        self.adjacency
            .entry(province_a)
            .or_insert_with(HashSet::new)
            .insert(province_b);

        // Add B -> A (bidirectional)
        self.adjacency
            .entry(province_b)
            .or_insert_with(HashSet::new)
            .insert(province_a);
    }

    /// Remove a border between two provinces
    pub fn remove_border(&mut self, province_a: ProvinceId, province_b: ProvinceId) {
        if let Some(neighbors) = self.adjacency.get_mut(&province_a) {
            neighbors.remove(&province_b);
        }
        if let Some(neighbors) = self.adjacency.get_mut(&province_b) {
            neighbors.remove(&province_a);
        }
    }

    /// Get all neighbors of a province
    pub fn get_neighbors(&self, province_id: ProvinceId) -> Vec<ProvinceId> {
        self.adjacency
            .get(&province_id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Check if two provinces share a border
    pub fn are_neighbors(&self, province_a: ProvinceId, province_b: ProvinceId) -> bool {
        self.adjacency
            .get(&province_a)
            .map(|neighbors| neighbors.contains(&province_b))
            .unwrap_or(false)
    }

    /// Get the number of neighbors for a province
    pub fn neighbor_count(&self, province_id: ProvinceId) -> usize {
        self.adjacency
            .get(&province_id)
            .map(|neighbors| neighbors.len())
            .unwrap_or(0)
    }

    /// Get all provinces in the graph
    pub fn all_provinces(&self) -> Vec<ProvinceId> {
        self.adjacency.keys().copied().collect()
    }

    /// Get total number of provinces
    pub fn province_count(&self) -> usize {
        self.adjacency.len()
    }

    /// Get total number of borders (edges)
    pub fn border_count(&self) -> usize {
        self.adjacency
            .values()
            .map(|neighbors| neighbors.len())
            .sum::<usize>()
            / 2 // Divide by 2 since borders are bidirectional
    }

    /// Clear all borders
    pub fn clear(&mut self) {
        self.adjacency.clear();
    }

    /// Find provinces with no neighbors (isolated)
    pub fn find_isolated(&self) -> Vec<ProvinceId> {
        self.adjacency
            .iter()
            .filter(|(_, neighbors)| neighbors.is_empty())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get provinces that border a specific nation (via province ownership)
    /// Note: Requires access to world to check OwnedBy components
    pub fn provinces_by_neighbor_count(&self) -> Vec<(ProvinceId, usize)> {
        let mut provinces: Vec<_> = self.adjacency
            .iter()
            .map(|(id, neighbors)| (*id, neighbors.len()))
            .collect();
        provinces.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by neighbor count descending
        provinces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_border() {
        let mut graph = ProvinceGraph::new();
        let p1 = ProvinceId::new();
        let p2 = ProvinceId::new();

        graph.add_border(p1, p2);

        assert!(graph.are_neighbors(p1, p2));
        assert!(graph.are_neighbors(p2, p1)); // Bidirectional
        assert_eq!(graph.neighbor_count(p1), 1);
        assert_eq!(graph.neighbor_count(p2), 1);
    }

    #[test]
    fn test_remove_border() {
        let mut graph = ProvinceGraph::new();
        let p1 = ProvinceId::new();
        let p2 = ProvinceId::new();

        graph.add_border(p1, p2);
        graph.remove_border(p1, p2);

        assert!(!graph.are_neighbors(p1, p2));
        assert_eq!(graph.neighbor_count(p1), 0);
    }

    #[test]
    fn test_multiple_neighbors() {
        let mut graph = ProvinceGraph::new();
        let p1 = ProvinceId::new();
        let p2 = ProvinceId::new();
        let p3 = ProvinceId::new();
        let p4 = ProvinceId::new();

        // Create a chain: p1 - p2 - p3 - p4
        graph.add_border(p1, p2);
        graph.add_border(p2, p3);
        graph.add_border(p3, p4);

        assert_eq!(graph.neighbor_count(p1), 1);
        assert_eq!(graph.neighbor_count(p2), 2); // Connected to p1 and p3
        assert_eq!(graph.neighbor_count(p3), 2); // Connected to p2 and p4
        assert_eq!(graph.neighbor_count(p4), 1);
    }

    #[test]
    fn test_get_neighbors() {
        let mut graph = ProvinceGraph::new();
        let p1 = ProvinceId::new();
        let p2 = ProvinceId::new();
        let p3 = ProvinceId::new();

        graph.add_border(p1, p2);
        graph.add_border(p1, p3);

        let neighbors = graph.get_neighbors(p1);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&p2));
        assert!(neighbors.contains(&p3));
    }

    #[test]
    fn test_border_count() {
        let mut graph = ProvinceGraph::new();
        let p1 = ProvinceId::new();
        let p2 = ProvinceId::new();
        let p3 = ProvinceId::new();

        graph.add_border(p1, p2);
        graph.add_border(p2, p3);
        graph.add_border(p1, p3);

        assert_eq!(graph.border_count(), 3);
        assert_eq!(graph.province_count(), 3);
    }

    #[test]
    fn test_find_isolated() {
        let mut graph = ProvinceGraph::new();
        let p1 = ProvinceId::new();
        let p2 = ProvinceId::new();
        let p3 = ProvinceId::new();

        graph.add_border(p1, p2);
        // p3 is isolated (no borders added)
        graph.adjacency.insert(p3, HashSet::new());

        let isolated = graph.find_isolated();
        assert_eq!(isolated.len(), 1);
        assert!(isolated.contains(&p3));
    }
}
