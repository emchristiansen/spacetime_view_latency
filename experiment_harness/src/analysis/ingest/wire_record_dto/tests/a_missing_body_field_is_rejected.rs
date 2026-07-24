//! A missing required body field is rejected by the second decode pass: a dose body without its
//! `observation` field cannot decode, so the kind-matched body contract is total.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn a_missing_body_field_is_rejected() {
    let record = wire_lines::dose_record();
    // An empty dose body: the required `observation` field is absent.
    let line = wire_lines::line(&record, "{}");

    let error = WireRecordDto::parse(&line).expect_err("a missing body field is rejected");
    assert!(
        error.contains("missing field `observation`"),
        "the diagnostic names the missing body field: {error}"
    );
}
