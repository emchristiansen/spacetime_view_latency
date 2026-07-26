//! The Arm's correct attempt validates, and reports its slices exactly.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: every other test in this tree asserts a *failure*, and a validator that rejected
/// everything would pass all of them. This is the one that says the gate admits the composition the
/// protocol actually prescribes — the Arm's sender-scoped view returning its own ten rows and
/// nothing else, before and after the saturated batch.
///
/// The recorded per-slice counts are asserted too, since they are what a reader sees in the ledger:
/// ten owned rows, and zero foreign rows, the latter being the candidate's security gate *passing*
/// rather than an absence of evidence.
#[test]
fn the_arm_census_accepts_the_required_composition() {
    let directory = fixture::attempt_directory("arm-accepts");
    let before = fixture::persist(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        fixture::seeded_rows(RunRole::Arm),
    );
    let after = fixture::persist(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        fixture::final_rows(RunRole::Arm),
    );

    let validated =
        ValidatedComposition::validate(fixture::transition(RunRole::Arm), before, after, 10, 10, 1)
            .expect("the Arm's required composition must validate");

    assert_eq!(
        validated.observed_owned_rows(),
        10,
        "the Arm observes the whole frozen owned slice"
    );
    assert_eq!(
        validated.observed_foreign_rows(),
        0,
        "the sender-scoped view must leak none of the foreign slice; zero is the gate passing"
    );
    assert_eq!(
        validated.scale(),
        fixture::scale(),
        "the finding carries the scale point its transition fixed"
    );

    let (retained_before, retained_after) = validated.observations();
    assert_ne!(
        retained_before.path(),
        retained_after.path(),
        "the finding must point at two distinct retained artifacts"
    );
}
