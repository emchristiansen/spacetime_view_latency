//! Shared builders for the calibration series constructor's tests.
//!
//! All range arithmetic goes through [`checked_end`]: the fail-fast policy is not relaxed for test
//! code, and a fixture that wrapped silently would verify a population nobody intended.

use anyhow::{Error, Result};
use spacetimedb_sdk::Identity;

use crate::indexed_sender_view_calibration_pilot::calibration_expectation::{
    unrelated_owner, CalibrationExpectation,
};
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, OWN_ACTIVITY_ID_BASE, OWN_CONTROL_UUID, SUBSCRIBER_OWN_ROWS,
    UNRELATED_ACTIVITY_ID_BASE, UNRELATED_CONTROL_UUID,
};
use crate::indexed_sender_view_calibration_pilot::calibration_rung::CalibrationRung;
use crate::indexed_sender_view_calibration_pilot::calibration_series::CalibrationSeries;
use crate::indexed_sender_view_calibration_pilot::expected_population::expected_ts_micros;
use crate::indexed_sender_view_calibration_pilot::observed_row::ObservedRow;
use crate::indexed_sender_view_calibration_pilot::paced_sample_nanos::PacedSampleNanos;
use crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation;

/// A plausible latency, in nanoseconds. The value is irrelevant to every assertion here — only
/// positivity and count are checked — but a realistic one keeps the fixture honest about what it
/// stands for.
const SAMPLE_NANOS: u128 = 24_300_000;

/// The exclusive end of a half-open key range, failing loud rather than wrapping.
fn checked_end(base: u64, count: u64) -> u64 {
    base.checked_add(count)
        .expect("a fixture key range must not overflow u64")
}

/// The identity the client connected as.
fn measured() -> Identity {
    let mut bytes = [0u8; 32];
    bytes[31] = 0x01;
    Identity::from_byte_array(bytes)
}

/// One row carrying exactly what the frozen recipe assigns to `id`.
fn row(id: u64, control_uuid: u64, owner: Identity) -> ObservedRow {
    ObservedRow {
        id,
        ts_micros: expected_ts_micros(id),
        control_uuid,
        user_identity: owner,
    }
}

/// A population token genuinely verified against `appends` completed appends.
///
/// Built by running the real verifier over correct rows rather than by constructing the token
/// directly — which is impossible from here anyway, and that impossibility is the point of the token.
pub(super) fn population(appends: u64) -> VerifiedPopulation {
    let expectation = CalibrationExpectation::after(CalibrationRung::Baseline, appends, measured())
        .expect("the fixture never exceeds the frozen ceiling");

    let own_end = checked_end(OWN_ACTIVITY_ID_BASE, checked_end(SUBSCRIBER_OWN_ROWS, appends));
    let own: Vec<ObservedRow> = (OWN_ACTIVITY_ID_BASE..own_end)
        .map(|id| row(id, OWN_CONTROL_UUID, measured()))
        .collect();

    let unrelated_end = checked_end(
        UNRELATED_ACTIVITY_ID_BASE,
        CalibrationRung::Baseline.unrelated_rows(),
    );
    let mut witness = own.clone();
    witness.extend(
        (UNRELATED_ACTIVITY_ID_BASE..unrelated_end)
            .map(|id| row(id, UNRELATED_CONTROL_UUID, unrelated_owner())),
    );

    VerifiedPopulation::verify(expectation, &own, &witness).expect("the fixture rows are correct")
}

/// A population verified against a complete batch.
pub(super) fn complete_population() -> VerifiedPopulation {
    population(u64::from(MAX_PACED_SAMPLES))
}

/// `count` usable samples.
pub(super) fn samples(count: usize) -> Vec<PacedSampleNanos> {
    vec![PacedSampleNanos::of(SAMPLE_NANOS); count]
}

/// The error from a sealing attempt that must fail, as `Result::expect_err` would give it.
///
/// Written as a match because `expect_err` requires the `Ok` type to implement `Debug`, and
/// [`CalibrationSeries`] deliberately does not: a derived rendering would hand back through `Debug`
/// exactly the raw samples [`PacedSampleNanos`] withholds, and that omission is carried up through
/// every containing type on purpose. Deriving `Debug` to make a test convenient would weaken the
/// invariant these tests exist to protect, so the test bends instead of the type.
pub(super) fn refusal(result: Result<CalibrationSeries>, must_fail: &str) -> Error {
    match result {
        Ok(_) => panic!("{must_fail}"),
        Err(error) => error,
    }
}
