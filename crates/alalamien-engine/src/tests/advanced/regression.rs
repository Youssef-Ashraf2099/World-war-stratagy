/// Regression tests
/// 
/// These tests verify that critical functionality doesn't break
/// when making changes. Each test validates a feature that was
/// previously broken and fixed.

use crate::tests::fixtures::TestWorldBuilder;
use crate::core::types::{Nation, NotificationLog, NotificationType};
use crate::subsystems::{nuclear, warfare};

#[test]
fn test_world_state_persists_ticks() {
    // Regression: Feature - tick count increments correctly
    // Past issue: Tick not advancing, world stuck at tick 0
    let mut world = TestWorldBuilder::new()
        .with_seed(4000)
        .with_nations(2)
        .build();

    assert_eq!(world.current_tick(), 0);
    world.execute_ticks(1);
    assert_eq!(world.current_tick(), 1);
    world.execute_ticks(10);
    assert_eq!(world.current_tick(), 11);
}

#[test]
fn test_multiple_executions_additive() {
    // Regression: Multiple execute_ticks calls should add up
    let mut world = TestWorldBuilder::new()
        .with_seed(4001)
        .with_nations(3)
        .build();

    world.execute_ticks(25);
    let tick_after_first = world.current_tick();
    world.execute_ticks(25);
    let tick_after_second = world.current_tick();

    assert_eq!(tick_after_first, 25);
    assert_eq!(tick_after_second, 50);
}

#[test]
fn test_nation_spawning_preserves_state() {
    // Regression: Spawned nations should exist in world
    let mut world = crate::core::WorldState::new(4002);

    world.spawn_nation(
        "Test Nation".to_string(),
        [255, 0, 0],
        false,
    );

    // Verify nation count
    let count = world.world.query::<&crate::core::types::Nation>()
        .iter(&world.world)
        .count();
    assert_eq!(count, 1);
}

#[test]
fn test_multiple_nations_coexist() {
    // Regression: Multiple nations should coexist without conflict
    let fixture = TestWorldBuilder::new()
        .with_seed(4003)
        .with_nations(5)
        .build();

    assert_eq!(fixture.nation_count(), 5);
}

#[test]
fn test_determinism_same_seed_same_ticks() {
    // Regression: Same seed should produce same outcome
    let mut world1 = TestWorldBuilder::new()
        .with_seed(4004)
        .with_nations(3)
        .build();

    let mut world2 = TestWorldBuilder::new()
        .with_seed(4004) // Same seed
        .with_nations(3)
        .build();

    world1.execute_ticks(50);
    world2.execute_ticks(50);

    // Both should have same tick count
    assert_eq!(world1.current_tick(), world2.current_tick());
    assert_eq!(world1.nation_count(), world2.nation_count());
}

#[test]
fn test_v0_6_pipeline_exists() {
    // Regression: V0.6 pipeline should be available and executable
    use crate::core::tick::TickPipeline;
    use crate::tests::fixtures::TestWorldBuilder;
    
    let mut fixture = TestWorldBuilder::new()
        .with_nations(1)
        .build();
    
    let mut pipeline = TickPipeline::new_v0_6();
    pipeline.execute(&mut fixture.world);
    assert_eq!(fixture.world.current_tick(), 1);
}

#[test]
fn test_no_panic_on_extended_execution() {
    // Regression: Extended execution should not panic
    let mut world = TestWorldBuilder::new()
        .with_seed(4005)
        .with_nations(2)
        .build();

    // Should not panic
    for _ in 0..10 {
        world.execute_ticks(10);
    }

    assert_eq!(world.current_tick(), 100);
}

#[test]
fn test_regression_war_declaration_emits_notifications() {
    let mut fixture = TestWorldBuilder::new().with_seed(4010).build();

    let attacker = fixture.world.spawn_nation("Attacker".to_string(), [255, 0, 0], false);
    let defender = fixture.world.spawn_nation("Defender".to_string(), [0, 255, 0], false);
    let attacker_id = fixture.world.world.get::<Nation>(attacker).unwrap().id;
    let defender_id = fixture.world.world.get::<Nation>(defender).unwrap().id;

    warfare::declare_war(
        &mut fixture.world.world,
        attacker_id,
        defender_id,
        crate::core::types::CasusBelli::PreemptiveStrike,
        crate::core::types::WarGoal::Total,
        10,
    );

    let log_attacker = fixture.world.world.get::<NotificationLog>(attacker).unwrap();
    let log_defender = fixture.world.world.get::<NotificationLog>(defender).unwrap();

    assert!(
        log_attacker.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::WarDeclared { .. })),
        "Attacker should receive war declaration notification"
    );
    assert!(
        log_defender.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::WarDeclared { .. })),
        "Defender should receive war declaration notification"
    );
}

#[test]
fn test_regression_nuclear_use_notifies_bystanders() {
    let mut fixture = TestWorldBuilder::new().with_seed(4011).build();

    let attacker = fixture.world.spawn_nation_with_nuclear("NukeA".to_string(), [255, 0, 0], false, Some(60.0));
    let target = fixture.world.spawn_nation_with_nuclear("NukeB".to_string(), [0, 255, 0], false, Some(60.0));
    let bystander = fixture.world.spawn_nation("Observer".to_string(), [0, 0, 255], false);

    let attacker_id = fixture.world.world.get::<Nation>(attacker).unwrap().id;
    let target_id = fixture.world.world.get::<Nation>(target).unwrap().id;

    nuclear::apply_nuclear_use_effects(
        &mut fixture.world.world,
        attacker_id,
        target_id,
        vec![],
        20,
    );

    let bystander_log = fixture.world.world.get::<NotificationLog>(bystander).unwrap();
    assert!(
        bystander_log.notifications.iter().any(|n| matches!(n.notification_type, NotificationType::NuclearWeaponUsed { .. })),
        "Bystander should receive global nuclear-use notification"
    );
}
