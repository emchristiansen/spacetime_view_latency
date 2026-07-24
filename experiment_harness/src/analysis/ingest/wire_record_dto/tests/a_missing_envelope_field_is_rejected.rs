//! A missing required envelope field is rejected: an envelope without `body` cannot decode, so a line
//! is never accepted with an absent record body.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn a_missing_envelope_field_is_rejected() {
    let record = wire_lines::manifest_record();
    // Only `record`; the required `body` field is absent.
    let line = format!(r#"{{"record":{record}}}"#);

    let error = WireRecordDto::parse(&line).expect_err("a missing envelope field is rejected");
    assert!(
        error.contains("missing field `body`"),
        "the diagnostic names the missing envelope field: {error}"
    );
}
