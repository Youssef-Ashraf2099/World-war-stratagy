//! Notifications and News System
//!
//! This subsystem manages in-game notifications and news events,
//! providing players with updates about important game events.
//!
//! Features:
//! - Event notification generation
//! - Notification filtering and querying
//! - Automatic notification cleanup
//! - Severity-based categorization

use crate::core::types::*;
use crate::core::tick::TickPhase;
use bevy_ecs::prelude::*;

/// Notification phase - processes and manages notifications
#[derive(Default)]
pub struct NotificationPhase {
    /// Maximum notifications to keep per nation
    pub max_notifications: usize,
}

impl NotificationPhase {
    pub fn new() -> Self {
        Self {
            max_notifications: 1000, // Keep last 1000 notifications
        }
    }

    /// Set maximum notifications to keep
    pub fn with_max_notifications(mut self, max: usize) -> Self {
        self.max_notifications = max;
        self
    }
}

impl TickPhase for NotificationPhase {
    fn name(&self) -> &str {
        "Notifications"
    }

    fn execute(&mut self, world: &mut World) {
        // Prune old notifications to prevent unbounded growth
        prune_old_notifications(world, self.max_notifications);
    }
}

// ============================================================================
// NOTIFICATION CREATION HELPERS
// ============================================================================

/// Create a war declaration notification
pub fn create_war_notification(
    world: &mut World,
    attacker_id: NationId,
    defender_id: NationId,
    tick: Tick,
) {
    let title = "War Declared".to_string();
    let message = format!("War has been declared between nations");
    let notification = Notification::new(
        NotificationType::WarDeclared { attacker: attacker_id, defender: defender_id },
        title,
        message,
        tick,
        NotificationSeverity::Major,
        vec![attacker_id, defender_id],
    );
    
    add_notification_to_nations(world, &notification, vec![attacker_id, defender_id]);
}

/// Create a war ended notification
pub fn create_war_ended_notification(
    world: &mut World,
    victor: Option<NationId>,
    defeated: NationId,
    tick: Tick,
) {
    let title = "War Ended".to_string();
    let message = if let Some(victor_id) = victor {
        format!("War has ended with a victor")
    } else {
        format!("War has ended")
    };
    
    let mut related = vec![defeated];
    if let Some(v) = victor {
        related.push(v);
    }
    
    let notification = Notification::new(
        NotificationType::WarEnded { victor, defeated },
        title,
        message,
        tick,
        NotificationSeverity::Major,
        related.clone(),
    );
    
    add_notification_to_nations(world, &notification, related);
}

/// Create an alliance formed notification
pub fn create_alliance_notification(
    world: &mut World,
    nations: Vec<NationId>,
    tick: Tick,
) {
    let title = "Alliance Formed".to_string();
    let message = format!("A new alliance has been formed between {} nations", nations.len());
    let notification = Notification::new(
        NotificationType::AllianceFormed { nations: nations.clone() },
        title,
        message,
        tick,
        NotificationSeverity::Warning,
        nations.clone(),
    );
    
    add_notification_to_nations(world, &notification, nations);
}

/// Create an alliance broken notification
pub fn create_alliance_broken_notification(
    world: &mut World,
    former_allies: Vec<NationId>,
    tick: Tick,
) {
    let title = "Alliance Broken".to_string();
    let message = format!("An alliance has been dissolved");
    let notification = Notification::new(
        NotificationType::AllianceBroken { former_allies: former_allies.clone() },
        title,
        message,
        tick,
        NotificationSeverity::Warning,
        former_allies.clone(),
    );
    
    add_notification_to_nations(world, &notification, former_allies);
}

/// Create a nuclear weapon used notification
pub fn create_nuclear_use_notification(
    world: &mut World,
    attacker_id: NationId,
    target_id: NationId,
    provinces: Vec<ProvinceId>,
    tick: Tick,
) {
    let title = "NUCLEAR WEAPON USED".to_string();
    let message = format!("Nuclear weapons have been deployed against {} provinces", provinces.len());
    let notification = Notification::new(
        NotificationType::NuclearWeaponUsed { 
            attacker: attacker_id, 
            target: target_id, 
            provinces: provinces.clone() 
        },
        title,
        message,
        tick,
        NotificationSeverity::Critical,
        vec![attacker_id, target_id],
    );
    
    // Nuclear events should notify ALL nations
    notify_all_nations(world, notification);
}

/// Create a nuclear treaty violation notification
pub fn create_nuclear_treaty_violation_notification(
    world: &mut World,
    violator_id: NationId,
    tick: Tick,
) {
    let title = "Nuclear Treaty Violation".to_string();
    let message = format!("A nation has violated nuclear treaties");
    let notification = Notification::new(
        NotificationType::NuclearTreatyViolation { violator: violator_id },
        title,
        message,
        tick,
        NotificationSeverity::Warning,
        vec![violator_id],
    );
    
    // Treaty violations should notify all nations
    notify_all_nations(world, notification);
}

/// Create a nuclear capability achieved notification
pub fn create_nuclear_capability_notification(
    world: &mut World,
    nation_id: NationId,
    tick: Tick,
) {
    let title = "Nuclear Capability Achieved".to_string();
    let message = format!("A nation has achieved nuclear weapons capability");
    let notification = Notification::new(
        NotificationType::NuclearCapabilityAchieved { nation: nation_id },
        title,
        message,
        tick,
        NotificationSeverity::Major,
        vec![nation_id],
    );
    
    // Nuclear capability should notify all nations
    notify_all_nations(world, notification);
}

/// Create an economic crisis notification
pub fn create_economic_crisis_notification(
    world: &mut World,
    nation_id: NationId,
    gdp_loss: f64,
    tick: Tick,
) {
    let title = "Economic Crisis".to_string();
    let message = format!("Economic crisis detected with GDP loss of {:.2}%", gdp_loss);
    let notification = Notification::new(
        NotificationType::EconomicCrisis { nation: nation_id, gdp_loss },
        title,
        message,
        tick,
        NotificationSeverity::Warning,
        vec![nation_id],
    );
    
    add_notification_to_nations(world, &notification, vec![nation_id]);
}

/// Create a legitimacy crisis notification
pub fn create_legitimacy_crisis_notification(
    world: &mut World,
    nation_id: NationId,
    legitimacy: f64,
    tick: Tick,
) {
    let title = "Legitimacy Crisis".to_string();
    let message = format!("Legitimacy has dropped to critical levels: {:.1}%", legitimacy);
    let notification = Notification::new(
        NotificationType::LegitimacyCrisis { nation: nation_id, legitimacy },
        title,
        message,
        tick,
        NotificationSeverity::Critical,
        vec![nation_id],
    );
    
    add_notification_to_nations(world, &notification, vec![nation_id]);
}

/// Create an espionage revealed notification
pub fn create_espionage_revealed_notification(
    world: &mut World,
    perpetrator_id: NationId,
    target_id: NationId,
    operation: String,
    tick: Tick,
) {
    let title = "Espionage Revealed".to_string();
    let message = format!("Espionage operation '{}' has been revealed", operation);
    let notification = Notification::new(
        NotificationType::EspionageRevealed { 
            perpetrator: perpetrator_id, 
            target: target_id, 
            operation 
        },
        title,
        message,
        tick,
        NotificationSeverity::Warning,
        vec![perpetrator_id, target_id],
    );
    
    add_notification_to_nations(world, &notification, vec![perpetrator_id, target_id]);
}

/// Create a vassalization notification
pub fn create_vassalization_notification(
    world: &mut World,
    overlord_id: NationId,
    vassal_id: NationId,
    tick: Tick,
) {
    let title = "Vassalization Occurred".to_string();
    let message = format!("A nation has become a vassal");
    let notification = Notification::new(
        NotificationType::VassalizationOccurred { 
            overlord: overlord_id, 
            vassal: vassal_id 
        },
        title,
        message,
        tick,
        NotificationSeverity::Major,
        vec![overlord_id, vassal_id],
    );
    
    add_notification_to_nations(world, &notification, vec![overlord_id, vassal_id]);
}

/// Create a vassal rebellion notification
pub fn create_vassal_rebellion_notification(
    world: &mut World,
    vassal_id: NationId,
    overlord_id: NationId,
    tick: Tick,
) {
    let title = "Vassal Rebellion".to_string();
    let message = format!("A vassal has rebelled against its overlord");
    let notification = Notification::new(
        NotificationType::VassalRebellion { 
            vassal: vassal_id, 
            overlord: overlord_id 
        },
        title,
        message,
        tick,
        NotificationSeverity::Major,
        vec![vassal_id, overlord_id],
    );
    
    add_notification_to_nations(world, &notification, vec![vassal_id, overlord_id]);
}

/// Create a territory lost notification
pub fn create_territory_lost_notification(
    world: &mut World,
    nation_id: NationId,
    provinces_lost: u32,
    tick: Tick,
) {
    let title = "Territory Lost".to_string();
    let message = format!("{} provinces have been lost", provinces_lost);
    let notification = Notification::new(
        NotificationType::TerritoryLost { nation: nation_id, provinces_lost },
        title,
        message,
        tick,
        NotificationSeverity::Warning,
        vec![nation_id],
    );
    
    add_notification_to_nations(world, &notification, vec![nation_id]);
}

/// Create a territory gained notification
pub fn create_territory_gained_notification(
    world: &mut World,
    nation_id: NationId,
    provinces_gained: u32,
    tick: Tick,
) {
    let title = "Territory Gained".to_string();
    let message = format!("{} provinces have been conquered", provinces_gained);
    let notification = Notification::new(
        NotificationType::TerritoryGained { nation: nation_id, provinces_gained },
        title,
        message,
        tick,
        NotificationSeverity::Info,
        vec![nation_id],
    );
    
    add_notification_to_nations(world, &notification, vec![nation_id]);
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Add a notification to specific nations
fn add_notification_to_nations(
    world: &mut World,
    notification: &Notification,
    nation_ids: Vec<NationId>,
) {
    let mut query = world.query::<(&Nation, &mut NotificationLog)>();
    for (nation, mut log) in query.iter_mut(world) {
        if nation_ids.contains(&nation.id) {
            log.add(notification.clone());
        }
    }
}

/// Notify all nations in the world
fn notify_all_nations(world: &mut World, notification: Notification) {
    let mut query = world.query::<&mut NotificationLog>();
    for mut log in query.iter_mut(world) {
        log.add(notification.clone());
    }
}

/// Prune old notifications from all nations
fn prune_old_notifications(world: &mut World, keep_last: usize) {
    let mut query = world.query::<&mut NotificationLog>();
    for mut log in query.iter_mut(world) {
        log.prune(keep_last);
    }
}

/// Get all notifications for a nation
pub fn get_nation_notifications(world: &mut World, nation_id: NationId) -> Option<Vec<Notification>> {
    let mut query = world.query::<(&Nation, &NotificationLog)>();
    for (nation, log) in query.iter(world) {
        if nation.id == nation_id {
            return Some(log.notifications.clone());
        }
    }
    None
}

/// Get unread notification count for a nation
pub fn get_unread_count(world: &mut World, nation_id: NationId) -> usize {
    let mut query = world.query::<(&Nation, &NotificationLog)>();
    for (nation, log) in query.iter(world) {
        if nation.id == nation_id {
            return log.unread().len();
        }
    }
    0
}

/// Mark all notifications as read for a nation
pub fn mark_all_read_for_nation(world: &mut World, nation_id: NationId) {
    let mut query = world.query::<(&Nation, &mut NotificationLog)>();
    for (nation, mut log) in query.iter_mut(world) {
        if nation.id == nation_id {
            log.mark_all_read();
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_phase_creation() {
        let phase = NotificationPhase::new();
        assert_eq!(phase.name(), "Notifications");
        assert_eq!(phase.max_notifications, 1000);
    }

    #[test]
    fn test_notification_phase_with_custom_max() {
        let phase = NotificationPhase::new().with_max_notifications(500);
        assert_eq!(phase.max_notifications, 500);
    }

    #[test]
    fn test_notification_creation() {
        let notification = Notification::new(
            NotificationType::GameEvent { message: "Test".to_string() },
            "Test Title".to_string(),
            "Test Message".to_string(),
            100,
            NotificationSeverity::Info,
            vec![],
        );
        
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.tick, 100);
        assert!(!notification.read);
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
        let notification = Notification::new(
            NotificationType::GameEvent { message: "Test".to_string() },
            "Test".to_string(),
            "Test".to_string(),
            100,
            NotificationSeverity::Info,
            vec![nation_id],
        );
        
        assert!(notification.involves_nation(nation_id));
        assert!(!notification.involves_nation(NationId::new()));
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
    fn test_notification_log_unread() {
        let mut log = NotificationLog::new();
        let mut notification = Notification::new(
            NotificationType::GameEvent { message: "Test".to_string() },
            "Test".to_string(),
            "Test".to_string(),
            100,
            NotificationSeverity::Info,
            vec![],
        );
        
        log.add(notification.clone());
        assert_eq!(log.unread().len(), 1);
        
        notification.mark_read();
        log.add(notification);
        assert_eq!(log.unread().len(), 1); // Still 1 unread
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
            NotificationType::GameEvent { message: "Critical".to_string() },
            "Critical".to_string(),
            "Critical".to_string(),
            101,
            NotificationSeverity::Critical,
            vec![],
        ));
        
        assert_eq!(log.by_severity(NotificationSeverity::Info).len(), 1);
        assert_eq!(log.by_severity(NotificationSeverity::Critical).len(), 1);
    }

    #[test]
    fn test_notification_log_recent() {
        let mut log = NotificationLog::new();
        
        log.add(Notification::new(
            NotificationType::GameEvent { message: "Old".to_string() },
            "Old".to_string(),
            "Old".to_string(),
            10,
            NotificationSeverity::Info,
            vec![],
        ));
        
        log.add(Notification::new(
            NotificationType::GameEvent { message: "Recent".to_string() },
            "Recent".to_string(),
            "Recent".to_string(),
            100,
            NotificationSeverity::Info,
            vec![],
        ));
        
        assert_eq!(log.recent(100, 50).len(), 1); // Only recent one within 50 ticks
    }

    #[test]
    fn test_notification_log_prune() {
        let mut log = NotificationLog::new();
        
        for i in 0..10 {
            log.add(Notification::new(
                NotificationType::GameEvent { message: format!("Test {}", i) },
                format!("Title {}", i),
                format!("Message {}", i),
                i as u64,
                NotificationSeverity::Info,
                vec![],
            ));
        }
        
        assert_eq!(log.notifications.len(), 10);
        log.prune(5);
        assert_eq!(log.notifications.len(), 5);
    }
}
