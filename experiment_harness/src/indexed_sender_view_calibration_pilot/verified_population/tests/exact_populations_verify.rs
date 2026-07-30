//! Exactly-correct caches verify, and the token reports what was proven.

use super::fixture;
use crate::indexed_sender_view_calibration_pilot::verified_population::VerifiedPopulation;

/// Coverage: the verifier accepts the populations the frozen recipe actually produces.
///
/// Worth its own test rather than being assumed by the rejection tests: a verifier that rejected
/// everything would pass every one of those and be useless. The retained counts are asserted too,
/// because `CalibrationSeries::recorded` reads `verified_appends` back as a cross-check — so it has
/// to be the count the composition was genuinely proven at, not a plausible constant.
#[test]
fn exact_populations_verify() {
    let verified = VerifiedPopulation::verify(
        fixture::expectation(),
        &fixture::own_rows(),
        &fixture::witness_rows(),
    )
    .expect("the exactly-correct populations must verify");

    assert_eq!(
        verified.arm_rows(),
        fixture::own_count(),
        "the arm holds the seeded slice plus every append"
    );
    assert_eq!(
        verified.witness_rows(),
        fixture::checked_end(fixture::own_count(), fixture::unrelated_count()),
        "the witness holds both populations"
    );
    assert_eq!(
        verified.verified_appends(),
        fixture::APPENDS,
        "the token must carry the append count the composition was proven against, because the \
         series constructor refuses a complete series verified against a different one"
    );
}
