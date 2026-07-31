//! One foreign row in the Arm's result set fails the check — the candidate's security gate.

use crate::module_artifact::bindings::EntityOwner;
use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::campaign_params::{GLOBAL_KEY_BASE, SEEDED_ROW_PAYLOAD};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: this is the candidate's security claim, and the one failure that must never be treated
/// as a small discrepancy. The sender-scoped view exists to return only the sender's own rows, so a
/// single leaked foreign row is the experiment's answer about the candidate — not a rounding error
/// in a count.
///
/// The leaked row is otherwise *perfectly well-formed*: correct foreign owner, correct seeded
/// payload, a key inside the preregistered foreign range. That is deliberate — it is caught for
/// being present at all, by the Arm's closed
/// [`None`](crate::view_read_set_campaign::composition_validation::expected_foreign_visibility::ExpectedForeignVisibility::None)
/// visibility, rather than by looking malformed. A check that only validated the *contents* of
/// foreign rows would pass this observation.
///
/// It is injected into the before-phase set, so the failure is attributed to the pre-measurement
/// observation and the diagnosis names which artifact was wrong.
#[test]
fn a_leaked_foreign_row_fails_the_arm() {
    let directory = fixture::attempt_directory("arm-leak");

    let mut leaked = fixture::seeded_rows(RunRole::Arm);
    leaked.push(EntityOwner {
        entity_uuid: GLOBAL_KEY_BASE,
        owner: fixture::foreign_owner(),
        record: SEEDED_ROW_PAYLOAD.to_string(),
    });

    let before = fixture::persist(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        leaked,
    );
    let after = fixture::persist(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        fixture::final_rows(RunRole::Arm),
    );

    let error = ValidatedComposition::validate(fixture::transition(RunRole::Arm), before, after)
        .expect_err("one foreign row in the Arm's view is a read-set leak, not a discrepancy");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("read-set leak"),
        "the failure must be reported as the leak it is, got {rendered}"
    );
    assert!(
        rendered.contains("before the first measured write"),
        "the failure must name which of the two observations it was found in, got {rendered}"
    );
}
