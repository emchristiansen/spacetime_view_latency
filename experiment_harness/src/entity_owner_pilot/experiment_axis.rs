//! Which frozen finite range an attempt varies.

use serde::Serialize;

/// The closed set of axes an attempt can walk — one variant per frozen finite range in the spec's
/// "Experiment protocol".
///
/// The spec's "Minimal type design" names the `axis: ExperimentAxis` field of
/// [`AttemptKey`](super::attempt_key::AttemptKey) but does not enumerate its variants. These four
/// are transcribed one-for-one from the spec's own frozen ranges, which is the only enumeration the
/// spec actually fixes:
///
/// - "unrelated/global rows: `1k, 2k, 4k, 8k, 16k, 32k`" → [`Self::UnrelatedGlobalRows`]
/// - "subscriber-visible rows: `10, 100, 1k, 10k`" → [`Self::SubscriberVisibleRows`]
/// - "active exact keys/subscriptions: `1, 2, 4, 8, 16, 32`" → [`Self::ActiveExactKeys`]
/// - "synthetic group-member/fan-out guard: low `1`, high `32`" → [`Self::SyntheticGroupFanOut`]
///
/// Deriving the variants from the frozen ranges rather than inventing them keeps the axis vocabulary
/// and the preregistered ladders single-sourced: an axis that no frozen range defines cannot be
/// named, and a frozen range with no axis cannot be walked.
///
/// The `EntityOwnerSenderView` Pilot is single-axis — [`Self::UnrelatedGlobalRows`] only, per the
/// Parked Frontier's `N_global` ladder. The other three are declared and unused until their
/// milestones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ExperimentAxis {
    /// Unrelated/global rows held by an identity other than the measured subscriber — the rows that
    /// grow the backing table without growing the measured result set.
    UnrelatedGlobalRows,
    /// Rows visible to the measured subscriber, where an own slice can accumulate.
    SubscriberVisibleRows,
    /// Active exact keys or per-key subscriptions held concurrently.
    ActiveExactKeys,
    /// Synthetic group-member/fan-out guard. The spec requires this be labeled synthetic rather than
    /// production-representative wherever it is reported.
    SyntheticGroupFanOut,
}
