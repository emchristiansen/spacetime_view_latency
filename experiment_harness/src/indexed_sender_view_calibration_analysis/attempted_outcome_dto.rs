//! The untrusted wire form of how an attempt that ran actually ended.

use serde::Deserialize;
use serde_json::value::RawValue;

use crate::indexed_sender_view_calibration_analysis::calibration_series_dto::CalibrationSeriesDto;

/// The wire form of
/// [`AttemptedOutcome`](crate::indexed_sender_view_calibration_pilot::attempted_outcome::AttemptedOutcome).
///
/// **Externally tagged, and that is proven rather than assumed.** The pilot's enum carries no
/// `#[serde(tag = …)]`, `untagged`, or `rename` attribute, so serde's default external tagging
/// applies: `CalibrationRecorded { series }` writes `{"CalibrationRecorded":{"series":{…}}}`.
///
/// The failure payload is retained verbatim rather than modelled. This analyzer never interprets a
/// failure — §569 evaluates `W` against **both complete retained series**, so a failed attempt is
/// inadmissible whatever it says — and mirroring the pilot's entire twelve-kind failure vocabulary
/// here would duplicate a contract that could then drift. Retaining it as a `RawValue` keeps
/// `deny_unknown_fields` total over the variant while decoding nothing this analyzer cannot use.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum AttemptedOutcomeDto {
    /// The attempt ran its paced batch to the frozen count and both caches validated.
    CalibrationRecorded { series: CalibrationSeriesDto },
    /// The attempt ran and did not record a complete, validated series.
    Failed {
        #[allow(dead_code)]
        failure: Box<RawValue>,
    },
}
