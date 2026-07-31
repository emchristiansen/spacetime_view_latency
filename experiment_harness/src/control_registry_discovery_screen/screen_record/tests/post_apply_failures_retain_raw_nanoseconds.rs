//! Every failure past a completed timed apply keeps its raw nanoseconds, and none can seal them.

use crate::control_registry_discovery_screen::attempt_failure::AttemptFailure;
use crate::control_registry_discovery_screen::failure_kind::FailureKind;
use crate::control_registry_discovery_screen::partial_evidence::PartialEvidence;
use crate::control_registry_discovery_screen::rejected_apply_nanos::RejectedApplyNanos;

use super::fixture::{
    assert_frozen_tables_are_total, diagnostic, failure_of, matched_composition,
    FROZEN_REQUIRES_SAMPLE, RETAINED_NANOS,
};

/// Coverage: the raw-retention biconditional against the frozen method, and that retention reaches
/// the ledger.
///
/// The expectation comes from [`FROZEN_REQUIRES_SAMPLE`], transcribed from the spec — not from
/// `kind.requires_sample()`. Comparing serialization against the predicate would only show the two
/// agreeing with each other, which they would do just as happily if the predicate were wrong; the
/// two assertions below are therefore kept separate, one pinning the predicate to the frozen method
/// and one pinning what actually reaches the ledger.
///
/// The spec requires raw per-sample retention even when a sample is rejected, so the four kinds
/// reachable only past a completed apply must each carry their duration — including a rejected zero,
/// which is precisely the reading a nonpositive-statistic failure has to show rather than discard.
///
/// Retention is asserted through **serialization**, because that is the only route by which the
/// value legitimately leaves the type: `RejectedApplyNanos` has a private field in its own module,
/// no accessor, and neither `Debug` nor `PartialEq`, so there is no typed path from a rejected
/// sample to `ColdApplyEvidence::sealed`. Seeing the number in the serialized record proves the
/// ledger keeps it without proving any in-process way to read it back.
#[test]
fn post_apply_failures_retain_raw_nanoseconds() {
    assert_frozen_tables_are_total();

    for (kind, expected) in FROZEN_REQUIRES_SAMPLE {
        assert_eq!(
            kind.requires_sample(),
            expected,
            "{kind:?} must agree with the frozen method about whether a completed apply precedes it"
        );

        let failure = failure_of(kind);
        let rendered = serde_json::to_string(&failure).expect("a failure serializes");
        assert_eq!(
            rendered.contains(&RETAINED_NANOS.to_string()),
            expected,
            "{kind:?} {} retain its raw apply nanoseconds, got {rendered}",
            if expected { "must" } else { "must not" },
        );
    }

    // A rejected zero is retained rather than dropped: the nonpositive statistic invalidates its
    // cell, and the ledger has to show the reading that did so.
    let zero = AttemptFailure::observed(
        FailureKind::Sample,
        PartialEvidence::RejectedSampleAndComposition {
            apply_nanos: RejectedApplyNanos::of(0),
            composition: matched_composition(),
        },
        diagnostic(),
    )
    .expect("a nonpositive sample is a recordable failure");
    let rendered = serde_json::to_string(&zero).expect("a failure serializes");
    assert!(
        rendered.contains("\"apply_nanos\":0"),
        "a rejected zero must be retained verbatim, got {rendered}"
    );

    // The biconditional's other half: a kind that strictly precedes the apply cannot invent one.
    let invented = AttemptFailure::observed(
        FailureKind::Connect,
        PartialEvidence::RejectedSample {
            apply_nanos: RejectedApplyNanos::of(RETAINED_NANOS),
        },
        diagnostic(),
    );
    assert!(
        invented.is_err(),
        "a Connect failure precedes the timed apply and cannot carry a sample"
    );

    // And a kind that only exists past the apply cannot claim it observed nothing.
    let dropped = AttemptFailure::observed(
        FailureKind::HostObservationAfter,
        PartialEvidence::NothingObserved,
        diagnostic(),
    );
    assert!(
        dropped.is_err(),
        "a HostObservationAfter failure has a completed apply behind it and must retain its sample"
    );
}
