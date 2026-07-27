//! A row belonging to neither preregistered key range fails the check outright.

use crate::module_artifact::bindings::EntityOwner;
use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::campaign_params::SEEDED_ROW_PAYLOAD;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: the two key ranges are the whole of what any attempt ever seeds, so a row outside both
/// means the observation is not the composition it claims to be — a server carrying rows from
/// another campaign, a data directory that was not fresh, or a subscription pointed somewhere
/// unexpected. Every one of those invalidates the measurement rather than perturbing it.
///
/// It has to be rejected *outright* rather than merely uncounted. An unexplained row that fell
/// through both range checks silently would leave the two slice counts correct and the observation
/// passing, which is exactly how a stale data directory would go unnoticed.
///
/// The stray key sits between the two ranges — above the owned slice, far below the foreign base —
/// which is where a plausible off-by-one or a leftover key would land.
#[test]
fn a_key_outside_both_ranges_fails_the_census() {
    let directory = fixture::attempt_directory("stray-key");

    let mut rows = fixture::seeded_rows(RunRole::Arm);
    rows.push(EntityOwner {
        entity_uuid: 500_000,
        owner: fixture::owned_owner(),
        record: SEEDED_ROW_PAYLOAD.to_string(),
    });

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

    let error = ValidatedComposition::validate(fixture::transition(RunRole::Arm), before, after)
        .expect_err("a row outside both preregistered ranges must fail the census outright");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains("entity_uuid=500000"),
        "the failure must name the unexplained row, got {rendered}"
    );
    assert!(
        rendered.contains("outside both preregistered key ranges"),
        "the failure must say the key belongs to neither seeded range, got {rendered}"
    );
}
