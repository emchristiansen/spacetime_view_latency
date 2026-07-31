//! A paced append batch that stopped early, with everything it had already measured.

use crate::client::measured_step_failure::MeasuredStepFailure;
use crate::observation::latency_sample::LatencySample;

/// A paced append batch's failure together with the ordered samples it completed before failing.
///
/// **The samples travel with the failure because they are evidence, not debris.** A paced append
/// batch is up to a thousand serial samples, so one that stops at sample 400 genuinely holds four
/// hundred completed issue-to-visible intervals — and those are informative about exactly the
/// stationarity, trend, and lag questions a calibration run exists to answer, up to where they stop.
/// Returning a bare error would discard most of what the attempt learned; the caller retains them as
/// non-evidence instead.
///
/// This is the structural difference from the cold-apply channel, whose measured unit is a single
/// interval and therefore has nothing to retain when it fails.
///
/// Every sample here is complete: the loop pushes only intervals that closed on their own row's
/// visibility, so a partial series is short, never padded and never approximate. The failing
/// sample contributes nothing but the classified cause.
pub(crate) struct PacedAppendFailure {
    /// The completed samples, in issue order, from before the batch stopped. May be empty when the
    /// very first append failed — which is "measured and got nothing", a different fact from never
    /// having reached the batch.
    pub(crate) samples: Vec<LatencySample>,
    /// Why the batch stopped, classified where the classification was still knowable.
    pub(crate) failure: MeasuredStepFailure,
}
