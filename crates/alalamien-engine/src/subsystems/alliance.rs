//! Alliance management subsystem (V0.4)
//! Handles cohesion decay, alliance dissolution, and obligation enforcement

use bevy_ecs::prelude::*;

use crate::core::types::*;
use crate::subsystems::notifications::create_alliance_broken_notification;

/// Alliance management phase
pub struct AlliancePhase;

impl AlliancePhase {
    pub fn new() -> Self {
        Self
    }

    /// Execute alliance phase: decay cohesion, check dissolutions, enforce obligations
    pub fn execute(&mut self, world: &mut World) {
        // Step 1: Decay cohesion for all alliances
        Self::decay_cohesion(world);

        // Step 2: Check for and execute alliance dissolutions
        Self::check_dissolutions(world);

        // Step 3: Enforce alliance obligations (DefensiveAgreement auto-join wars)
        Self::enforce_obligations(world);
    }

    /// Decay cohesion for all alliances each tick
    fn decay_cohesion(world: &mut World) {
        let mut query = world.query::<&mut Alliance>();
        for mut alliance in query.iter_mut(world) {
            alliance.decay_cohesion();
        }
    }

    /// Check for dissolutions and remove alliances that are dissolved
    fn check_dissolutions(world: &mut World) {
        // Collect entities and data of dissolved alliances first
        let mut to_dissolve: Vec<(Entity, AllianceId, String, Vec<NationId>)> = Vec::new();

        {
            let mut query = world.query::<(Entity, &Alliance)>();
            for (entity, alliance) in query.iter(world) {
                if alliance.is_dissolved() {
                    to_dissolve.push((
                        entity,
                        alliance.alliance_id,
                        alliance.alliance_name.clone(),
                        alliance.members.clone(),
                    ));
                }
            }
        }

        // Despawn dissolved alliances and create notifications
        for (entity, alliance_id, alliance_name, members) in to_dissolve {
            // Create notification for alliance dissolution
            create_alliance_broken_notification(world, members, 0);
            
            // Despawn the alliance entity
            world.despawn(entity);
        }
    }

    /// Enforce alliance doctrines and obligations
    fn enforce_obligations(world: &mut World) {
        // Step 1: Collect DefensiveAgreement alliances and their members
        let defensive_alliances: Vec<(Vec<NationId>, AllianceId)> = {
            let mut query = world.query::<&Alliance>();
            query
                .iter(world)
                .filter_map(|alliance| {
                    if alliance.doctrine == AllianceDoctrine::DefensiveAgreement {
                        Some((alliance.members.clone(), alliance.alliance_id))
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Step 2: Auto-join wars for DefensiveAgreement members
        for (members, _alliance_id) in defensive_alliances {
            Self::auto_join_defense(world, members);
        }
    }

    /// Auto-join wars for defensive agreement members
    fn auto_join_defense(world: &mut World, members: Vec<NationId>) {
        // Find wars where alliance members are being attacked
        let attacks_on_members: Vec<(NationId, NationId)> = {
            let mut query = world.query::<&WarDeclaration>();
            query
                .iter(world)
                .filter_map(|war| {
                    if members.contains(&war.defender) {
                        Some((war.aggressor, war.defender))
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Auto-join: all members join defense if one is attacked
        for (aggressor, defender) in attacks_on_members {
            for &member in &members {
                if member != defender {
                    // Auto-add member to the defensive war
                    Self::add_nation_to_war_support(world, member, aggressor);
                }
            }
        }
    }

    /// Add a nation as supporter to another nation's war against an aggressor
    fn add_nation_to_war_support(world: &mut World, supporter: NationId, aggressor: NationId) {
        // Step 1: Add aggressor to supporter's war list (supporter joins the war)
        let mut query = world.query::<(&NationId, &mut WarState)>();
        for (nation_id, mut war_state) in query.iter_mut(world) {
            if *nation_id == supporter {
                if !war_state.at_war_with.contains(&aggressor) {
                    war_state.at_war_with.push(aggressor);
                }
            }
        }

        // Step 2: Add supporter to aggressor's war list (bidirectional war state)
        let mut query = world.query::<(&NationId, &mut WarState)>();
        for (nation_id, mut war_state) in query.iter_mut(world) {
            if *nation_id == aggressor {
                if !war_state.at_war_with.contains(&supporter) {
                    war_state.at_war_with.push(supporter);
                }
            }
        }
    }
}

impl crate::core::tick::TickPhase for AlliancePhase {
    fn name(&self) -> &str {
        "Alliance"
    }

    fn execute(&mut self, world: &mut World) {
        AlliancePhase::execute(self, world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alliance_phase_creation() {
        let phase = AlliancePhase::new();
        assert_eq!(std::mem::size_of_val(&phase), std::mem::size_of::<()>());
    }

    #[test]
    fn test_cohesion_decay_execution() {
        let mut world = World::new();

        // Create test alliance
        let alliance = Alliance {
            alliance_id: AllianceId::new(),
            alliance_name: "Test Alliance".to_string(),
            founding_nation: NationId::default(),
            members: vec![NationId::default()],
            cohesion: 100.0,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            founded_tick: 0,
            threat_reduction: 0.25,
            cohesion_decay_rate: 2.0,
        };

        world.spawn(alliance);

        let mut phase = AlliancePhase::new();
        phase.execute(&mut world);

        // Verify cohesion decayed
        let mut query = world.query::<&Alliance>();
        for alliance in query.iter(&world) {
            assert!(alliance.cohesion < 100.0);
            assert!((100.0 - alliance.cohesion - 2.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_alliance_dissolution_check() {
        let mut world = World::new();

        // Create alliance with very low cohesion
        let alliance = Alliance {
            alliance_id: AllianceId::new(),
            alliance_name: "Dying Alliance".to_string(),
            founding_nation: NationId::default(),
            members: vec![NationId::default()],
            cohesion: 10.0, // Below dissolution threshold of 15
            doctrine: AllianceDoctrine::DefensiveAgreement,
            founded_tick: 0,
            threat_reduction: 0.25,
            cohesion_decay_rate: 1.0,
        };

        world.spawn(alliance);

        // Check if alliance is dissolved
        let mut query = world.query::<&Alliance>();
        for alliance in query.iter(&world) {
            assert!(alliance.is_dissolved());
        }
    }

    #[test]
    fn test_alliance_doctrine_defensive_agreement() {
        let mut world = World::new();

        // Create two nations
        let nation1 = Nation {
            id: NationId::new(),
            name: "Nation A".to_string(),
            color: [255, 0, 0],
        };
        let nation1_id = nation1.id;

        let nation2 = Nation {
            id: NationId::new(),
            name: "Nation B".to_string(),
            color: [0, 255, 0],
        };
        let nation2_id = nation2.id;

        world.spawn(nation1);
        world.spawn(nation2);

        // Create defensive agreement alliance
        let alliance = Alliance {
            alliance_id: AllianceId::new(),
            alliance_name: "Defense Pact".to_string(),
            founding_nation: nation1_id,
            members: vec![nation1_id, nation2_id],
            cohesion: 95.0,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            founded_tick: 0,
            threat_reduction: 0.35,
            cohesion_decay_rate: 1.0,
        };

        world.spawn(alliance);

        // Verify alliance created successfully
        let mut query = world.query::<&Alliance>();
        for alliance in query.iter(&world) {
            assert_eq!(alliance.doctrine, AllianceDoctrine::DefensiveAgreement);
            assert_eq!(alliance.member_count(), 2);
            assert!(!alliance.is_dissolved());
        }
    }

    #[test]
    fn test_multiple_alliance_cohesion_independent() {
        let mut world = World::new();

        // Create two alliances with different decay rates
        let alliance1 = Alliance {
            alliance_id: AllianceId::new(),
            alliance_name: "Fast Decay".to_string(),
            founding_nation: NationId::default(),
            members: vec![NationId::default(), NationId::new()], // Need 2+ members to not dissolve
            cohesion: 100.0,
            doctrine: AllianceDoctrine::DefensiveAgreement,
            founded_tick: 0,
            threat_reduction: 0.25,
            cohesion_decay_rate: 3.0, // Fast
        };

        let alliance2 = Alliance {
            alliance_id: AllianceId::new(),
            alliance_name: "Slow Decay".to_string(),
            founding_nation: NationId::default(),
            members: vec![NationId::default(), NationId::new()], // Need 2+ members to not dissolve
            cohesion: 100.0,
            doctrine: AllianceDoctrine::EconomicBloc,
            founded_tick: 0,
            threat_reduction: 0.20,
            cohesion_decay_rate: 0.5, // Slow
        };

        world.spawn(alliance1);
        world.spawn(alliance2);

        let mut phase = AlliancePhase::new();
        phase.execute(&mut world);

        // Check both decayed by their own rates
        let mut query = world.query::<&Alliance>();
        let alliances: Vec<_> = query.iter(&world).collect();
        assert_eq!(alliances.len(), 2);

        for alliance in alliances {
            if alliance.cohesion < 97.5 {
                // Fast decay alliance
                assert!(alliance.cohesion_decay_rate > 2.5);
            } else {
                // Slow decay alliance
                assert!(alliance.cohesion_decay_rate < 1.0);
            }
        }
    }
}
