//! The two phases have their own frozen filenames, and neither can be written twice.

use crate::view_read_set_campaign::campaign_params::{
    AFTER_SATURATED_ROW_SET_STEM, SEEDED_ROW_SET_STEM,
};
use crate::view_read_set_campaign::composition_validation::observed_row_set::{
    ObservedRowSet, ROW_SET_EXTENSION,
};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

use super::attempt_directory::attempt_directory;
use super::row::row;

/// Coverage: two claims that only hold together. The phase labels must map to *different* frozen
/// filenames — otherwise an attempt's second observation would collide with its first and the
/// after-state would never be retained — and each filename must be creatable only once, so an
/// observation can never overwrite another.
///
/// Overwriting is the specific hazard: the recorded digest of the replaced artifact would still name
/// rows the file no longer holds, and every later recomputation would report corruption at a file
/// that was simply rewritten. `O_EXCL` turns that into a failure at the write.
///
/// The paths are checked against the frozen stems rather than against each other, since being
/// distinct is not enough — they are recorded in findings, so they are part of the preregistration.
#[test]
fn each_phase_writes_its_own_artifact_exactly_once() {
    let directory = attempt_directory("phase-artifacts");
    let seeded_rows = vec![row(0, "view-read-set-campaign-seeded-payload")];

    let before = ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        seeded_rows.clone(),
    )
    .expect("the first observation of a phase persists");
    let after = ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::AfterSaturatedBatch,
        vec![row(
            0,
            "view-read-set-campaign-mutation:e1-saturated-queue-growth:990",
        )],
    )
    .expect("the other phase has its own filename and persists too");

    assert_eq!(
        before.path(),
        directory
            .path()
            .join(format!("{SEEDED_ROW_SET_STEM}{ROW_SET_EXTENSION}")),
        "the before-phase artifact is filed under its frozen stem"
    );
    assert_eq!(
        after.path(),
        directory
            .path()
            .join(format!("{AFTER_SATURATED_ROW_SET_STEM}{ROW_SET_EXTENSION}")),
        "the after-phase artifact is filed under its own frozen stem"
    );

    let error = ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        seeded_rows,
    )
    .expect_err("a phase's artifact must be creatable exactly once");
    assert!(
        format!("{error:#}").contains(SEEDED_ROW_SET_STEM),
        "the exclusive-creation failure must name the artifact it refused to overwrite, got \
         {error:#}"
    );

    let (_, _, before_hex, _) = before.content_address();
    let (_, _, after_hex, _) = after.content_address();
    assert_ne!(
        before_hex, after_hex,
        "the two phases hold different payloads, so their content addresses must differ"
    );
}
