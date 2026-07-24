//! Shared fixtures: build real NDJSON wire lines from `Serialize`-only production fixtures.
//!
//! The nested sub-values (record identity, manifest, reference, observation) are the exact bytes the
//! production types serialize, so the closed contract these tests exercise cannot drift from the sink's
//! output for the parts that matter. Only the two envelope keys (`record`, `body`) and the manifest
//! body's two keys are composed textually — the same wire field names the DTOs deserialize — so a test
//! can place them in any order or inject a duplicate/unknown/missing field the sink never would.

use crate::dataset::dose_index::DoseIndex;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::dose_observation::DoseObservation;
use crate::observation::manifest_reference::ManifestReference;
use crate::observation::record_id::RecordId;
use crate::observation::record_kind::RecordKind;
use crate::observation::record_seq::RecordSeq;

/// The serialized record identity of a manifest line: `{"seq":0,"kind":"Manifest"}`.
pub(super) fn manifest_record() -> String {
    let record = RecordId::new(RecordSeq::zero(), RecordKind::Manifest);
    serde_json::to_string(&record).expect("a record identity serializes")
}

/// The serialized record identity of a dose line: `{"seq":0,"kind":{"Dose":1}}`. The dose index is not
/// cross-checked against the body here — that is the `validate` pass's job — so the first ladder index
/// suffices for every parse-layer test.
pub(super) fn dose_record() -> String {
    let record = RecordId::new(RecordSeq::zero(), RecordKind::Dose(DoseIndex::ALL[0]));
    serde_json::to_string(&record).expect("a record identity serializes")
}

/// The bare serialized dose observation — the value wrapped by a dose body's `observation` field.
pub(super) fn observation() -> String {
    serde_json::to_string(&DoseObservation::fixture()).expect("an observation serializes")
}

/// A well-formed dose body: `{"observation":<observation>}`.
pub(super) fn dose_body() -> String {
    format!(r#"{{"observation":{}}}"#, observation())
}

/// A well-formed manifest body: `{"reference":<reference>,"manifest":<manifest>}`, where the reference
/// is derived from the same fixture manifest exactly as the sink derives it.
pub(super) fn manifest_body() -> String {
    let manifest = ValidatedRunManifest::fixture();
    let reference = ManifestReference::of(&manifest);
    format!(
        r#"{{"reference":{},"manifest":{}}}"#,
        serde_json::to_string(&reference).expect("a manifest reference serializes"),
        serde_json::to_string(&manifest).expect("a manifest serializes"),
    )
}

/// Compose a whole envelope line from a serialized record identity and body: `{"record":R,"body":B}`.
pub(super) fn line(record: &str, body: &str) -> String {
    format!(r#"{{"record":{record},"body":{body}}}"#)
}
