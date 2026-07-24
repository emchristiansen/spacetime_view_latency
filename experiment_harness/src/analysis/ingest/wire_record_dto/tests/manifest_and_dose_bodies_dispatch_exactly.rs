//! The record kind dispatches the body into exactly one DTO: a manifest line decodes only the manifest
//! body, a dose line decodes only the observation body, and a kind paired with the other kind's body is
//! rejected rather than silently coerced.

use crate::analysis::ingest::wire_record_dto::WireRecordDto;

use super::wire_lines;

#[test]
fn manifest_and_dose_bodies_dispatch_exactly() {
    let manifest_record = wire_lines::manifest_record();
    let dose_record = wire_lines::dose_record();
    let manifest_body = wire_lines::manifest_body();
    let dose_body = wire_lines::dose_body();

    // Each kind decodes its own body into its own variant.
    let manifest_line = wire_lines::line(&manifest_record, &manifest_body);
    assert!(
        matches!(
            WireRecordDto::parse(&manifest_line).expect("a manifest line decodes"),
            WireRecordDto::Manifest { .. }
        ),
        "the Manifest kind selects the manifest body"
    );

    let dose_line = wire_lines::line(&dose_record, &dose_body);
    assert!(
        matches!(
            WireRecordDto::parse(&dose_line).expect("a dose line decodes"),
            WireRecordDto::Dose { .. }
        ),
        "the Dose kind selects the observation body"
    );

    // A kind paired with the other kind's body is rejected, not coerced.
    let manifest_kind_dose_body = wire_lines::line(&manifest_record, &dose_body);
    WireRecordDto::parse(&manifest_kind_dose_body)
        .expect_err("a Manifest kind rejects an observation body");

    let dose_kind_manifest_body = wire_lines::line(&dose_record, &manifest_body);
    WireRecordDto::parse(&dose_kind_manifest_body)
        .expect_err("a Dose kind rejects a manifest body");
}
