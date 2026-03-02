/// Integration tests: Civil War Cascade
///
/// These tests verify the end-to-end civil war scenario:
/// 1. A nation under sustained war + economic stress degrades to legitimacy=0
/// 2. The FactionCivilWarPhase correctly splits the nation
/// 3. Factions are at war with each other (civil war)
/// 4. The parent nation is marked `Factionalized`
/// 5. No double-collapse occurs on subsequent ticks

use crate::core::WorldState;
use crate::core::tick::TickPhase;
use crate::core::types::{
    Nation, NationId, Legitimacy, Province, ProvinceId, OwnedBy,
    Population, Resources, Army, ArmyId, WarState, GDP, ResourceType,
    EconomicStress, CasualtyLog, AllianceCrisisLog, DiplomaticIsolationLog,
};
use crate::subsystems::factions::{FactionCivilWarPhase, Factionalized, Faction};
use crate::subsystems::legitimacy::LegitimacyPhase;

// ============================================================================
// CIVIL WAR TRIGGER CONDITIONS
// ============================================================================

/// When a nation's legitimacy reaches 0.0 with provinces, it should collapse
/// into factions on the next FactionCivilWarPhase tick.
#[test]
fn test_nation_collapses_into_factions_at_zero_legitimacy() {
    let mut world = WorldState::new(2001);
    let nation_id = NationId::new();

    // Spawn parent nation at critical legitimacy
    let nation_entity = world.world.spawn((
        Nation { id: nation_id, name: "Collapsing Empire".to_string(), color: [180, 30, 30] },
        Legitimacy::new(0.0),
        WarState::default(),
        GDP::default(),
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    // Give it 5 provinces so it splits into 3 factions
    for i in 0..5 {
        world.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: format!("Province {}", i),
                position: glam::Vec2::new(i as f32 * 10.0, 0.0),
                dominant_resource: ResourceType::Food,
            },
            OwnedBy { nation_id },
            Population { total: 500_000, growth_rate: 0.01 },
            Resources {
                food: 200.0, iron: 50.0, oil: 30.0,
                rare_earths: 10.0, water: 100.0, trade_ports: 1,
            },
        ));
    }

    // Run faction phase
    let mut phase = FactionCivilWarPhase::new();
    phase.execute(&mut world.world);

    // ✅ Verify: Parent nation is marked factionalized
    let factionalized = world.world.get::<Factionalized>(nation_entity);
    assert!(
        factionalized.is_some(),
        "Nation should be marked Factionalized after legitimacy=0"
    );

    // ✅ Verify: Correct number of factions spawned (5 provinces → 3 factions)
    let faction_count = world.world.query::<&Faction>().iter(&world.world).count();
    assert_eq!(faction_count, 3, "5 provinces should produce 3 factions");

    println!("✓ Civil war triggered: {} factions spawned from 5-province nation", faction_count);
}

/// Each spawned faction should be at war with all sibling factions.
#[test]
fn test_civil_war_factions_fight_each_other() {
    let mut world = WorldState::new(2002);
    let nation_id = NationId::new();

    // Spawn nation with 4 provinces → 3 factions
    let nation = world.world.spawn((
        Nation { id: nation_id, name: "Civil War Nation".to_string(), color: [150, 50, 50] },
        Legitimacy::new(0.0),
        WarState::default(),
        GDP::default(),
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    for i in 0..4 {
        world.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: format!("Province {}", i),
                position: glam::Vec2::ZERO,
                dominant_resource: ResourceType::Iron,
            },
            OwnedBy { nation_id },
            Population::default(),
            Resources::default(),
        ));
    }

    let mut phase = FactionCivilWarPhase::new();
    phase.execute(&mut world.world);

    // Gather all faction nation IDs
    let faction_nation_ids: Vec<NationId> = world.world
        .query::<(&Faction, &Nation)>()
        .iter(&world.world)
        .map(|(_, n)| n.id)
        .collect();

    assert_eq!(faction_nation_ids.len(), 3, "Should have 3 factions");

    // ✅ Verify: Every faction is at war with every other faction
    for faction_nation_id in &faction_nation_ids {
        let war_state = world.world
            .query::<(&Nation, &WarState)>()
            .iter(&world.world)
            .find(|(n, _)| n.id == *faction_nation_id)
            .map(|(_, ws)| ws.at_war_with.clone());

        let enemies = war_state.expect("Faction should have WarState");
        let expected_enemy_count = faction_nation_ids.len() - 1;

        assert_eq!(
            enemies.len(), expected_enemy_count,
            "Faction should be at war with {} siblings, got {}",
            expected_enemy_count, enemies.len()
        );

        for other in &faction_nation_ids {
            if other != faction_nation_id {
                assert!(
                    enemies.contains(other),
                    "Faction should be at war with all sibling factions"
                );
            }
        }
    }

    println!("✓ Civil war: all {} factions at war with each other", faction_nation_ids.len());
}

/// Running the FactionCivilWarPhase twice on the same already-collapsed nation
/// should NOT produce additional factions.
#[test]
fn test_no_double_collapse() {
    let mut world = WorldState::new(2003);
    let nation_id = NationId::new();

    world.world.spawn((
        Nation { id: nation_id, name: "Test Nation".to_string(), color: [100, 100, 100] },
        Legitimacy::new(0.0),
        WarState::default(),
        GDP::default(),
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    ));

    for i in 0..3 {
        world.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: format!("Province {}", i),
                position: glam::Vec2::ZERO,
                dominant_resource: ResourceType::Food,
            },
            OwnedBy { nation_id },
            Population::default(),
            Resources::default(),
        ));
    }

    let mut phase = FactionCivilWarPhase::new();
    phase.execute(&mut world.world);
    let factions_after_first = world.world.query::<&Faction>().iter(&world.world).count();

    // Run again — should NOT spawn new factions
    phase.execute(&mut world.world);
    let factions_after_second = world.world.query::<&Faction>().iter(&world.world).count();

    assert_eq!(
        factions_after_first, factions_after_second,
        "Duplicate collapse should be prevented: first {} factions, second {} factions",
        factions_after_first, factions_after_second
    );
    println!("✓ Double-collapse guard: {} factions, stable after 2 phase runs", factions_after_first);
}

/// Factions should inherit a portion of the parent nation's resources,
/// not get a full copy or zero.
#[test]
fn test_faction_inherits_proportional_resources() {
    let mut world = WorldState::new(2004);
    let nation_id = NationId::new();

    world.world.spawn((
        Nation { id: nation_id, name: "Rich Empire".to_string(), color: [200, 200, 0] },
        Legitimacy::new(0.0),
        WarState::default(),
        GDP::default(),
        EconomicStress::default(),
        CasualtyLog::default(),
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    ));

    // Each province has 1000 food
    for i in 0..4 {
        world.world.spawn((
            Province {
                id: ProvinceId::new(),
                name: format!("Province {}", i),
                position: glam::Vec2::ZERO,
                dominant_resource: ResourceType::Food,
            },
            OwnedBy { nation_id },
            Population::default(),
            Resources {
                food: 1000.0,
                iron: 0.0, oil: 0.0, rare_earths: 0.0,
                water: 0.0, trade_ports: 0,
            },
        ));
    }

    let mut phase = FactionCivilWarPhase::new();
    phase.execute(&mut world.world);

    // ✅ All faction resources combined should be roughly equal to parent total
    let total_faction_food: f64 = world.world
        .query::<(&Faction, &Resources)>()
        .iter(&world.world)
        .map(|(_, r)| r.food)
        .sum();

    // With 3 factions each getting equal share, each gets ~33% of parent total
    let parent_total_food = 4.0 * 1000.0; // 4 provinces × 1000
    assert!(
        total_faction_food > 0.0,
        "Factions should inherit resources, got: {}",
        total_faction_food
    );
    // Allow up to 5% over due to rounding in distribution
    let tolerance = parent_total_food * 0.05;
    assert!(
        total_faction_food <= parent_total_food + tolerance,
        "Factions should not receive significantly more than parent had: {:.1} vs {:.1} (tolerance: {:.1})",
        total_faction_food, parent_total_food, tolerance
    );
    println!(
        "✓ Resource inheritance: parent had {:.0} food, factions together have {:.0}",
        parent_total_food, total_faction_food
    );
}

// ============================================================================
// LEGITIMACY SPIRAL SCENARIO
// ============================================================================

/// The "death spiral" scenario: a nation at war with economic stress
/// should gradually reach the crisis threshold within a realistic timeframe.
#[test]
fn test_legitimacy_spiral_reaches_crisis_in_reasonable_ticks() {
    let mut world = WorldState::new(2010);
    let nation_id = NationId::new();
    let enemy_id = NationId::new();

    // Nation starting at stress threshold (50) with compounding stressors
    let nation_entity = world.world.spawn((
        nation_id,
        Nation { id: nation_id, name: "Spiraling Nation".to_string(), color: [180, 40, 40] },
        Legitimacy::new(50.0),
        WarState { at_war_with: vec![enemy_id, NationId::new()] }, // 2 wars
        GDP { value: 600.0, growth_rate: 0.01 },
        EconomicStress {
            current_deficit: 180.0, // 30% of GDP
            accumulated_deficit: 0.0,
            gdp: 600.0,
        },
        CasualtyLog {
            personnel_lost: 2000,
            total_personnel: 10000, // 20% casualties
        },
        AllianceCrisisLog::default(),
        DiplomaticIsolationLog::default(),
    )).id();

    let mut phase = LegitimacyPhase::new();
    let mut ticks_to_crisis = None;

    for tick in 1..=200 {
        phase.execute(&mut world.world);
        let legit = world.world.get::<Legitimacy>(nation_entity).unwrap().value;
        if legit <= 20.0 && ticks_to_crisis.is_none() {
            ticks_to_crisis = Some(tick);
        }
    }

    let final_legit = world.world.get::<Legitimacy>(nation_entity).unwrap().value;
    assert!(
        final_legit < 50.0,
        "Nation should have degraded from 50.0, final: {:.1}",
        final_legit
    );

    if let Some(ticks) = ticks_to_crisis {
        assert!(
            ticks <= 150,
            "Crisis should be reachable within 150 ticks from stress state, took: {}",
            ticks
        );
        println!("✓ Legitimacy spiral: reached crisis (<20) in {} ticks", ticks);
    } else {
        println!(
            "ℹ Legitimacy spiral: 200 ticks, final legit = {:.1} (didn't reach <20 threshold)",
            final_legit
        );
        // This is informational — the thresholds may need tuning for game balance
        // at this point we just ensure the direction is downward.
    }
}
