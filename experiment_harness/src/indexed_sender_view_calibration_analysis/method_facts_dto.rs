//! The untrusted wire form of the frozen method every record restates.

use serde::Deserialize;

use crate::indexed_sender_view_calibration_analysis::measurement_channel_dto::MeasurementChannelDto;
use crate::indexed_sender_view_calibration_analysis::outcome_ceiling_dto::OutcomeCeilingDto;
use crate::indexed_sender_view_calibration_pilot::calibration_params::{
    MAX_PACED_SAMPLES, SUBSCRIBER_OWN_ROWS, WITH_CONFIRMED_READS,
};
use crate::view_read_set_campaign::campaign_params::PACED_SAMPLE_DELAY_MS;

/// The wire form of
/// [`MethodFacts`](crate::indexed_sender_view_calibration_pilot::method_facts::MethodFacts), rechecked
/// on **every** line rather than once per file.
///
/// The pilot carries this on every record — including a slot that never ran — precisely so a reader
/// holding only the ledger can confirm what was frozen. Checking it once and trusting the rest would
/// throw that away: a ledger assembled from two runs under different constants would pass a
/// first-line check and then contribute a series measured under a method §569 was not stated over.
///
/// **The expected values are read from the pilot's own frozen constants, never restated as
/// literals.** Restating `1_000` or `10` here would create a second copy of one truth, free to drift
/// from the constant the run actually used — the same duplication the pilot's own
/// `WITH_CONFIRMED_READS` was corrected to remove. Reading a `const` adds no accessor to any sealed
/// pilot type.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MethodFactsDto {
    pub(crate) channel: MeasurementChannelDto,
    pub(crate) sample_count: u32,
    pub(crate) paced_sample_delay_ms: u64,
    pub(crate) seeded_own_rows: u64,
    pub(crate) with_confirmed_reads: bool,
    pub(crate) outcome_ceiling: OutcomeCeilingDto,
}

impl MethodFactsDto {
    /// Whether this line was written under exactly the method §569's decision rule is stated over.
    ///
    /// **What the exhaustiveness here does and does not buy, stated exactly.** The comparison
    /// destructures `Self`, so every field *this DTO currently decodes* must participate: adding a
    /// field to this struct without deciding what it means is a compile error.
    ///
    /// It does **not** protect against the pilot growing a method fact. That crosses a serde
    /// boundary, and no compiler sees both sides. What happens instead is a *runtime* refusal: this
    /// DTO is `deny_unknown_fields`, so an unrecognised field makes the whole line fail to decode as
    /// a malformed-line ingest error. That is a loud failure rather than a silent one — an analyzer
    /// built before the change cannot quietly analyse evidence written after it — but it is not a
    /// build-time guarantee, and must not be described as one.
    pub(crate) fn matches_frozen(self) -> bool {
        todo!("frozen-method equality against the pilot's own constants")
    }
}
