//! The first line that fails to decode aborts with its 1-based position: a valid first line followed by
//! a malformed second line reports line 2, proving the position is the 1-based file line, not the
//! 0-based enumeration index.

use crate::analysis::ingest::parse_ndjson::parse_ndjson;
use crate::dataset::dose_index::DoseIndex;
use crate::observation::dose_observation::DoseObservation;
use crate::observation::record_id::RecordId;
use crate::observation::record_kind::RecordKind;
use crate::observation::record_seq::RecordSeq;

#[test]
fn a_malformed_line_is_numbered_one_based() {
    // A well-formed dose line built from the same `Serialize`-only fixtures the sink writes.
    let record = serde_json::to_string(&RecordId::new(
        RecordSeq::zero(),
        RecordKind::Dose(DoseIndex::ALL[0]),
    ))
    .expect("a record identity serializes");
    let body = format!(
        r#"{{"observation":{}}}"#,
        serde_json::to_string(&DoseObservation::fixture()).expect("an observation serializes"),
    );
    let valid = format!(r#"{{"record":{record},"body":{body}}}"#);

    // The valid line is 1; the malformed line is 2.
    let contents = format!("{valid}\nthis is not a wire record");
    let error = parse_ndjson(&contents).expect_err("the second line is malformed");

    assert_eq!(
        error.line_number(),
        2,
        "the malformed line is the second file line, numbered from one"
    );
}
