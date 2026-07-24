//! `from_nanos` retains a `u128` value above `u64::MAX` exactly — no narrowing to 64 bits.

use crate::observation::latency_sample::LatencySample;

#[test]
fn a_value_above_u64_max_is_retained_exactly() {
    // One nanosecond past the widest `u64` — a value that would be lost by any 64-bit narrowing.
    let nanos = u128::from(u64::MAX) + 1;
    let sample = LatencySample::from_nanos(nanos);
    assert_eq!(
        sample.nanos(),
        nanos,
        "a sample reconstructed from raw nanoseconds retains the full u128 value"
    );
}
