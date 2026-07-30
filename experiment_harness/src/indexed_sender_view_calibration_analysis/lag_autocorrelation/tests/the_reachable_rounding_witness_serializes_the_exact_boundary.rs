//! An admissible series whose exact coefficient is `-1` serializes exactly `-1.0` end to end.

use crate::indexed_sender_view_calibration_analysis::complete_replicate::CompleteReplicate;
use crate::indexed_sender_view_calibration_analysis::lag_autocorrelation::LagAutocorrelation;
use crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture;
use crate::indexed_sender_view_calibration_analysis::parse_calibration_ndjson::parse_calibration_ndjson;
use crate::indexed_sender_view_calibration_pilot::calibration_params::MAX_PACED_SAMPLES_USIZE;

/// Coverage: the reachable one-ULP range violation, through the **production** computation.
///
/// The constructor's own unit test pins the boundary repair in isolation; that is not sufficient,
/// because it cannot show the production path ever reaches the boundary. This test feeds the real
/// witness through `LagAutocorrelation::of` and asserts on the emitted artifact.
///
/// **Why this series reaches it.** At `lag = 999` a thousand-sample series has `pairs = 1`, so the
/// three sums degenerate to `numerator = z₀·z₉₉₉`, `left = z₀²`, `right = z₉₉₉²` and therefore
/// `numerator² = left · right` *exactly* — Cauchy–Schwarz equality holds unconditionally at the top
/// of the lag domain rather than by coincidence. The exact coefficient is exactly `-1`, while the
/// `f64` path rounds the numerator and both energies independently and returned
/// `-1.0000000000000002` before the range type existed.
///
/// The magnitudes are ordinary paced-append nanoseconds — about one to four milliseconds — so this
/// is not an `i128`-edge-case construction: admission constrains sample count, positivity, and
/// population, never magnitude.
///
/// The exact components are asserted alongside the coefficient. The boundary repair must be visible
/// only in the derived float; if it ever altered the retained integers, the authority this report
/// rests on would have moved.
#[test]
fn the_reachable_rounding_witness_serializes_the_exact_boundary() {
    let replicate = witness_replicate();
    let computed = LagAutocorrelation::of(&replicate, MAX_PACED_SAMPLES_USIZE - 1);
    let rendered = serde_json::to_value(computed).expect("the diagnostic serializes");

    assert_eq!(
        rendered["pairs"], 1,
        "the top of the lag domain leaves exactly one pair, which is what forces Cauchy-Schwarz \
         equality and makes the overshoot reachable"
    );
    assert_eq!(
        rendered["normalized"]["Defined"]["coefficient"],
        serde_json::json!(-1.0),
        "the exact ratio is exactly -1, so the artifact must carry -1.0 rather than the \
         -1.0000000000000002 the unguarded float division produces"
    );

    // Compared as whole objects: the right energy exceeds `i64`, so an accessor-based comparison
    // would have to switch integer types between the three components and could silently read a
    // `None` as a match. Comparing objects also pins the denominators, which matters here.
    //
    // **Each denominator is `n² = 1_000_000`, and that is the point.** The implementation centres on
    // an integer scale — `scaledᵢ = n·xᵢ − S` — so every product carries a common `n²` factor, and
    // the published component is that scaled sum with the factor divided out *exactly* as a
    // rational rather than rounded away. These fractions are already in lowest terms because each
    // numerator is odd and not divisible by five, so the surviving `1_000_000` is direct evidence
    // the scaling was undone exactly. A denominator of `1` here would mean the factor had been
    // dropped, and a coefficient computed from the scaled sums would still look correct because the
    // factor cancels in the ratio — so only these components can catch it.
    assert_eq!(
        rendered["numerator"],
        serde_json::json!({ "numerator": -9_588_033_344_648_951i64, "denominator": 1_000_000 }),
        "the retained exact numerator is the centred sum with the n² factor divided out exactly, \
         unchanged by the boundary repair"
    );
    assert_eq!(
        rendered["left_energy"],
        serde_json::json!({ "numerator": 9_585_259_344_049i64, "denominator": 1_000_000 }),
        "the retained exact left energy is unchanged by the boundary repair"
    );
    assert_eq!(
        rendered["right_energy"],
        serde_json::json!({ "numerator": 9_590_808_148_052_358_049u64, "denominator": 1_000_000 }),
        "the retained exact right energy is unchanged by the boundary repair"
    );
}

/// The witness: ordinary millisecond-scale nanosecond samples whose ends make the lag-999 ratio
/// exactly `-1`.
fn witness_replicate() -> CompleteReplicate {
    let mut samples = vec![1_000_000u128; MAX_PACED_SAMPLES_USIZE];
    samples[0] = 1_000_004;
    samples[MAX_PACED_SAMPLES_USIZE - 1] = 4_100_003;

    let line = LedgerFixture::complete(0).with_samples(samples).line();
    let records = parse_calibration_ndjson(&line).expect("the fixture line decodes");
    CompleteReplicate::admit(&records[0]).expect("the witness is an admissible replicate")
}
