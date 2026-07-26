//! An observation holding no rows is an empty artifact, not a blank line.

use std::fs;

use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

use super::attempt_directory::attempt_directory;

/// Coverage: an [`ObservedRowSet`] holds a subscriber's *whole* result set, so a wholly empty one is
/// never a valid attempt — every role must observe the ten owned rows, and the Arm's security gate
/// concerns the foreign slice within that result set, not the result set itself. Composition
/// validation is what rejects it, at the owned census.
///
/// Persistence must nevertheless handle it, and canonically: an empty observation is exactly what a
/// failed subscription produces, and retaining it is what lets the resulting rejection be diagnosed
/// against a real artifact instead of an absence. So the empty case has to encode consistently with
/// the terminated one — zero lines for zero rows, keeping `wc -l` equal to the recorded row count at
/// both ends — while remaining a diagnostic observation rather than a finding.
///
/// A blank line would be the natural bug — one `\n` for "nothing" — and it would make an empty
/// artifact indistinguishable from an artifact holding one unparseable row.
///
/// This goes through persistence rather than the encoder alone, because the recorded row count is
/// part of the same claim and only the constructor produces it.
#[test]
fn zero_rows_produce_an_empty_file() {
    let directory = attempt_directory("zero-rows");

    let observed = ObservedRowSet::persisted(
        &directory,
        ObservedRowSetLabel::SeededBeforeMeasurement,
        Vec::new(),
    )
    .expect("an observation holding no rows is persistable");

    let bytes = fs::read(observed.path()).expect("reading back the retained artifact");
    assert!(
        bytes.is_empty(),
        "zero rows must produce an empty file, not a blank line; got {bytes:?}"
    );

    let (_, _, _, row_count) = observed.content_address();
    assert_eq!(
        row_count, 0,
        "the recorded row count of an empty observation is zero"
    );
    assert!(
        observed.rows().is_empty(),
        "the retained rows must agree with the empty artifact"
    );
}
