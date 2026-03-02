/// System validation tests (pre-UI hardening)
///
/// These tests act as broad safety nets over long-running and high-entity
/// simulations to reduce risk before UI integration.

use crate::core::tick::TickPipeline;
use crate::core::types::{Nation, NotificationLog, Population, Resources};
use crate::tests::fixtures::TestWorldBuilder;

#[test]
fn test_pre_ui_world_health_after_long_v0_7_run() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(9200)
        .with_nations(20)
        .build();

    let mut pipeline = TickPipeline::new_v0_7();
    pipeline.execute_many(&mut fixture.world, 1000);

    assert_eq!(fixture.world.current_tick(), 1000);

    let nation_count = fixture.world.world.query::<&Nation>().iter(&fixture.world.world).count();
    assert!(nation_count >= 20, "Nation count should remain stable or grow over long runs");

    let mut pop_query = fixture.world.world.query::<&Population>();
    for pop in pop_query.iter(&fixture.world.world) {
        assert!(pop.total > 0, "Population should never drop to zero/negative");
        assert!(pop.total < 1_000_000_000_000, "Population overflow guard");
    }

    let mut res_query = fixture.world.world.query::<&Resources>();
    for res in res_query.iter(&fixture.world.world) {
        assert!(!res.food.is_nan(), "Food should not be NaN");
        assert!(!res.iron.is_nan(), "Iron should not be NaN");
        assert!(!res.oil.is_nan(), "Oil should not be NaN");
    }
}

#[test]
fn test_notification_system_remains_bounded_in_chaotic_run() {
    let mut fixture = TestWorldBuilder::new()
        .with_seed(9201)
        .with_nations(30)
        .build();

    let mut pipeline = TickPipeline::new_v0_7();
    pipeline.execute_many(&mut fixture.world, 1200);

    let mut logs = fixture.world.world.query::<&NotificationLog>();
    for log in logs.iter(&fixture.world.world) {
        assert!(
            log.notifications.len() <= 1000,
            "Notification pruning cap must hold under chaotic runs"
        );
    }
}

#[test]
fn test_pre_ui_deterministic_ticks_same_seed_same_result_shape() {
    let mut fixture_a = TestWorldBuilder::new()
        .with_seed(9202)
        .with_nations(10)
        .build();
    let mut fixture_b = TestWorldBuilder::new()
        .with_seed(9202)
        .with_nations(10)
        .build();

    let mut pipeline_a = TickPipeline::new_v0_7();
    let mut pipeline_b = TickPipeline::new_v0_7();
    pipeline_a.execute_many(&mut fixture_a.world, 300);
    pipeline_b.execute_many(&mut fixture_b.world, 300);

    assert_eq!(fixture_a.world.current_tick(), fixture_b.world.current_tick());

    let nations_a = fixture_a.world.world.query::<&Nation>().iter(&fixture_a.world.world).count();
    let nations_b = fixture_b.world.world.query::<&Nation>().iter(&fixture_b.world.world).count();
    assert_eq!(nations_a, nations_b, "Deterministic runs should preserve world shape");

    let logs_a = fixture_a.world.world.query::<&NotificationLog>().iter(&fixture_a.world.world).count();
    let logs_b = fixture_b.world.world.query::<&NotificationLog>().iter(&fixture_b.world.world).count();
    assert_eq!(logs_a, logs_b, "Deterministic runs should keep same notification log count");
}
