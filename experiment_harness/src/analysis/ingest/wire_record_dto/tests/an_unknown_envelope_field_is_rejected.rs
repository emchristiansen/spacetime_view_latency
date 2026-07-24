//! An unknown envelope field is rejected by `deny_unknown_fields`, so the closed outer contract admits
//! exactly `record` and `body`.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn an_unknown_envelope_field_is_rejected() {
    let record = wire_lines::dose_record();
    let body = wire_lines::dose_body();
    // A stray `surprise` field alongside the two known envelope fields.
    let line = format!(r#"{{"record":{record},"body":{body},"surprise":1}}"#);

    let error = WireRecordDto::parse(&line).expect_err("an unknown envelope field is rejected");
    assert!(
        error.contains("unknown field `surprise`"),
        "the diagnostic names the unknown envelope field: {error}"
    );
}
