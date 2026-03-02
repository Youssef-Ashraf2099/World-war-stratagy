/// Unit tests for Economic subsystem
/// 
/// Tests cover:
/// - GDP calculations
/// - Resource production
/// - Trade mechanics
/// - Economic stress

use bevy_ecs::prelude::*;
use crate::core::types::{
	EconomicStress, GDP, Infrastructure, Legitimacy, Logistics, Nation, NationId, NotificationLog,
	NotificationSeverity, Population, Province, ProvinceId, ResourceType, Resources, OwnedBy,
	MilitaryCapacity,
};
use crate::subsystems::economic::EconomicPhase;
use crate::core::tick::TickPhase;
use glam::Vec2;

#[test]
fn test_economic_phase_updates_gdp_and_stress() {
	let mut world = World::new();
	let nation_id = NationId::new();

	world.spawn((
		Nation {
			id: nation_id,
			name: "EconomyNation".to_string(),
			color: [100, 100, 200],
		},
		MilitaryCapacity::default(),
		Logistics::default(),
		GDP {
			value: 100.0,
			growth_rate: 0.0,
		},
		EconomicStress::default(),
		Legitimacy::new(75.0),
		NotificationLog::default(),
	));

	world.spawn((
		Province {
			id: ProvinceId::new(),
			name: "IronProvince".to_string(),
			position: Vec2::ZERO,
			dominant_resource: ResourceType::Iron,
		},
		Resources::default(),
		Population {
			total: 2_000_000,
			growth_rate: 0.0,
		},
		Infrastructure {
			level: 5,
			max_level: 10,
		},
		OwnedBy { nation_id },
	));

	let mut phase = EconomicPhase::new();
	phase.execute(&mut world);

	let mut nation_query = world.query::<(&GDP, &EconomicStress)>();
	let (gdp, stress) = nation_query.iter(&world).next().unwrap();

	assert!(gdp.value > 100.0, "GDP should grow from resource production");
	assert!(stress.gdp > 0.0, "Economic stress should track current GDP");
}

#[test]
fn test_economic_phase_emits_crisis_notifications_for_critical_values() {
	let mut world = World::new();
	let nation_id = NationId::new();

	let nation_entity = world.spawn((
		Nation {
			id: nation_id,
			name: "CrisisNation".to_string(),
			color: [220, 40, 40],
		},
		MilitaryCapacity::default(),
		Logistics::default(),
		GDP {
			value: 4.0,
			growth_rate: 0.0,
		},
		EconomicStress {
			current_deficit: 30.0,
			accumulated_deficit: 60.0,
			gdp: 4.0,
		},
		Legitimacy::new(20.0),
		NotificationLog::default(),
	)).id();

	let mut phase = EconomicPhase::new();
	phase.execute(&mut world);

	let log = world.get::<NotificationLog>(nation_entity).unwrap();
	assert!(
		log.notifications.iter().any(|n| n.severity == NotificationSeverity::Warning),
		"Economic crisis warning should be created"
	);
	assert!(
		log.notifications.iter().any(|n| n.severity == NotificationSeverity::Critical),
		"Legitimacy crisis critical notification should be created"
	);
}
