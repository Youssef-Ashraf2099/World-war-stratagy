// Core types unit tests
// Tests for basic game types and components

use crate::core::types::*;

#[test]
fn test_nation_id_uniqueness() {
    let id1 = NationId::new();
    let id2 = NationId::new();
    
    // IDs should be different
    assert_ne!(id1, id2);
}

#[test]
fn test_gdp_component() {
    let mut gdp = GDP {
        value: 1000.0,
        growth_rate: 0.02,
    };
    
    assert_eq!(gdp.value, 1000.0);
    assert_eq!(gdp.growth_rate, 0.02);
    
    // Simulate growth
    gdp.value = (gdp.value * (1.0 + gdp.growth_rate)).round();
    assert!(gdp.value > 1000.0);
}

#[test]
fn test_resources_operations() {
    let mut resources = Resources::default();
    
    // Test get
    assert_eq!(resources.get(ResourceType::Food), 100.0);
    
    // Test set
    resources.set(ResourceType::Food, 150.0);
    assert_eq!(resources.get(ResourceType::Food), 150.0);
    
    // Test add
    resources.add(ResourceType::Food, 50.0);
    assert_eq!(resources.get(ResourceType::Food), 200.0);
}

#[test]
fn test_legitimacy_bounds() {
    let mut legitimacy = Legitimacy::new(50.0);
    
    // Test upper bound
    legitimacy.modify(100.0);
    assert_eq!(legitimacy.value, 100.0);
    
    // Test lower bound
    legitimacy.modify(-150.0);
    assert_eq!(legitimacy.value, 0.0);
}

#[test]
fn test_legitimacy_stability() {
    assert!(Legitimacy::new(50.0).is_stable());
    assert!(!Legitimacy::new(30.0).is_stable());
    assert!(Legitimacy::new(15.0).is_critical());
}
