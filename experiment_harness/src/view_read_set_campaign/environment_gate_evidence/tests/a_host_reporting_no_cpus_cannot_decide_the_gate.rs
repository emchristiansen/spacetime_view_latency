//! The load clause is a comparison against the CPU count, so a zero count has no gate in it.

use super::fixture::{passing_first, passing_second, LOGICAL_CPUS};
use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;

/// Coverage: zero CPUs would make the ceiling 0.00 and refuse every host that is running at all,
/// which is an unreadable count rather than a busy machine. Refused at the constructor, so `passes`
/// never derives a verdict from one.
#[test]
fn a_host_reporting_no_cpus_cannot_decide_the_gate() {
    assert!(
        EnvironmentGateEvidence::paired(passing_first(), passing_second(), 0).is_err(),
        "a host reporting no CPUs cannot supply the load clause's ceiling"
    );

    for logical_cpus in [1, LOGICAL_CPUS] {
        match EnvironmentGateEvidence::paired(passing_first(), passing_second(), logical_cpus) {
            Ok(_) => {}
            Err(e) => panic!("{logical_cpus} CPUs is a readable count: {e:#}"),
        }
    }
}
