//! An owned slice short of one row fails the check.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: the owned count is exact, not a lower bound, because the measured mutation updates rows
/// in place — cardinality is constant at every committed state the attempt passes through. A missing
/// owned row therefore means the seeding was incomplete or a measured write deleted rather than
/// updated, and either invalidates the per-write cost the channel is measuring.
///
/// The row is dropped from the *after* observation only, so the before phase still holds all ten.
/// That is the case a per-phase check catches and a "both phases agree" check would not: the two
/// observations disagreeing is the symptom, but the requirement is against the frozen slice size,
/// which is what makes a *pair* of equally-short observations fail too.
///
/// The exact count is named in the failure, since "nine of ten" is the diagnosis and "some rows were
/// missing" is not.
#[test]
fn a_missing_owned_row_fails_the_census() {
    let directory = fixture::attempt_directory("missing-owned-row");

    let mut rows = fixture::final_rows(RunRole::Arm);
    let dropped = rows.remove(0);

    let before = fixture::persist(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        fixture::seeded_rows(RunRole::Arm),
    );
    let after = fixture::persist(&directory, ObservedRowSetLabel::AfterSaturatedBatch, rows);

    let error = ValidatedComposition::validate(fixture::transition(RunRole::Arm), before, after)
        .expect_err("an owned slice short of a row must fail the census");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("holds 9 of the measured identity's 10 own rows"),
        "the failure must name the exact shortfall, got {rendered}"
    );
    assert_eq!(
        dropped.entity_uuid,
        fixture::owned_key(0),
        "the dropped row is the owned slice's first key, which the fixture builds in offset order"
    );
}
