//! Field-order independence: a line whose `body` is serialized before `record` still decodes, because
//! the whole envelope (hence its kind) is read before the retained raw body is decoded.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn body_field_may_precede_the_record_field() {
    let record = wire_lines::dose_record();
    let body = wire_lines::dose_body();
    // `body` deliberately precedes `record` on the line — the order the sink never emits.
    let line = format!(r#"{{"body":{body},"record":{record}}}"#);

    let decoded = WireRecordDto::parse(&line).expect("a body-first line decodes");
    assert!(
        matches!(decoded, WireRecordDto::Dose { .. }),
        "the Dose kind selects the observation body regardless of field order"
    );
}
