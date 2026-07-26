//! An owned-range row held by the wrong identity fails the check.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: the completed Pilot established that a count-only check passes a role returning the
/// right *number* of wrong rows, which is why the expectation names the canonical-hex owner each key
/// range must carry. Ownership is also the exact property the sender-scoped view routes on, so a row
/// in the owned range owned by someone else is either a seeding fault or a view returning another
/// identity's row under a key that looks like the subscriber's.
///
/// The defect here is a single row's owner and nothing else: the key is in range, the payload is the
/// phase's required one, and the count is right. Only an owner comparison catches it.
#[test]
fn a_wrong_owner_fails_the_census() {
    let directory = fixture::attempt_directory("wrong-owner");

    let mut rows = fixture::seeded_rows(RunRole::Arm);
    let misowned = rows
        .first_mut()
        .expect("the fixture's owned slice is not empty");
    let misowned_key = misowned.entity_uuid;
    misowned.owner = fixture::foreign_owner();

    let before = fixture::persist(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        rows,
    );
    let after = fixture::persist(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        fixture::final_rows(RunRole::Arm),
    );

    let error =
        ValidatedComposition::validate(fixture::transition(RunRole::Arm), before, after, 10, 10, 1)
            .expect_err("an owned row held by another identity must fail the census");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains(&format!("entity_uuid={misowned_key}")),
        "the failure must name the misowned row, got {rendered}"
    );
    assert!(
        rendered.contains("not the measured identity"),
        "the failure must say the owner is not the one its key range was seeded for, got {rendered}"
    );
}
