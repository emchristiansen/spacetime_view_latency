//! An unknown body field is rejected by the body DTO's `deny_unknown_fields`, so the closed contract is
//! enforced on the retained raw body just as strictly as on the envelope.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn an_unknown_body_field_is_rejected() {
    let record = wire_lines::dose_record();
    let observation = wire_lines::observation();
    // A well-formed observation plus a stray `surprise` field in the dose body.
    let body = format!(r#"{{"observation":{observation},"surprise":1}}"#);
    let line = wire_lines::line(&record, &body);

    let error = WireRecordDto::parse(&line).expect_err("an unknown body field is rejected");
    assert!(
        error.contains("unknown field `surprise`"),
        "the diagnostic names the unknown body field: {error}"
    );
}
