//! An owned row carrying another write's payload fails the after-phase check.

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;
use crate::view_read_set_campaign::composition_validation::validated_composition::ValidatedComposition;

use super::fixture;

/// Coverage: the after-phase expectation is per-key — offset `k` must carry the payload of the last
/// write to reach it — so a check that merely required *some* mutation payload everywhere would pass
/// a batch whose writes landed out of order, or one that stopped early and left a stale payload
/// behind. Ordering and completeness of the measured batch are exactly what E1 is measuring, so a
/// stale payload is a fault in the measurement rather than a cosmetic mismatch.
///
/// The defect is one row carrying the payload of a *different, real* write from the same batch:
/// right prefix, right channel tag, wrong index. That is the near-miss a laxer comparison admits,
/// and it is the shape an actually-dropped write would produce.
#[test]
fn a_wrong_payload_fails_the_census() {
    let directory = fixture::attempt_directory("wrong-payload");

    let mut rows = fixture::final_rows(RunRole::Arm);
    let stale = rows
        .first_mut()
        .expect("the fixture's owned slice is not empty");
    let stale_key = stale.entity_uuid;
    // The payload the *previous* cycle left at this offset: a real write of this very batch, ten
    // writes earlier — what a dropped final write would leave behind.
    stale.record = "view-read-set-campaign-mutation:e1-saturated-queue-growth:980".to_string();

    let before = fixture::persist(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        fixture::seeded_rows(RunRole::Arm),
    );
    let after = fixture::persist(&directory, ObservedRowSetLabel::AfterSaturatedBatch, rows);

    let error = ValidatedComposition::validate(fixture::transition(RunRole::Arm), before, after)
        .expect_err("a stale payload on an owned key must fail the after-phase census");
    let rendered = format!("{error:#}");
    assert!(
        rendered.contains(&format!("entity_uuid={stale_key}")),
        "the failure must name the row carrying the wrong payload, got {rendered}"
    );
    assert!(
        rendered.contains("this phase requires"),
        "the failure must state that the payload is not the phase's required one, got {rendered}"
    );
    assert!(
        rendered.contains("saturated batch confirmed"),
        "the failure must name the after-phase observation it was found in, got {rendered}"
    );
}
