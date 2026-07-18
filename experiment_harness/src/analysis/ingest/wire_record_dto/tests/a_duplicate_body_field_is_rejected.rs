//! A duplicated body field is rejected by the second (body) decode pass, proving the retained raw body
//! is decoded under the same closed contract as the envelope, not staged through a value that would
//! collapse duplicates.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn a_duplicate_body_field_is_rejected() {
    let record = wire_lines::dose_record();
    let observation = wire_lines::observation();
    // Two `observation` fields inside the dose body.
    let body = format!(r#"{{"observation":{observation},"observation":{observation}}}"#);
    let line = wire_lines::line(&record, &body);

    let error = WireRecordDto::parse(&line).expect_err("a duplicate body field is rejected");
    assert!(
        error.contains("duplicate field `observation`"),
        "the diagnostic names the duplicated body field: {error}"
    );
}
