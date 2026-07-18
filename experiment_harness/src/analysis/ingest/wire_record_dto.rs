//! The whole-line wire envelope: a record identity plus its kind-dispatched body.

use crate::analysis::ingest::manifest_body_dto::ManifestBodyDto;
use crate::analysis::ingest::observation_body_dto::ObservationBodyDto;
use crate::analysis::ingest::record_id_dto::RecordIdDto;

/// One decoded NDJSON line: the record identity and the body whose shape its
/// [`kind`](crate::analysis::ingest::record_kind_dto::RecordKindDto) selects.
///
/// The body's type is chosen by the record's *kind*, which is nested inside `record` rather than being
/// an external tag on the line, so this envelope cannot be a plain derived `Deserialize` (serde would
/// need the discriminant to be a sibling of the variant payloads). It is decoded by [`Self::parse`],
/// which reads the kind first and then decodes the body into exactly the matching DTO. That keeps the
/// closed-contract guarantees — unknown/duplicate/missing-field rejection on both the outer envelope
/// and the body — while routing by kind.
pub(crate) enum WireRecordDto {
    /// A `Manifest`-kind line carrying the reference + immutable manifest body.
    Manifest {
        record: RecordIdDto,
        body: ManifestBodyDto,
    },
    /// A `Dose`-kind line carrying one cumulative dose observation body.
    Dose {
        record: RecordIdDto,
        body: ObservationBodyDto,
    },
}

impl WireRecordDto {
    /// Decode one NDJSON line, reading the record kind first and then the kind-matched body under the
    /// closed wire contract (unknown, duplicate, and missing fields all rejected on both the envelope
    /// and the body). On failure, the human-readable diagnostic is returned for
    /// [`parse_ndjson`](super::parse_ndjson::parse_ndjson) to tag with the line's position.
    ///
    /// The concrete dispatch mechanism — a closed derived envelope of the typed
    /// [`RecordIdDto`](super::record_id_dto::RecordIdDto) plus a borrowed
    /// `serde_json::value::RawValue` body, then a kind-dispatched typed decode of the retained raw body
    /// (preserving the body's duplicate/unknown-field detection, field-order-independent, no
    /// duplicate-erasing `serde_json::Value` staging) — is filled in the behavior phase, so this is a
    /// stub. The placeholder deliberately does not embed the untrusted line, so a reachable panic can
    /// never dump an entire evidence line.
    pub(crate) fn parse(line: &str) -> std::result::Result<Self, String> {
        let _ = line;
        todo!("kind-dispatch the wire line body under the closed wire contract")
    }

    /// This line's record identity (sequence and kind), regardless of body shape.
    pub(crate) fn record(&self) -> &RecordIdDto {
        match self {
            WireRecordDto::Manifest { record, .. } | WireRecordDto::Dose { record, .. } => record,
        }
    }
}
