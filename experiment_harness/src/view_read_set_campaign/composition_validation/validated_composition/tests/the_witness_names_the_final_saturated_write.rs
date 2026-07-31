//! The witness is the batch's last write — key 9, pre-image 989, final 999 — and no caller chooses
//! it.

use anyhow::Result;

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::campaign_params::{
    MUTATION_PAYLOAD_PREFIX, SATURATED_MUTATION_TAG, SEEDED_ROW_PAYLOAD,
};
use crate::view_read_set_campaign::composition_validation::composition_transition_expectation::CompositionTransitionExpectation;
use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// The signature of the composition check, restated so that regaining any free input breaks the
/// build.
///
/// The spec removed the free `witness: OwnedSliceOffset` parameter precisely because a caller could
/// point it at any of the ten owned keys — nine of which were last written earlier in the batch — and
/// so claim a finding about the final mutation while evidencing a different one. It later removed the
/// free `delivered_rows`, `client_cache_rows` and `subscription_handles` inputs for the same reason:
/// nothing in the comparison could contradict them, and two of them cannot bear on composition at
/// all. What remains is a transition and two retained artifacts — the finding is exactly a function
/// of what a reader can replay.
///
/// Those absences are properties of the *signature*, which no runtime assertion can observe: a test
/// can only ever call the function that exists. Binding it to an explicit function type is the check
/// that can fail, and it fails at compile time, which is where the guarantee lives.
const NO_FREE_CALLER_INPUTS: fn(
    CompositionTransitionExpectation,
    ObservedRowSet,
    ObservedRowSet,
) -> Result<ValidatedComposition> = ValidatedComposition::validate;

/// Coverage: the recorded witness is what a reader consults to see that the attempt's *final*
/// measured mutation actually landed, so its three payloads must be the right three — and they carry
/// different provenance, which the field names have to keep apart.
///
/// The frozen numbers are asserted exactly: entity key 9, because the 1,000-write batch over a
/// ten-row slice ends at offset 9; the pre-image from write 989, one whole cycle earlier; the final
/// payload from write 999. The pre-image is the field that can never be observed — capturing the
/// state between two writes of a saturated batch would mean stopping the pipeline under measurement
/// — so it is derived, named as an expectation, and checked here against the frozen schedule rather
/// than against the artifacts.
///
/// The seeded payload is asserted too, since it is the value that makes the record cross-checkable
/// against the retained before-artifact — and it must be the *seeded* constant, not the pre-image:
/// conflating them would make the witness say the final write replaced the seeding, which is the
/// weaker claim the spec rejected.
#[test]
fn the_witness_names_the_final_saturated_write() {
    let directory = fixture::attempt_directory("witness-fields");
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

    let validated = NO_FREE_CALLER_INPUTS(fixture::transition(RunRole::Arm), before, after)
        .expect("the Arm's required composition must validate");

    assert_eq!(
        validated.witness_entity_key(),
        fixture::owned_key(9),
        "the frozen batch's last write targets owned-slice offset 9"
    );
    assert_eq!(
        validated.witness_seeded_payload(),
        SEEDED_ROW_PAYLOAD,
        "the seeded field must hold the payload validated in the pre-E2 artifact"
    );
    assert_eq!(
        validated.witness_expected_payload_before_final_write(),
        format!("{MUTATION_PAYLOAD_PREFIX}:{SATURATED_MUTATION_TAG}:989"),
        "the pre-image is write 989's payload, one ten-row cycle before the final write"
    );
    assert_eq!(
        validated.witness_payload_after(),
        format!("{MUTATION_PAYLOAD_PREFIX}:{SATURATED_MUTATION_TAG}:999"),
        "the after field must hold the payload validated in the post-E1 artifact"
    );
    assert_ne!(
        validated.witness_expected_payload_before_final_write(),
        validated.witness_payload_after(),
        "a byte-identical update is elided at the pinned commit, so an equal pair would mean the \
         final write measured nothing"
    );
}
