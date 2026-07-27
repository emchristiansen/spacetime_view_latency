//! A read-set leak is classified as the candidate's answer, and the channels that ran are kept.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::campaign_driver::{validate_final_composition, MeasuredAttempt};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::failure_kind::FailureKind;
use crate::view_read_set_campaign::failure_stage::FailureStage;

use super::composition_fixture as fixture;

/// Coverage: the classification, and that a refusal does not throw away measured evidence.
///
/// One foreign row in the Arm's sender-scoped view is the leak the whole campaign exists to detect,
/// so the kind must be `SemanticsOrSecurity` — an operational kind here would file the candidate's
/// own answer as a harness fault and make it eligible for a retry that could only produce the same
/// answer. The stage is `AfterFirstSample` because the composition check runs only once the
/// saturated batch has confirmed, which is necessarily past the first measured sample; that stage is
/// also what
/// [`MeasuredSampleBoundary`](crate::view_read_set_campaign::measured_sample_boundary::MeasuredSampleBoundary)
/// derives the retry rule from, so it is asserted rather than assumed.
///
/// The retained prefix is compared as its full structural value, snapshotted from the `Vec` the
/// stage is then handed. A refusal does not unmeasure the channels that already ran, and that prefix
/// is exactly what a partial record is built from — so dropping or altering any part of it would
/// quietly shrink the evidence a failed attempt still contributes.
#[test]
fn a_leaked_foreign_row_fails_as_semantics_or_security() {
    let role = RunRole::Arm;
    let directory = fixture::attempt_directory("arm-leak", role);

    let mut seeded = fixture::seeded_rows(role);
    seeded.push(fixture::one_foreign_row());
    let mut final_rows = fixture::final_rows(role);
    final_rows.push(fixture::one_foreign_row());

    let measured = MeasuredAttempt {
        channels: fixture::measured_prefix(),
        before: fixture::persist(
            &directory,
            ObservedRowSetLabel::SeededBeforeMeasurement,
            seeded,
        ),
        after: fixture::persist(
            &directory,
            ObservedRowSetLabel::AfterSaturatedBatch,
            final_rows,
        ),
    };
    let expected_channels =
        serde_json::to_value(&measured.channels).expect("measured channel evidence serializes");

    let failure = validate_final_composition(
        fixture::key(role),
        measured,
        fixture::owned_owner(),
        fixture::foreign_owner(),
    )
    .expect_err("one foreign row in the Arm's view is a read-set leak, not a passing composition");

    assert_eq!(
        failure.kind,
        FailureKind::SemanticsOrSecurity,
        "a leaked row is the candidate's answer about its read set, not an operational fault"
    );
    assert_eq!(
        failure.stage,
        FailureStage::AfterFirstSample,
        "the composition check runs only once the saturated batch has confirmed"
    );
    assert_eq!(
        serde_json::to_value(&failure.measured).expect("measured channel evidence serializes"),
        expected_channels,
        "a refusal retains the channels that ran, whole and in order"
    );

    let rendered = format!("{:#}", failure.error);
    assert!(
        rendered.contains("read-set leak"),
        "the failure must carry the census's own reason, got {rendered}"
    );
}
