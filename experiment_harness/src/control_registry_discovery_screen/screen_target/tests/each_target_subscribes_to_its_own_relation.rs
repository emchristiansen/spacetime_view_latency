//! One query per target, naming that target's relation, timed and untimed alike.

use std::collections::BTreeSet;

use crate::control_registry_discovery_screen::screen_target::ScreenTarget;

/// The relation each target subscribes to, transcribed from the module's own accessor declarations
/// rather than read back from the `TABLE_CONTROL_*` constants the function under test uses.
///
/// Reading those constants here would assert the function equals itself and would pass just as
/// happily after a swap that pointed Arm A at the base table — which is exactly the defect that
/// matters, because Arm A over `control_registry` instead of `control_registry_all_view` would still
/// return K rows and still pass every cardinality check the screen makes.
const FROZEN_RELATIONS: [(ScreenTarget, &str); 4] = [
    (ScreenTarget::ArmA, "control_registry_all_view"),
    (ScreenTarget::ControlA, "control_registry"),
    (
        ScreenTarget::ArmB,
        "control_activity_latest_by_control_view",
    ),
    (ScreenTarget::ControlB, "control_activity"),
];

/// Coverage: the single source of truth the driver's timed and untimed paths both draw from.
///
/// The screen times exactly one target and then issues the other three as untimed validation
/// subscriptions. Both paths take their SQL from [`ScreenTarget::subscription_sql`], so what this
/// pins is that each target's query names that target's own relation, and that no two targets name
/// the same one — a collision would leave one of the four caches never subscribed while the
/// composition check happily read another twice.
///
/// The exact-equality assertion is deliberate rather than a substring check: `control_registry` is a
/// prefix of `control_registry_all_view`, so "contains the relation name" would be satisfied by Arm A
/// and its Control pointing at the same relation, which is the precise confusion this guards.
#[test]
fn each_target_subscribes_to_its_own_relation() {
    let mut queries = BTreeSet::new();

    for (target, relation) in FROZEN_RELATIONS {
        let sql = target.subscription_sql();
        assert_eq!(
            sql,
            format!("SELECT * FROM {relation}"),
            "{target:?} must subscribe to {relation}"
        );
        assert!(
            queries.insert(sql.clone()),
            "{target:?} repeats a query another target already issues, so one of the four caches \
             would never be subscribed: {sql}"
        );
    }

    assert_eq!(
        queries.len(),
        ScreenTarget::ALL.len(),
        "every target needs its own relation, or the four-way composition is not four-way"
    );
    for target in ScreenTarget::ALL {
        assert!(
            FROZEN_RELATIONS.iter().any(|(frozen, _)| *frozen == target),
            "{target:?} is missing from the frozen relation table, so its query is unchecked"
        );
    }
}
