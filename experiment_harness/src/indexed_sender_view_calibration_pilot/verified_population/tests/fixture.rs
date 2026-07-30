//! Shared row builders for the composition verifier's tests.
//!
//! Every builder produces the population the frozen recipe demands, so each test states its attack as
//! a single mutation of a known-good set. A test that built its own near-correct rows inline would be
//! proving the verifier against a second, unreviewed recipe.
//!
//! All range arithmetic goes through [`checked_end`]. The fail-fast policy is not relaxed for test
//! code: a fixture that wrapped silently would hand the verifier a population nobody intended and
//! report the result as a property of the verifier.

use spacetimedb_sdk::Identity;

use crate::indexed_sender_view_calibration_pilot::calibration_expectation::{
    unrelated_owner, CalibrationExpectation,
};
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    OWN_ACTIVITY_ID_BASE, OWN_CONTROL_UUID, SUBSCRIBER_OWN_ROWS, UNRELATED_ACTIVITY_ID_BASE,
    UNRELATED_CONTROL_UUID,
};
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;
use crate::indexed_sender_view_calibration_pilot::expected_population::expected_ts_micros;
use crate::indexed_sender_view_calibration_pilot::observed_row::ObservedRow;

/// Appends this fixture pretends the batch recorded.
///
/// Small and not round, so an off-by-one in a range bound or a count cannot happen to coincide with
/// a boundary the fixture also uses. Far below the frozen ceiling, which the expectation enforces.
pub(super) const APPENDS: u64 = 7;

/// The own population's control, re-exported so a test can build a well-formed own row without
/// importing the frozen constants itself.
pub(super) const OWN_CONTROL_UUID_FOR_TESTS: u64 = OWN_CONTROL_UUID;

/// The exclusive end of a half-open key range, failing loud rather than wrapping.
pub(super) fn checked_end(base: u64, count: u64) -> u64 {
    base.checked_add(count)
        .expect("a fixture key range must not overflow u64")
}

/// The identity the client connected as — distinct from the non-connecting unrelated owner.
pub(super) fn measured() -> Identity {
    let mut bytes = [0u8; 32];
    bytes[31] = 0x01;
    Identity::from_byte_array(bytes)
}

/// The expectation both caches are checked against.
pub(super) fn expectation() -> CalibrationExpectation {
    CalibrationExpectation::after(CalibrationRung::Baseline, APPENDS, measured())
        .expect("seven appends is far below the frozen ceiling")
}

/// How many rows the own population holds at this fixture's append count.
pub(super) fn own_count() -> u64 {
    checked_end(SUBSCRIBER_OWN_ROWS, APPENDS)
}

/// How many rows the unrelated population holds at the baseline rung.
pub(super) fn unrelated_count() -> u64 {
    CalibrationRung::Baseline.unrelated_rows()
}

/// The first id past the end of the own range — in neither preregistered population.
pub(super) fn past_own_range_id() -> u64 {
    checked_end(OWN_ACTIVITY_ID_BASE, own_count())
}

/// One row carrying exactly what the frozen recipe assigns to `id`.
pub(super) fn row(id: u64, control_uuid: u64, owner: Identity) -> ObservedRow {
    ObservedRow {
        id,
        ts_micros: expected_ts_micros(id),
        control_uuid,
        user_identity: owner,
    }
}

/// The arm cache as it should be: the seeded own slice plus every completed append.
pub(super) fn own_rows() -> Vec<ObservedRow> {
    (OWN_ACTIVITY_ID_BASE..past_own_range_id())
        .map(|id| row(id, OWN_CONTROL_UUID, measured()))
        .collect()
}

/// The unrelated population as it should be at the baseline rung.
pub(super) fn unrelated_rows() -> Vec<ObservedRow> {
    let end = checked_end(UNRELATED_ACTIVITY_ID_BASE, unrelated_count());
    (UNRELATED_ACTIVITY_ID_BASE..end)
        .map(|id| row(id, UNRELATED_CONTROL_UUID, unrelated_owner()))
        .collect()
}

/// The witness cache as it should be: both populations.
pub(super) fn witness_rows() -> Vec<ObservedRow> {
    let mut rows = own_rows();
    rows.extend(unrelated_rows());
    rows
}
