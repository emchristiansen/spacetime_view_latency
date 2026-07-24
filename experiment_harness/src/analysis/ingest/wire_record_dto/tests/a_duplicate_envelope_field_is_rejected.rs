//! A duplicated envelope field is rejected structurally by serde's derived struct visitor, so a wire
//! line can never silently collapse two `record` fields to one.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn a_duplicate_envelope_field_is_rejected() {
    let record = wire_lines::dose_record();
    let body = wire_lines::dose_body();
    // Two `record` fields on the envelope.
    let line = format!(r#"{{"record":{record},"record":{record},"body":{body}}}"#);

    let error = WireRecordDto::parse(&line).expect_err("a duplicate envelope field is rejected");
    assert!(
        error.contains("duplicate field `record`"),
        "the diagnostic names the duplicated envelope field: {error}"
    );
}
