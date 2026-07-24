//! The whole-line wire envelope: a record identity plus its kind-dispatched body.

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::analysis::ingest::manifest_body_dto::ManifestBodyDto;
use crate::analysis::ingest::observation_body_dto::ObservationBodyDto;
use crate::analysis::ingest::record_id_dto::RecordIdDto;
use crate::analysis::ingest::record_kind_dto::RecordKindDto;

/// One decoded NDJSON line: the record identity and the body whose shape its
/// [`kind`](crate::analysis::ingest::record_kind_dto::RecordKindDto) selects.
///
/// The body's type is chosen by the record's *kind*, which is nested inside `record` rather than being
/// an external tag on the line, so this envelope cannot be a plain derived `Deserialize` (serde would
/// need the discriminant to be a sibling of the variant payloads). It is decoded by [`Self::parse`],
/// which reads the kind first and then decodes the body into exactly the matching DTO. That keeps the
/// closed-contract guarantees — unknown/duplicate/missing-field rejection on both the outer envelope
/// and the body — while routing by kind.
#[derive(Debug)]
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
    /// Decode one NDJSON line under the closed wire contract, returning the record identity and the
    /// body decoded into exactly the DTO its kind selects. On failure, serde's own message is returned
    /// for [`parse_ndjson`](super::parse_ndjson::parse_ndjson) to tag with the line's position. That
    /// message may quote the offending token — an unknown field name or variant tag, whose bytes do
    /// come from the line — plus a position, but this method never appends or interpolates the complete
    /// source line itself.
    ///
    /// Dispatch is two closed passes, not one derived enum (serde would need the kind discriminant to
    /// be a sibling of the variant payloads, but it is nested inside `record`). First the envelope is
    /// decoded into the typed [`RecordIdDto`] plus its body retained verbatim as a
    /// `Box<serde_json::value::RawValue>`, so the kind is read while the body is neither parsed nor
    /// staged through a duplicate-erasing `serde_json::Value`. Then the retained raw body is decoded
    /// into the kind-matched DTO. Both passes are `#[serde(deny_unknown_fields)]` derived struct
    /// visitors, so unknown, duplicate, and missing fields are rejected structurally on the envelope
    /// and the body alike, and both are field-order-independent — the body may precede `record` on the
    /// line, and the retained raw body is decoded only after the whole envelope (hence its kind) is read.
    pub(crate) fn parse(line: &str) -> std::result::Result<Self, String> {
        let Envelope { record, body } =
            serde_json::from_str(line).map_err(|error| error.to_string())?;
        match record.kind {
            RecordKindDto::Manifest => {
                let body = serde_json::from_str(body.get()).map_err(|error| error.to_string())?;
                Ok(WireRecordDto::Manifest { record, body })
            }
            RecordKindDto::Dose(_) => {
                let body = serde_json::from_str(body.get()).map_err(|error| error.to_string())?;
                Ok(WireRecordDto::Dose { record, body })
            }
        }
    }

    /// This line's record identity (sequence and kind), regardless of body shape.
    pub(crate) fn record(&self) -> &RecordIdDto {
        match self {
            WireRecordDto::Manifest { record, .. } | WireRecordDto::Dose { record, .. } => record,
        }
    }
}

/// The closed outer-envelope surface: the typed record identity and the body retained verbatim, so
/// the kind can be read without parsing or `serde_json::Value`-staging the body. `deny_unknown_fields`
/// plus serde's derived struct visitor reject unknown, duplicate, and missing envelope fields, and the
/// visitor is field-order-independent, so `body` may appear before `record` on the line.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    record: RecordIdDto,
    body: Box<RawValue>,
}

#[cfg(test)]
mod tests;
