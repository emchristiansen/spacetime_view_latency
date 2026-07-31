//! A record carrying `\n` or `\r` is refused, and nothing is written.

use crate::view_read_set_campaign::campaign_params::SEEDED_ROW_SET_STEM;
use crate::view_read_set_campaign::composition_validation::observed_row_set::{
    ObservedRowSet, ROW_SET_EXTENSION,
};
use crate::view_read_set_campaign::composition_validation::observed_row_set_label::ObservedRowSetLabel;

use super::attempt_directory::attempt_directory;
use super::row::row;

/// Coverage: "one line, one row" is the artifact's whole parse contract, and a record containing a
/// line break breaks it in the worst available way — one logical row forges two artifact lines, so
/// the retained file hashes and re-parses as a composition the observation never held. A reader
/// checking `wc -l` against the recorded row count would see the forged rows and the mismatch
/// together, but a reader parsing the file would simply believe it.
///
/// Both bytes are rejected. `\r` is the one that would otherwise slip through: it does not split a
/// line for `str::lines`, so an encoder checking only `\n` would emit a file that a
/// carriage-return-aware reader splits differently than the writer intended.
///
/// No seeded or measured payload can contain either byte — both are frozen constants — so a
/// rejection here is always evidence of a fault rather than of a legitimate payload this encoding
/// cannot express. That is why it fails loud instead of escaping.
///
/// Checked through persistence, so the test also establishes that the rejection precedes any write:
/// a partial artifact would be an unreferenced file in the attempt's directory with the phase's one
/// exclusive creation already spent.
#[test]
fn a_record_containing_a_line_break_is_rejected() {
    for (byte_name, forged) in [
        ("newline", "seeded\n1:forged:row"),
        ("carriage return", "seeded\r1:forged:row"),
    ] {
        let directory = attempt_directory(&format!("line-break-{}", byte_name.replace(' ', "-")));

        let error = match ObservedRowSet::persisted(
            &directory,
            ObservedRowSetLabel::SeededBeforeMeasurement,
            vec![row(4, forged)],
        ) {
            Err(error) => error,
            Ok(_) => panic!("a record containing a {byte_name} must not acquire an artifact"),
        };
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains("entity_uuid=4"),
            "the {byte_name} failure must name the offending row, got {rendered}"
        );

        let path = directory
            .path()
            .join(format!("{SEEDED_ROW_SET_STEM}{ROW_SET_EXTENSION}"));
        assert!(
            !path.exists(),
            "the {byte_name} rejection must happen before persistence, leaving no artifact at {}",
            path.display()
        );
    }
}
