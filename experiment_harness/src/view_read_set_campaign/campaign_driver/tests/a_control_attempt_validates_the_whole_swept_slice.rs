//! The Control's correct attempt validates against its own role's opposite foreign-slice rule.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::campaign_driver::{validate_final_composition, MeasuredAttempt};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

use super::composition_fixture as fixture;

/// Coverage: the two roles are judged by opposite foreign-slice rules, and the stage chooses which
/// rule applies by reading the role off the attempt key. A stage that ignored the key's role — or
/// took the Arm's rule as a default — would still pass the Arm test and fail only here, where the
/// entire swept slice must be *present* rather than absent.
///
/// The foreign count is asserted against the scale point rather than a literal, which ties it to the
/// frozen ladder instead of to a number typed here.
#[test]
fn a_control_attempt_validates_the_whole_swept_slice() {
    let role = RunRole::Control;
    let directory = fixture::attempt_directory("control-validates", role);
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

    let (_channels, composition) = fixture::accepted(validate_final_composition(
        fixture::key(role),
        measured,
        fixture::owned_owner(),
        fixture::foreign_owner(),
    ));

    assert_eq!(
        composition.observed_owned_rows(),
        10,
        "the Control observes the same owned slice the Arm does"
    );
    assert_eq!(
        composition.observed_foreign_rows(),
        fixture::scale().scale(),
        "the direct-table Control must observe the entire seeded foreign slice at this rung"
    );
    assert_eq!(
        composition.client_cache_rows(),
        composition.observed_owned_rows() + composition.observed_foreign_rows(),
        "the cache count is the two validated slice counts and nothing else"
    );
}
