//! The Control's correct attempt validates, with the entire swept slice present.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: the two roles are judged by opposite foreign-slice rules, and a validator that had
/// collapsed them into one numeric comparison would still pass the Arm. The Control's direct-table
/// subscription must return the *whole* seeded foreign slice — that is what makes it the composition
/// baseline the Arm is compared against, rather than an independent measurement.
///
/// The scale point is the ladder's first rung, so "the whole slice" is a concrete thousand rows;
/// asserting the count against the scale point rather than a literal is what ties this to the frozen
/// ladder instead of to a number typed here.
#[test]
fn the_control_census_accepts_the_whole_foreign_slice() {
    let directory = fixture::attempt_directory("control-accepts");
    let before = fixture::persist(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        fixture::seeded_rows(RunRole::Control),
    );
    let after = fixture::persist(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        fixture::final_rows(RunRole::Control),
    );

    let validated = ValidatedComposition::validate(
        fixture::transition(RunRole::Control),
        before,
        after,
        1_010,
        1_010,
        1,
    )
    .expect("the Control's required composition must validate");

    assert_eq!(
        validated.observed_owned_rows(),
        10,
        "the Control observes the same owned slice the Arm does"
    );
    assert_eq!(
        validated.observed_foreign_rows(),
        fixture::scale().scale(),
        "the direct-table Control must observe the entire seeded foreign slice at this rung"
    );
}
