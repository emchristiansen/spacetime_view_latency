//! The untrusted wire form of one calibration ledger line.

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::indexed_sender_view_calibration_analysis::attempt_key_dto::AttemptKeyDto;
use crate::indexed_sender_view_calibration_analysis::attempted_outcome_dto::AttemptedOutcomeDto;
use crate::indexed_sender_view_calibration_analysis::method_facts_dto::MethodFactsDto;

/// The wire form of
/// [`CalibrationRecord`](crate::indexed_sender_view_calibration_pilot::calibration_record::CalibrationRecord)
/// — a five-variant mirror of the pilot's five terminal record shapes.
///
/// **The line shape is proven from the writer.** `append_all` in the pilot's driver writes exactly
/// `serde_json::to_string(record)` followed by a newline, and `CalibrationRecord` is an
/// attribute-free `#[derive(Serialize)]` enum, so each line is one externally tagged JSON object:
/// `{"Attempted":{"key":…,"method":…,…}}`. There is no envelope, no record-kind discriminant beside
/// the payload, and no rename anywhere in the chain — which is why this can be a plain derived
/// `Deserialize`, unlike the campaign's
/// [`WireRecordDto`](crate::analysis::ingest::wire_record_dto::WireRecordDto), whose kind is nested
/// inside its `record` field.
///
/// All five variants are mirrored even though only `Attempted` can carry a series. A ledger holding
/// four `NotRun` lines and one `Attempted` must **parse** so the analyzer can say that only one
/// replicate is present; refusing to decode the other four would report a malformed ledger for a run
/// that recorded its outcome honestly.
///
/// Fields this analyzer does not interpret are retained verbatim as `RawValue` rather than modelled.
/// `deny_unknown_fields` stays total over every variant, so the pilot cannot add a field without this
/// type being updated, but the twelve-kind failure vocabulary, the provision provenance, the host
/// observations, and the release disposition are not duplicated here — a second copy of those
/// contracts could drift from the one the pilot actually writes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum CalibrationRecordDto {
    /// The slot never entered acquisition.
    NotRun {
        key: AttemptKeyDto,
        method: MethodFactsDto,
        #[allow(dead_code)]
        pinned: Box<RawValue>,
        #[allow(dead_code)]
        schedule_seed: u64,
        #[allow(dead_code)]
        reason: Box<RawValue>,
    },
    /// Acquisition began and failed before any instance existed.
    NotProvisioned {
        key: AttemptKeyDto,
        method: MethodFactsDto,
        #[allow(dead_code)]
        pinned: Box<RawValue>,
        #[allow(dead_code)]
        schedule_seed: u64,
        #[allow(dead_code)]
        partial_provision: Box<RawValue>,
        #[allow(dead_code)]
        failure: Box<RawValue>,
        #[allow(dead_code)]
        release: Box<RawValue>,
    },
    /// The instance was published, but the measurement window never opened.
    NotMeasured {
        key: AttemptKeyDto,
        method: MethodFactsDto,
        #[allow(dead_code)]
        pinned: Box<RawValue>,
        #[allow(dead_code)]
        schedule_seed: u64,
        #[allow(dead_code)]
        provenance: Box<RawValue>,
        #[allow(dead_code)]
        failure: Box<RawValue>,
        #[allow(dead_code)]
        release: Box<RawValue>,
    },
    /// The `after` observation failed, so the bracket could not be closed.
    Unbracketed {
        key: AttemptKeyDto,
        method: MethodFactsDto,
        #[allow(dead_code)]
        pinned: Box<RawValue>,
        #[allow(dead_code)]
        schedule_seed: u64,
        #[allow(dead_code)]
        provenance: Box<RawValue>,
        #[allow(dead_code)]
        before: Box<RawValue>,
        #[allow(dead_code)]
        failure: Box<RawValue>,
        #[allow(dead_code)]
        release: Box<RawValue>,
    },
    /// The measurement window opened and closed with both host observations. The only shape that can
    /// hold a recorded series.
    Attempted {
        key: AttemptKeyDto,
        method: MethodFactsDto,
        #[allow(dead_code)]
        pinned: Box<RawValue>,
        #[allow(dead_code)]
        schedule_seed: u64,
        #[allow(dead_code)]
        provenance: Box<RawValue>,
        #[allow(dead_code)]
        host: Box<RawValue>,
        outcome: AttemptedOutcomeDto,
        #[allow(dead_code)]
        release: Box<RawValue>,
    },
}

impl CalibrationRecordDto {
    /// This line's attempt identity, whatever shape the record has.
    pub(crate) fn key(&self) -> &AttemptKeyDto {
        match self {
            CalibrationRecordDto::NotRun { key, .. }
            | CalibrationRecordDto::NotProvisioned { key, .. }
            | CalibrationRecordDto::NotMeasured { key, .. }
            | CalibrationRecordDto::Unbracketed { key, .. }
            | CalibrationRecordDto::Attempted { key, .. } => key,
        }
    }

    /// The frozen method this line restates, whatever shape the record has.
    pub(crate) fn method(&self) -> MethodFactsDto {
        match self {
            CalibrationRecordDto::NotRun { method, .. }
            | CalibrationRecordDto::NotProvisioned { method, .. }
            | CalibrationRecordDto::NotMeasured { method, .. }
            | CalibrationRecordDto::Unbracketed { method, .. }
            | CalibrationRecordDto::Attempted { method, .. } => *method,
        }
    }
}
