//! The Arm's correct attempt joins into a finding at its own identity, and its measured channels
//! come back untouched.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::campaign_driver::{validate_final_composition, MeasuredAttempt};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

use super::composition_fixture as fixture;

/// Coverage: the stage is a join, so what it can get wrong is which transition it mints and what it
/// does with the channels. Both are asserted here.
///
/// The finding is checked to carry *this attempt's* scale point rather than merely to exist: the
/// expectation is derived from the key handed in, so a stage that minted it from anything else would
/// still validate these rows and attach the wrong identity's transition to them.
///
/// The channels are compared as their full structural values, snapshotted from the very `Vec` that
/// is then moved into the stage. Comparing channel tags would pass while a reduction, a sample, or a
/// retained batch had been altered, and rebuilding a second fixture prefix as the expectation would
/// compare the stage against a reconstruction rather than against its own input.
#[test]
fn an_arm_attempt_validates_its_own_read_set_and_keeps_its_channels() {
    let role = RunRole::Arm;
    let directory = fixture::attempt_directory("arm-validates", role);
    let measured = MeasuredAttempt {
        channels: fixture::measured_prefix(),
        before: fixture::persist(
            &directory,
            ObservedRowSetLabel::SeededBeforeMeasurement,
            fixture::seeded_rows(role),
        ),
        after: fixture::persist(
            &directory,
            ObservedRowSetLabel::AfterSaturatedBatch,
            fixture::final_rows(role),
        ),
    };
    let expected_channels =
        serde_json::to_value(&measured.channels).expect("measured channel evidence serializes");

    let (channels, composition) = fixture::accepted(validate_final_composition(
        fixture::key(role),
        measured,
        fixture::owned_owner(),
        fixture::foreign_owner(),
    ));

    assert_eq!(
        serde_json::to_value(&channels).expect("measured channel evidence serializes"),
        expected_channels,
        "the measured prefix must come back whole and in order, every field unchanged"
    );
    assert_eq!(
        composition.scale(),
        fixture::scale(),
        "the finding must carry the scale point of the attempt whose identity minted its transition"
    );
    assert_eq!(
        composition.observed_owned_rows(),
        10,
        "the Arm observes the whole frozen owned slice"
    );
    assert_eq!(
        composition.observed_foreign_rows(),
        0,
        "the sender-scoped view must leak none of the foreign slice; zero is the gate passing"
    );
    assert_eq!(
        composition.client_cache_rows(),
        10,
        "the Arm's whole client cache is its own ten rows"
    );
}
