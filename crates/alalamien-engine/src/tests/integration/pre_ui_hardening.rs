//! Pre-UI hardening integration tests
//!
//! These scenarios validate that major subsystems interact safely and emit
//! meaningful state transitions before UI implementation starts.

use crate::core::tick::{TickPhase, TickPipeline};
use crate::core::types::{
    Alliance, AllianceDoctrine, AllianceId, Nation, NotificationLog, NotificationType,
    EconomicStress, GDP, Legitimacy, Logistics, MilitaryCapacity,
};
use crate::subsystems::{alliance::AlliancePhase, economic::EconomicPhase, warfare, nuclear};
use crate::tests::fixtures::TestWorldBuilder;

#[test]
fn test_warfare_and_nuclear_actions_emit_notifications() {
    let mut fixture = TestWorldBuilder::new().with_seed(8100).build();

    let attacker = fixture.world.spawn_nation_with_nuclear(
        "Attacker".to_string(),
        [255, 0, 0],
        false,
        Some(50.0),
    );
    let defender = fixture.world.spawn_nation_with_nuclear(
        "Defender".to_string(),
        [0, 255, 0],
        false,
        Some(40.0),
    );
    let bystander = fixture.world.spawn_nation("Bystander".to_string(), [0, 0, 255], false);

    let attacker_id = fixture.world.world.get::<Nation>(attacker).unwrap().id;
    let defender_id = fixture.world.world.get::<Nation>(defender).unwrap().id;

    warfare::declare_war(
        &mut fixture.world.world,
        attacker_id,
        defender_id,
        crate::core::types::CasusBelli::PreemptiveStrike,
        crate::core::types::WarGoal::Total,
        1,
    );

    nuclear::apply_nuclear_use_effects(
        &mut fixture.world.world,
        attacker_id,
        defender_id,
        vec![],
        2,
    );

    let attacker_log = fixture.world.world.get::<NotificationLog>(attacker).unwrap();
    let bystander_log = fixture.world.world.get::<NotificationLog>(bystander).unwrap();

    assert!(
        attacker_log.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::WarDeclared { .. })),
        "War declaration should generate notification for attacker"
    );
    assert!(
        attacker_log.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::NuclearWeaponUsed { .. })),
        "Nuclear use should generate notification for attacker"
    );
    assert!(
        bystander_log.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::NuclearWeaponUsed { .. })),
        "Nuclear use should notify bystanders globally"
    );
}

#[test]
fn test_alliance_dissolution_notifies_members_and_despawns_alliance() {
    let mut fixture = TestWorldBuilder::new().with_seed(8101).build();

    let n1 = fixture.world.spawn_nation("N1".to_string(), [200, 0, 0], false);
    let n2 = fixture.world.spawn_nation("N2".to_string(), [0, 200, 0], false);
    let id1 = fixture.world.world.get::<Nation>(n1).unwrap().id;
    let id2 = fixture.world.world.get::<Nation>(n2).unwrap().id;

    fixture.world.world.spawn(Alliance {
        alliance_id: AllianceId::new(),
        alliance_name: "Fragile Pact".to_string(),
        founding_nation: id1,
        members: vec![id1, id2],
        cohesion: 10.0,
        doctrine: AllianceDoctrine::DefensiveAgreement,
        founded_tick: 0,
        threat_reduction: 0.1,
        cohesion_decay_rate: 1.0,
    });

    let mut phase = AlliancePhase::new();
    phase.execute(&mut fixture.world.world);

    let alliance_count = fixture.world.world.query::<&Alliance>().iter(&fixture.world.world).count();
    assert_eq!(alliance_count, 0, "Dissolved alliance should be despawned");

    let log1 = fixture.world.world.get::<NotificationLog>(n1).unwrap();
    let log2 = fixture.world.world.get::<NotificationLog>(n2).unwrap();
    assert!(
        log1.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::AllianceBroken { .. })),
        "Alliance dissolution should notify member 1"
    );
    assert!(
        log2.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::AllianceBroken { .. })),
        "Alliance dissolution should notify member 2"
    );
}

#[test]
fn test_economic_crisis_path_emits_both_crisis_notifications() {
    let mut fixture = TestWorldBuilder::new().with_seed(8102).build();

    let nation = fixture.world.spawn_nation("CrisisState".to_string(), [180, 90, 20], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;

    fixture.world.world.entity_mut(nation).insert((
        GDP { value: 4.0, growth_rate: 0.0 },
        EconomicStress {
            current_deficit: 30.0,
            accumulated_deficit: 100.0,
            gdp: 4.0,
        },
        Legitimacy::new(20.0),
        MilitaryCapacity::default(),
        Logistics::default(),
    ));

    let mut phase = EconomicPhase::new();
    phase.execute(&mut fixture.world.world);

    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    assert!(
        log.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::EconomicCrisis { nation: id, .. } if id == nation_id)),
        "Economic crisis notification should be emitted"
    );
    assert!(
        log.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::LegitimacyCrisis { nation: id, .. } if id == nation_id)),
        "Legitimacy crisis notification should be emitted"
    );
}

#[test]
fn test_v0_7_system_stability_and_notification_bounds() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(8103)
        .with_nations(12)
        .build();

    let mut pipeline = TickPipeline::new_v0_7();
    pipeline.execute_many(&mut fixture.world, 250);

    assert_eq!(fixture.world.current_tick(), 250);

    let mut query = fixture.world.world.query::<&NotificationLog>();
    for log in query.iter(&fixture.world.world) {
        assert!(
            log.notifications.len() <= 1000,
            "Notification log should never exceed pruning limit"
        );
    }
}
