//! Integration tests for Notifications System
//!
//! Tests notification generation and integration with game events:
//! - War notifications
//! - Alliance notifications
//! - Nuclear notifications
//! - Multi-nation scenarios
//! - Notification persistence

use crate::core::types::*;
use crate::core::tick::{TickPipeline, TickPhase};
use crate::subsystems::notifications::*;
use crate::tests::fixtures::TestWorldBuilder;

// ============================================================================
// WAR NOTIFICATION TESTS
// ============================================================================

#[test]
fn test_war_declaration_notifications() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create two nations
    let nation1 = fixture.world.spawn_nation(
        "Attacker".to_string(),
        [255, 0, 0],
        false,
    );
    
    let nation2 = fixture.world.spawn_nation(
        "Defender".to_string(),
        [0, 255, 0],
        false,
    );
    
    let nation1_id = fixture.world.world.get::<Nation>(nation1).unwrap().id;
    let nation2_id = fixture.world.world.get::<Nation>(nation2).unwrap().id;
    
    // Create war declaration notification
    create_war_notification(&mut fixture.world.world, nation1_id, nation2_id, 100);
    
    // Verify both nations received the notification
    let log1 = fixture.world.world.get::<NotificationLog>(nation1).unwrap();
    let log2 = fixture.world.world.get::<NotificationLog>(nation2).unwrap();
    
    assert_eq!(log1.notifications.len(), 1);
    assert_eq!(log2.notifications.len(), 1);
    assert_eq!(log1.notifications[0].severity, NotificationSeverity::Major);
}

#[test]
fn test_war_ended_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let victor = fixture.world.spawn_nation(
        "Victor".to_string(),
        [255, 0, 0],
        false,
    );
    
    let defeated = fixture.world.spawn_nation(
        "Defeated".to_string(),
        [0, 255, 0],
        false,
    );
    
    let victor_id = fixture.world.world.get::<Nation>(victor).unwrap().id;
    let defeated_id = fixture.world.world.get::<Nation>(defeated).unwrap().id;
    
    // Create war ended notification
    create_war_ended_notification(&mut fixture.world.world, Some(victor_id), defeated_id, 200);
    
    // Verify notifications
    let log_victor = fixture.world.world.get::<NotificationLog>(victor).unwrap();
    let log_defeated = fixture.world.world.get::<NotificationLog>(defeated).unwrap();
    
    assert_eq!(log_victor.notifications.len(), 1);
    assert_eq!(log_defeated.notifications.len(), 1);
}

// ============================================================================
// ALLIANCE NOTIFICATION TESTS
// ============================================================================

#[test]
fn test_alliance_formed_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation1 = fixture.world.spawn_nation("Nation1".to_string(), [255, 0, 0], false);
    let nation2 = fixture.world.spawn_nation("Nation2".to_string(), [0, 255, 0], false);
    let nation3 = fixture.world.spawn_nation("Nation3".to_string(), [0, 0, 255], false);
    
    let id1 = fixture.world.world.get::<Nation>(nation1).unwrap().id;
    let id2 = fixture.world.world.get::<Nation>(nation2).unwrap().id;
    let id3 = fixture.world.world.get::<Nation>(nation3).unwrap().id;
    
    // Create alliance notification
    create_alliance_notification(&mut fixture.world.world, vec![id1, id2, id3], 100);
    
    // Verify all alliance members received notification
    let log1 = fixture.world.world.get::<NotificationLog>(nation1).unwrap();
    let log2 = fixture.world.world.get::<NotificationLog>(nation2).unwrap();
    let log3 = fixture.world.world.get::<NotificationLog>(nation3).unwrap();
    
    assert_eq!(log1.notifications.len(), 1);
    assert_eq!(log2.notifications.len(), 1);
    assert_eq!(log3.notifications.len(), 1);
    assert_eq!(log1.notifications[0].severity, NotificationSeverity::Warning);
}

#[test]
fn test_alliance_broken_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation1 = fixture.world.spawn_nation("Nation1".to_string(), [255, 0, 0], false);
    let nation2 = fixture.world.spawn_nation("Nation2".to_string(), [0, 255, 0], false);
    
    let id1 = fixture.world.world.get::<Nation>(nation1).unwrap().id;
    let id2 = fixture.world.world.get::<Nation>(nation2).unwrap().id;
    
    // Create alliance broken notification
    create_alliance_broken_notification(&mut fixture.world.world, vec![id1, id2], 150);
    
    // Verify both received notification
    let log1 = fixture.world.world.get::<NotificationLog>(nation1).unwrap();
    let log2 = fixture.world.world.get::<NotificationLog>(nation2).unwrap();
    
    assert_eq!(log1.notifications.len(), 1);
    assert_eq!(log2.notifications.len(), 1);
}

// ============================================================================
// NUCLEAR NOTIFICATION TESTS
// ============================================================================

#[test]
fn test_nuclear_use_notification_global() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create multiple nations
    let attacker = fixture.world.spawn_nation("Attacker".to_string(), [255, 0, 0], false);
    let target = fixture.world.spawn_nation("Target".to_string(), [0, 255, 0], false);
    let bystander = fixture.world.spawn_nation("Bystander".to_string(), [0, 0, 255], false);
    
    let attacker_id = fixture.world.world.get::<Nation>(attacker).unwrap().id;
    let target_id = fixture.world.world.get::<Nation>(target).unwrap().id;
    
    // Create provinces
    let prov1 = ProvinceId::new();
    let prov2 = ProvinceId::new();
    
    // Create nuclear use notification
    create_nuclear_use_notification(
        &mut fixture.world.world,
        attacker_id,
        target_id,
        vec![prov1, prov2],
        100,
    );
    
    // Verify ALL nations received the notification (global event)
    let log_attacker = fixture.world.world.get::<NotificationLog>(attacker).unwrap();
    let log_target = fixture.world.world.get::<NotificationLog>(target).unwrap();
    let log_bystander = fixture.world.world.get::<NotificationLog>(bystander).unwrap();
    
    assert_eq!(log_attacker.notifications.len(), 1);
    assert_eq!(log_target.notifications.len(), 1);
    assert_eq!(log_bystander.notifications.len(), 1); // Even bystander gets notified
    assert_eq!(log_attacker.notifications[0].severity, NotificationSeverity::Critical);
}

#[test]
fn test_nuclear_treaty_violation_notification_global() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let violator = fixture.world.spawn_nation("Violator".to_string(), [255, 0, 0], false);
    let observer = fixture.world.spawn_nation("Observer".to_string(), [0, 255, 0], false);
    
    let violator_id = fixture.world.world.get::<Nation>(violator).unwrap().id;
    
    // Create nuclear treaty violation notification
    create_nuclear_treaty_violation_notification(&mut fixture.world.world, violator_id, 100);
    
    // Verify all nations received notification
    let log_violator = fixture.world.world.get::<NotificationLog>(violator).unwrap();
    let log_observer = fixture.world.world.get::<NotificationLog>(observer).unwrap();
    
    assert_eq!(log_violator.notifications.len(), 1);
    assert_eq!(log_observer.notifications.len(), 1);
}

#[test]
fn test_nuclear_capability_notification_global() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let achiever = fixture.world.spawn_nation("Achiever".to_string(), [255, 0, 0], false);
    let observer = fixture.world.spawn_nation("Observer".to_string(), [0, 255, 0], false);
    
    let achiever_id = fixture.world.world.get::<Nation>(achiever).unwrap().id;
    
    // Create nuclear capability notification
    create_nuclear_capability_notification(&mut fixture.world.world, achiever_id, 100);
    
    // Verify all nations received notification
    let log_achiever = fixture.world.world.get::<NotificationLog>(achiever).unwrap();
    let log_observer = fixture.world.world.get::<NotificationLog>(observer).unwrap();
    
    assert_eq!(log_achiever.notifications.len(), 1);
    assert_eq!(log_observer.notifications.len(), 1);
    assert_eq!(log_achiever.notifications[0].severity, NotificationSeverity::Major);
}

// ============================================================================
// ECONOMIC NOTIFICATION TESTS
// ============================================================================

#[test]
fn test_economic_crisis_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Create economic crisis notification
    create_economic_crisis_notification(&mut fixture.world.world, nation_id, 25.5, 100);
    
    // Verify notification
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    
    assert_eq!(log.notifications.len(), 1);
    assert_eq!(log.notifications[0].severity, NotificationSeverity::Warning);
    assert!(log.notifications[0].message.contains("25.5"));
}

#[test]
fn test_legitimacy_crisis_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Create legitimacy crisis notification
    create_legitimacy_crisis_notification(&mut fixture.world.world, nation_id, 15.0, 100);
    
    // Verify notification
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    
    assert_eq!(log.notifications.len(), 1);
    assert_eq!(log.notifications[0].severity, NotificationSeverity::Critical);
}

// ============================================================================
// ESPIONAGE NOTIFICATION TESTS
// ============================================================================

#[test]
fn test_espionage_revealed_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let perpetrator = fixture.world.spawn_nation("Spy".to_string(), [255, 0, 0], false);
    let target = fixture.world.spawn_nation("Target".to_string(), [0, 255, 0], false);
    
    let perpetrator_id = fixture.world.world.get::<Nation>(perpetrator).unwrap().id;
    let target_id = fixture.world.world.get::<Nation>(target).unwrap().id;
    
    // Create espionage revealed notification
    create_espionage_revealed_notification(
        &mut fixture.world.world,
        perpetrator_id,
        target_id,
        "Steal Technology".to_string(),
        100,
    );
    
    // Verify both nations received notification
    let log_perp = fixture.world.world.get::<NotificationLog>(perpetrator).unwrap();
    let log_target = fixture.world.world.get::<NotificationLog>(target).unwrap();
    
    assert_eq!(log_perp.notifications.len(), 1);
    assert_eq!(log_target.notifications.len(), 1);
    assert_eq!(log_perp.notifications[0].severity, NotificationSeverity::Warning);
}

// ============================================================================
// VASSALAGE NOTIFICATION TESTS
// ============================================================================

#[test]
fn test_vassalization_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let overlord = fixture.world.spawn_nation("Overlord".to_string(), [255, 0, 0], false);
    let vassal = fixture.world.spawn_nation("Vassal".to_string(), [0, 255, 0], false);
    
    let overlord_id = fixture.world.world.get::<Nation>(overlord).unwrap().id;
    let vassal_id = fixture.world.world.get::<Nation>(vassal).unwrap().id;
    
    // Create vassalization notification
    create_vassalization_notification(&mut fixture.world.world, overlord_id, vassal_id, 100);
    
    // Verify both nations received notification
    let log_overlord = fixture.world.world.get::<NotificationLog>(overlord).unwrap();
    let log_vassal = fixture.world.world.get::<NotificationLog>(vassal).unwrap();
    
    assert_eq!(log_overlord.notifications.len(), 1);
    assert_eq!(log_vassal.notifications.len(), 1);
    assert_eq!(log_overlord.notifications[0].severity, NotificationSeverity::Major);
}

#[test]
fn test_vassal_rebellion_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let vassal = fixture.world.spawn_nation("Vassal".to_string(), [255, 0, 0], false);
    let overlord = fixture.world.spawn_nation("Overlord".to_string(), [0, 255, 0], false);
    
    let vassal_id = fixture.world.world.get::<Nation>(vassal).unwrap().id;
    let overlord_id = fixture.world.world.get::<Nation>(overlord).unwrap().id;
    
    // Create vassal rebellion notification
    create_vassal_rebellion_notification(&mut fixture.world.world, vassal_id, overlord_id, 100);
    
    // Verify both nations received notification
    let log_vassal = fixture.world.world.get::<NotificationLog>(vassal).unwrap();
    let log_overlord = fixture.world.world.get::<NotificationLog>(overlord).unwrap();
    
    assert_eq!(log_vassal.notifications.len(), 1);
    assert_eq!(log_overlord.notifications.len(), 1);
    assert_eq!(log_vassal.notifications[0].severity, NotificationSeverity::Major);
}

// ============================================================================
// TERRITORY NOTIFICATION TESTS
// ============================================================================

#[test]
fn test_territory_lost_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Create territory lost notification
    create_territory_lost_notification(&mut fixture.world.world, nation_id, 5, 100);
    
    // Verify notification
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    
    assert_eq!(log.notifications.len(), 1);
    assert_eq!(log.notifications[0].severity, NotificationSeverity::Warning);
    assert!(log.notifications[0].message.contains("5"));
}

#[test]
fn test_territory_gained_notification() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Create territory gained notification
    create_territory_gained_notification(&mut fixture.world.world, nation_id, 3, 100);
    
    // Verify notification
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    
    assert_eq!(log.notifications.len(), 1);
    assert_eq!(log.notifications[0].severity, NotificationSeverity::Info);
    assert!(log.notifications[0].message.contains("3"));
}

// ============================================================================
// MULTI-NATION SCENARIO TESTS
// ============================================================================

#[test]
fn test_multiple_notifications_per_nation() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Add multiple different notifications
    create_economic_crisis_notification(&mut fixture.world.world, nation_id, 10.0, 100);
    create_legitimacy_crisis_notification(&mut fixture.world.world, nation_id, 20.0, 101);
    create_territory_lost_notification(&mut fixture.world.world, nation_id, 2, 102);
    
    // Verify all notifications received
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    
    assert_eq!(log.notifications.len(), 3);
    assert_eq!(log.notifications[0].tick, 100);
    assert_eq!(log.notifications[1].tick, 101);
    assert_eq!(log.notifications[2].tick, 102);
}

#[test]
fn test_notification_pruning_across_ticks() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Add many notifications
    for i in 0..150 {
        create_territory_gained_notification(&mut fixture.world.world, nation_id, 1, i);
    }
    
    // Create phase with low max limit
    let mut phase = NotificationPhase::new().with_max_notifications(100);
    
    // Execute phase to trigger pruning
    phase.execute(&mut fixture.world.world);
    
    // Verify pruning occurred
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    assert_eq!(log.notifications.len(), 100); // Should be pruned to 100
}

#[test]
fn test_notification_phase_execution_with_nations() {
    let mut fixture = TestWorldBuilder::new().build();
    
    // Create nations
    let nation1 = fixture.world.spawn_nation("Nation1".to_string(), [255, 0, 0], false);
    let nation2 = fixture.world.spawn_nation("Nation2".to_string(), [0, 255, 0], false);
    
    let id1 = fixture.world.world.get::<Nation>(nation1).unwrap().id;
    let id2 = fixture.world.world.get::<Nation>(nation2).unwrap().id;
    
    // Add notifications
    create_war_notification(&mut fixture.world.world, id1, id2, 100);
    
    // Execute notification phase
    let mut phase = NotificationPhase::new();
    phase.execute(&mut fixture.world.world);
    
    // Verify execution succeeded and notifications persisted
    let log1 = fixture.world.world.get::<NotificationLog>(nation1).unwrap();
    let log2 = fixture.world.world.get::<NotificationLog>(nation2).unwrap();
    
    assert_eq!(log1.notifications.len(), 1);
    assert_eq!(log2.notifications.len(), 1);
}

#[test]
fn test_notification_filtering_by_severity_in_multi_nation_scenario() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Add notifications of different severities
    create_territory_gained_notification(&mut fixture.world.world, nation_id, 1, 100); // Info
    create_economic_crisis_notification(&mut fixture.world.world, nation_id, 10.0, 101); // Warning
    create_legitimacy_crisis_notification(&mut fixture.world.world, nation_id, 15.0, 102); // Critical
    
    // Verify filtering by severity
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    
    assert_eq!(log.by_severity(NotificationSeverity::Info).len(), 1);
    assert_eq!(log.by_severity(NotificationSeverity::Warning).len(), 1);
    assert_eq!(log.by_severity(NotificationSeverity::Critical).len(), 1);
    assert_eq!(log.by_severity(NotificationSeverity::Major).len(), 0);
}

#[test]
fn test_notification_time_filtering() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation("Nation".to_string(), [255, 0, 0], false);
    let nation_id = fixture.world.world.get::<Nation>(nation).unwrap().id;
    
    // Add notifications at different times
    create_territory_gained_notification(&mut fixture.world.world, nation_id, 1, 50);
    create_territory_gained_notification(&mut fixture.world.world, nation_id, 1, 90);
    create_territory_gained_notification(&mut fixture.world.world, nation_id, 1, 100);
    
    // Filter notifications from last 20 ticks (current_tick=100)
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    let recent = log.recent(100, 20);
    
    assert_eq!(recent.len(), 2); // Only ticks 90 and 100
}
