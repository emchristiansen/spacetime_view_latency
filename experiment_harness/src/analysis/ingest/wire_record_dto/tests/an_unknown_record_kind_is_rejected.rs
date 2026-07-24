//! An unknown record kind is rejected while decoding the envelope's `record`, so a line tagged with a
//! kind outside the closed `{Manifest, Dose}` set never reaches body dispatch.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn an_unknown_record_kind_is_rejected() {
    let body = wire_lines::dose_body();
    // A record whose kind is neither `Manifest` nor `Dose`.
    let line = format!(r#"{{"record":{{"seq":0,"kind":"Frobnicate"}},"body":{body}}}"#);

    let error = WireRecordDto::parse(&line).expect_err("an unknown record kind is rejected");
    assert!(
        error.contains("unknown variant `Frobnicate`"),
        "the diagnostic names the unknown record kind: {error}"
    );
}
