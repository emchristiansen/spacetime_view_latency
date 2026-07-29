//! Seeding reproduces Step 1's recipe exactly, and its ids and timestamps survive both rungs.

use std::collections::{BTreeMap, BTreeSet};

use spacetimedb_sdk::{Identity, Timestamp};

use crate::control_registry_discovery_screen::screen_params::{
    CONTROL_UUID_BASE, IDENTITY_BYTE_BASE, REGISTRY_CONTROLS, TS_BASE_MICROS, TS_STEP_MICROS,
};
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;

use super::super::{activity_id, activity_ts, control_identity, control_uuid};

/// The four seeding constants as `control_registry_step_one_reproducer.rs` declares them,
/// transcribed rather than imported.
///
/// Imported they would prove nothing: Step 1's are private to its own module, and re-exporting them
/// to compare would make both sides one value agreeing with itself. Transcribed, a drift in either
/// file fails here — which is the whole content of "copy the accepted Step 1 recipe exactly".
const STEP_ONE_CONTROL_UUID_BASE: u64 = 9_000;
const STEP_ONE_TS_BASE_MICROS: i64 = 1_700_000_000_000_000;
const STEP_ONE_TS_STEP_MICROS: i64 = 1_000;
const STEP_ONE_IDENTITY_BYTE_BASE: u8 = 0x40;

/// Coverage: the seeding recipe's constants, and the two properties the registry reducers refuse to
/// run without, at both frozen rungs.
///
/// **Why this is worth proving before a server exists.** The high rung seeds 32,000 rows through
/// 32,000 confirmed round trips. A duplicate activity id or a non-increasing timestamp is refused by
/// the reducers themselves, so a defect here does not produce a wrong number — it produces a
/// `Reducer` failure part-way through seeding, after tens of minutes, on all eight high-rung
/// attempts. The properties are pure functions of the recipe, so they are checkable in
/// milliseconds instead.
///
/// The id oracle is contiguity, not the formula: `occurrence * K + control_index` over the frozen
/// ranges must cover exactly `0..N` with nothing repeated and nothing missing. Re-deriving the
/// formula and comparing it to itself would pass for any bijection, including one that collided
/// across controls.
#[test]
fn the_seeding_recipe_stays_the_accepted_step_one_one() {
    assert_eq!(
        CONTROL_UUID_BASE, STEP_ONE_CONTROL_UUID_BASE,
        "the screen seeds the key space Step 1 proved"
    );
    assert_eq!(
        TS_BASE_MICROS, STEP_ONE_TS_BASE_MICROS,
        "the screen seeds from the timestamp base Step 1 proved"
    );
    assert_eq!(
        TS_STEP_MICROS, STEP_ONE_TS_STEP_MICROS,
        "the screen advances timestamps by the step Step 1 proved"
    );
    assert_eq!(
        IDENTITY_BYTE_BASE, STEP_ONE_IDENTITY_BYTE_BASE,
        "the screen derives identities from the byte base Step 1 proved"
    );

    for rung in ScreenRung::ALL {
        let history_rows = rung.history_rows();
        let rows_per_control = rung.rows_per_control();
        assert_eq!(
            rows_per_control * REGISTRY_CONTROLS,
            history_rows,
            "{rung:?} must seed exactly N rows, or the composition check is validating a history \
             the freeze does not describe"
        );

        let mut ids = BTreeSet::new();
        let mut timestamps = BTreeSet::new();
        let mut latest_per_control: BTreeMap<u64, Timestamp> = BTreeMap::new();
        let mut identity_per_control: BTreeMap<u64, Identity> = BTreeMap::new();

        for occurrence in 0..rows_per_control {
            for control_index in 0..REGISTRY_CONTROLS {
                let uuid = control_uuid(control_index);
                let id = activity_id(control_index, occurrence);
                let ts = activity_ts(id);

                assert!(
                    ids.insert(id),
                    "{rung:?}: activity id {id} is reused, and the reducer refuses a duplicate id"
                );
                assert!(
                    timestamps.insert(ts),
                    "{rung:?}: timestamp {ts:?} is reused, so the latest-per-control comparator \
                     would be ambiguous"
                );

                // Strictly increasing *per control*, which is the precondition the repeat reducer
                // actually enforces — a globally increasing sequence that went backwards within one
                // control would still be refused.
                if let Some(previous) = latest_per_control.insert(uuid, ts) {
                    assert!(
                        previous < ts,
                        "{rung:?}: control {uuid} went from {previous:?} to {ts:?}, and the repeat \
                         reducer refuses a ts that is not strictly later than the recorded last_ts"
                    );
                }

                // One identity per control for the whole history: the first-activity call
                // establishes it and every repeat preserves it, so a recipe that varied it by
                // occurrence would contradict what the registry stores.
                let identity = control_identity(control_index);
                if let Some(previous) = identity_per_control.insert(uuid, identity) {
                    assert_eq!(
                        previous, identity,
                        "{rung:?}: control {uuid} must own one identity across its whole history"
                    );
                }
            }
        }

        // Contiguous over `0..N`: nothing repeated, nothing missing, no gap a control could fall
        // into.
        assert_eq!(
            ids,
            (0..history_rows).collect::<BTreeSet<u64>>(),
            "{rung:?}: the seeded ids must cover exactly 0..{history_rows}"
        );
        assert_eq!(
            timestamps.len() as u64,
            history_rows,
            "{rung:?}: every seeded row needs a globally unique timestamp"
        );
        assert_eq!(
            identity_per_control.len() as u64,
            REGISTRY_CONTROLS,
            "{rung:?}: the history must span exactly K controls"
        );
        assert_eq!(
            identity_per_control
                .values()
                .collect::<BTreeSet<&Identity>>()
                .len() as u64,
            REGISTRY_CONTROLS,
            "{rung:?}: each control needs its own identity, or an identity-dropping bug would pass"
        );
    }
}
