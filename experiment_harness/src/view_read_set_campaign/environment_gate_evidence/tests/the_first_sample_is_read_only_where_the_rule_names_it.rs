//! The first sample enters the gate exactly twice, and only where the frozen rule names it.

use super::fixture::{passing_second, FIRST_OFFSET_NANOS, LOGICAL_CPUS, SWAP_OUT_BYTES};
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;
use crate::view_read_set_campaign::environment_sample::EnvironmentSample;

/// A first-sample load above the ceiling the second sample is held to.
const OVER_CEILING_LOAD_CENTI: u64 = LOGICAL_CPUS * 100 + 1;

/// Coverage: every threshold clause — the load ceiling, the RAM floor, the pressure ceiling — is
/// governed by "the second sample". The first supplies only the load the second must fall below and
/// the base of the swap-out delta. Applying the thresholds to it as well would be a stricter gate
/// than the one preregistered, so a first sample violating all three must still clear.
#[test]
fn the_first_sample_is_read_only_where_the_rule_names_it() {
    let dire_first = EnvironmentSample::observed(
        FIRST_OFFSET_NANOS,
        OVER_CEILING_LOAD_CENTI,
        0,
        SWAP_OUT_BYTES,
        u64::MAX,
    );
    let evidence = EnvironmentGateEvidence::paired(dire_first, passing_second(), LOGICAL_CPUS)
        .expect("the pair is the frozen separation apart");

    assert!(
        evidence.passes(),
        "no RAM, unbounded memory pressure, and a load above the ceiling in the *first* sample fall \
         outside every clause the frozen rule states; only its load and swap counter are read"
    );
}
