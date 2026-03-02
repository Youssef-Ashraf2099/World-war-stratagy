//! Unit tests for Notifications System
//!
//! Tests cover:
//! - Notification creation and management
//! - NotificationLog operations
//! - Severity filtering
//! - Time-based filtering
//! - Pruning and cleanup
//! - Nation-specific notifications

use crate::core::types::*;
use crate::subsystems::notifications::*;
use crate::core::tick::TickPhase;
use crate::tests::fixtures::TestWorldBuilder;

// ============================================================================
// NOTIFICATION CREATION TESTS
// ============================================================================

#[test]
fn test_notification_basic_creation() {
    let notification = Notification::new(
        NotificationType::GameEvent { message: "Test".to_string() },
        "Test Title".to_string(),
        "Test Message".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    );
    
    assert_eq!(notification.title, "Test Title");
    assert_eq!(notification.message, "Test Message");
    assert_eq!(notification.tick, 100);
    assert_eq!(notification.severity, NotificationSeverity::Info);
    assert!(!notification.read);
}

#[test]
fn test_notification_id_uniqueness() {
    let notif1 = Notification::new(
        NotificationType::GameEvent { message: "Test".to_string() },
        "Test1".to_string(),
        "Test1".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    );
    
    let notif2 = Notification::new(
        NotificationType::GameEvent { message: "Test".to_string() },
        "Test2".to_string(),
        "Test2".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    );
    
    assert_ne!(notif1.id, notif2.id);
}

#[test]
fn test_notification_mark_read() {
    let mut notification = Notification::new(
        NotificationType::GameEvent { message: "Test".to_string() },
        "Test".to_string(),
        "Test".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    );
    
    assert!(!notification.read);
    notification.mark_read();
    assert!(notification.read);
}

#[test]
fn test_notification_involves_nation() {
    let nation_id = NationId::new();
    let other_id = NationId::new();
    
    let notification = Notification::new(
        NotificationType::GameEvent { message: "Test".to_string() },
        "Test".to_string(),
        "Test".to_string(),
        100,
        NotificationSeverity::Info,
        vec![nation_id],
    );
    
    assert!(notification.involves_nation(nation_id));
    assert!(!notification.involves_nation(other_id));
}

#[test]
fn test_notification_multiple_nations() {
    let nation1 = NationId::new();
    let nation2 = NationId::new();
    let nation3 = NationId::new();
    
    let notification = Notification::new(
        NotificationType::AllianceFormed { nations: vec![nation1, nation2] },
        "Alliance".to_string(),
        "Alliance formed".to_string(),
        100,
        NotificationSeverity::Warning,
        vec![nation1, nation2],
    );
    
    assert!(notification.involves_nation(nation1));
    assert!(notification.involves_nation(nation2));
    assert!(!notification.involves_nation(nation3));
}

// ============================================================================
// NOTIFICATION LOG TESTS
// ============================================================================

#[test]
fn test_notification_log_creation() {
    let log = NotificationLog::new();
    assert_eq!(log.notifications.len(), 0);
}

#[test]
fn test_notification_log_add() {
    let mut log = NotificationLog::new();
    
    let notification = Notification::new(
        NotificationType::GameEvent { message: "Test".to_string() },
        "Test".to_string(),
        "Test".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    );
    
    log.add(notification);
    assert_eq!(log.notifications.len(), 1);
}

#[test]
fn test_notification_log_add_multiple() {
    let mut log = NotificationLog::new();
    
    for i in 0..5 {
        log.add(Notification::new(
            NotificationType::GameEvent { message: format!("Test {}", i) },
            format!("Title {}", i),
            format!("Message {}", i),
            i as u64,
            NotificationSeverity::Info,
            vec![],
        ));
    }
    
    assert_eq!(log.notifications.len(), 5);
}

#[test]
fn test_notification_log_unread() {
    let mut log = NotificationLog::new();
    
    // Add unread notification
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Unread".to_string() },
        "Unread".to_string(),
        "Unread".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    ));
    
    // Add read notification
    let mut read_notif = Notification::new(
        NotificationType::GameEvent { message: "Read".to_string() },
        "Read".to_string(),
        "Read".to_string(),
        101,
        NotificationSeverity::Info,
        vec![],
    );
    read_notif.mark_read();
    log.add(read_notif);
    
    assert_eq!(log.unread().len(), 1);
    assert_eq!(log.notifications.len(), 2);
}

#[test]
fn test_notification_log_by_severity() {
    let mut log = NotificationLog::new();
    
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Info".to_string() },
        "Info".to_string(),
        "Info".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    ));
    
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Warning".to_string() },
        "Warning".to_string(),
        "Warning".to_string(),
        101,
        NotificationSeverity::Warning,
        vec![],
    ));
    
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Critical".to_string() },
        "Critical".to_string(),
        "Critical".to_string(),
        102,
        NotificationSeverity::Critical,
        vec![],
    ));
    
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Major".to_string() },
        "Major".to_string(),
        "Major".to_string(),
        103,
        NotificationSeverity::Major,
        vec![],
    ));
    
    assert_eq!(log.by_severity(NotificationSeverity::Info).len(), 1);
    assert_eq!(log.by_severity(NotificationSeverity::Warning).len(), 1);
    assert_eq!(log.by_severity(NotificationSeverity::Critical).len(), 1);
    assert_eq!(log.by_severity(NotificationSeverity::Major).len(), 1);
}

#[test]
fn test_notification_log_recent() {
    let mut log = NotificationLog::new();
    
    // Add old notification
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Old".to_string() },
        "Old".to_string(),
        "Old".to_string(),
        10,
        NotificationSeverity::Info,
        vec![],
    ));
    
    // Add recent notifications
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Recent1".to_string() },
        "Recent1".to_string(),
        "Recent1".to_string(),
        95,
        NotificationSeverity::Info,
        vec![],
    ));
    
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Recent2".to_string() },
        "Recent2".to_string(),
        "Recent2".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    ));
    
    // Get notifications from last 20 ticks (current_tick=100, look back 20)
    let recent = log.recent(100, 20);
    assert_eq!(recent.len(), 2); // Only the two recent ones
}

#[test]
fn test_notification_log_mark_all_read() {
    let mut log = NotificationLog::new();
    
    for i in 0..5 {
        log.add(Notification::new(
            NotificationType::GameEvent { message: format!("Test {}", i) },
            format!("Test {}", i),
            format!("Test {}", i),
            i as u64,
            NotificationSeverity::Info,
            vec![],
        ));
    }
    
    assert_eq!(log.unread().len(), 5);
    log.mark_all_read();
    assert_eq!(log.unread().len(), 0);
}

#[test]
fn test_notification_log_count_by_severity() {
    let mut log = NotificationLog::new();
    
    for _ in 0..3 {
        log.add(Notification::new(
            NotificationType::GameEvent { message: "Info".to_string() },
            "Info".to_string(),
            "Info".to_string(),
            100,
            NotificationSeverity::Info,
            vec![],
        ));
    }
    
    for _ in 0..2 {
        log.add(Notification::new(
            NotificationType::GameEvent { message: "Warning".to_string() },
            "Warning".to_string(),
            "Warning".to_string(),
            100,
            NotificationSeverity::Warning,
            vec![],
        ));
    }
    
    assert_eq!(log.count_by_severity(NotificationSeverity::Info), 3);
    assert_eq!(log.count_by_severity(NotificationSeverity::Warning), 2);
    assert_eq!(log.count_by_severity(NotificationSeverity::Critical), 0);
}

#[test]
fn test_notification_log_prune() {
    let mut log = NotificationLog::new();
    
    for i in 0..100 {
        log.add(Notification::new(
            NotificationType::GameEvent { message: format!("Test {}", i) },
            format!("Test {}", i),
            format!("Test {}", i),
            i as u64,
            NotificationSeverity::Info,
            vec![],
        ));
    }
    
    assert_eq!(log.notifications.len(), 100);
    log.prune(50);
    assert_eq!(log.notifications.len(), 50);
    
    // Verify we kept the most recent ones
    assert_eq!(log.notifications.first().unwrap().tick, 50);
    assert_eq!(log.notifications.last().unwrap().tick, 99);
}

#[test]
fn test_notification_log_prune_no_effect_if_under_limit() {
    let mut log = NotificationLog::new();
    
    for i in 0..10 {
        log.add(Notification::new(
            NotificationType::GameEvent { message: format!("Test {}", i) },
            format!("Test {}", i),
            format!("Test {}", i),
            i as u64,
            NotificationSeverity::Info,
            vec![],
        ));
    }
    
    assert_eq!(log.notifications.len(), 10);
    log.prune(50); // Keep last 50, but only have 10
    assert_eq!(log.notifications.len(), 10); // No change
}

// ============================================================================
// NOTIFICATION PHASE TESTS
// ============================================================================

#[test]
fn test_notification_phase_creation() {
    let phase = NotificationPhase::new();
    assert_eq!(phase.name(), "Notifications");
}

#[test]
fn test_notification_phase_custom_max() {
    let phase = NotificationPhase::new().with_max_notifications(500);
    assert_eq!(phase.max_notifications, 500);
}

#[test]
fn test_notification_phase_executes() {
    let mut fixture = TestWorldBuilder::new().build();
    let mut phase = NotificationPhase::new();
    
    // Should not panic
    phase.execute(&mut fixture.world.world);
}

// ============================================================================
// WORLD INTEGRATION TESTS
// ============================================================================

#[test]
fn test_nation_spawns_with_notification_log() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation(
        "Test Nation".to_string(),
        [255, 0, 0],
        false,
    );
    
    let has_log = fixture.world.world.get::<NotificationLog>(nation).is_some();
    assert!(has_log);
}

#[test]
fn test_notification_log_initially_empty() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation = fixture.world.spawn_nation(
        "Test Nation".to_string(),
        [255, 0, 0],
        false,
    );
    
    let log = fixture.world.world.get::<NotificationLog>(nation).unwrap();
    assert_eq!(log.notifications.len(), 0);
}

#[test]
fn test_add_notification_to_nation() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation_entity = fixture.world.spawn_nation(
        "Test Nation".to_string(),
        [255, 0, 0],
        false,
    );
    
    let nation_id = fixture.world.world.get::<Nation>(nation_entity).unwrap().id;
    
    // Add notification
    let notification = Notification::new(
        NotificationType::GameEvent { message: "Test".to_string() },
        "Test".to_string(),
        "Test".to_string(),
        100,
        NotificationSeverity::Info,
        vec![nation_id],
    );
    
    let mut log = fixture.world.world.get_mut::<NotificationLog>(nation_entity).unwrap();
    log.add(notification);
    drop(log);
    
    let log = fixture.world.world.get::<NotificationLog>(nation_entity).unwrap();
    assert_eq!(log.notifications.len(), 1);
}

#[test]
fn test_multiple_nations_independent_logs() {
    let mut fixture = TestWorldBuilder::new().build();
    
    let nation1 = fixture.world.spawn_nation(
        "Nation 1".to_string(),
        [255, 0, 0],
        false,
    );
    
    let nation2 = fixture.world.spawn_nation(
        "Nation 2".to_string(),
        [0, 255, 0],
        false,
    );
    
    // Add notification to nation1
    let mut log1 = fixture.world.world.get_mut::<NotificationLog>(nation1).unwrap();
    log1.add(Notification::new(
        NotificationType::GameEvent { message: "Test1".to_string() },
        "Test1".to_string(),
        "Test1".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    ));
    drop(log1);
    
    // Verify nation1 has notification but nation2 doesn't
    let log1 = fixture.world.world.get::<NotificationLog>(nation1).unwrap();
    let log2 = fixture.world.world.get::<NotificationLog>(nation2).unwrap();
    
    assert_eq!(log1.notifications.len(), 1);
    assert_eq!(log2.notifications.len(), 0);
}

// ============================================================================
// NOTIFICATION TYPE TESTS
// ============================================================================

#[test]
fn test_notification_type_war_declared() {
    let attacker = NationId::new();
    let defender = NationId::new();
    
    let notif = Notification::new(
        NotificationType::WarDeclared { attacker, defender },
        "War Declared".to_string(),
        "War declared".to_string(),
        100,
        NotificationSeverity::Major,
        vec![attacker, defender],
    );
    
    assert!(matches!(notif.notification_type, NotificationType::WarDeclared { .. }));
}

#[test]
fn test_notification_type_nuclear_used() {
    let attacker = NationId::new();
    let target = NationId::new();
    let provinces = vec![ProvinceId::new(), ProvinceId::new()];
    
    let notif = Notification::new(
        NotificationType::NuclearWeaponUsed { 
            attacker, 
            target, 
            provinces: provinces.clone() 
        },
        "Nuclear Weapon Used".to_string(),
        "Nuclear strike detected".to_string(),
        100,
        NotificationSeverity::Critical,
        vec![attacker, target],
    );
    
    assert!(matches!(notif.notification_type, NotificationType::NuclearWeaponUsed { .. }));
    assert_eq!(notif.severity, NotificationSeverity::Critical);
}

#[test]
fn test_notification_type_alliance_formed() {
    let nations = vec![NationId::new(), NationId::new(), NationId::new()];
    
    let notif = Notification::new(
        NotificationType::AllianceFormed { nations: nations.clone() },
        "Alliance Formed".to_string(),
        "New alliance formed".to_string(),
        100,
        NotificationSeverity::Warning,
        nations.clone(),
    );
    
    assert!(matches!(notif.notification_type, NotificationType::AllianceFormed { .. }));
}

#[test]
fn test_notification_type_economic_crisis() {
    let nation = NationId::new();
    
    let notif = Notification::new(
        NotificationType::EconomicCrisis { nation, gdp_loss: 25.5 },
        "Economic Crisis".to_string(),
        "GDP dropped significantly".to_string(),
        100,
        NotificationSeverity::Warning,
        vec![nation],
    );
    
    assert!(matches!(notif.notification_type, NotificationType::EconomicCrisis { .. }));
}

#[test]
fn test_notification_type_espionage_revealed() {
    let perpetrator = NationId::new();
    let target = NationId::new();
    
    let notif = Notification::new(
        NotificationType::EspionageRevealed { 
            perpetrator, 
            target, 
            operation: "Steal Intel".to_string() 
        },
        "Espionage Revealed".to_string(),
        "Spy operation exposed".to_string(),
        100,
        NotificationSeverity::Warning,
        vec![perpetrator, target],
    );
    
    assert!(matches!(notif.notification_type, NotificationType::EspionageRevealed { .. }));
}

#[test]
fn test_notification_type_vassalization() {
    let overlord = NationId::new();
    let vassal = NationId::new();
    
    let notif = Notification::new(
        NotificationType::VassalizationOccurred { overlord, vassal },
        "Vassalization".to_string(),
        "Nation vassalized".to_string(),
        100,
        NotificationSeverity::Major,
        vec![overlord, vassal],
    );
    
    assert!(matches!(notif.notification_type, NotificationType::VassalizationOccurred { .. }));
}

// ============================================================================
// SEVERITY TESTS
// ============================================================================

#[test]
fn test_severity_levels() {
    assert_eq!(NotificationSeverity::Info, NotificationSeverity::Info);
    assert_eq!(NotificationSeverity::Warning, NotificationSeverity::Warning);
    assert_eq!(NotificationSeverity::Critical, NotificationSeverity::Critical);
    assert_eq!(NotificationSeverity::Major, NotificationSeverity::Major);
    
    assert_ne!(NotificationSeverity::Info, NotificationSeverity::Warning);
    assert_ne!(NotificationSeverity::Warning, NotificationSeverity::Critical);
}

#[test]
fn test_severity_filtering() {
    let mut log = NotificationLog::new();
    
    // Add different severity notifications
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Info".to_string() },
        "Info".to_string(),
        "Info".to_string(),
        100,
        NotificationSeverity::Info,
        vec![],
    ));
    
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Critical".to_string() },
        "Critical".to_string(),
        "Critical".to_string(),
        101,
        NotificationSeverity::Critical,
        vec![],
    ));
    
    log.add(Notification::new(
        NotificationType::GameEvent { message: "Major".to_string() },
        "Major".to_string(),
        "Major".to_string(),
        102,
        NotificationSeverity::Major,
        vec![],
    ));
    
    // Filter by severity
    let critical = log.by_severity(NotificationSeverity::Critical);
    assert_eq!(critical.len(), 1);
    assert_eq!(critical[0].title, "Critical");
}
