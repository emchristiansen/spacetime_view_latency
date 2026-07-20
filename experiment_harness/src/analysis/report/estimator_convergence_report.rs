//! The report projection of the primary estimator's fixed convergence protocol.

use serde::Serialize;

use crate::params::REPETITION_BLOCKS;

/// The frozen 30-block count the estimator consumes, encoded in this marker's serialized tag. This
/// compile-time assertion ties that "thirty" to the single frozen [`REPETITION_BLOCKS`] source: if the
/// preregistered block count ever changed, this fails to compile, forcing the tag to be corrected rather
/// than silently drifting.
const _: () = assert!(
    REPETITION_BLOCKS == 30,
    "the estimator-convergence marker tag encodes a 30-block protocol; update it if REPETITION_BLOCKS changes"
);

/// The primary estimator's convergence protocol (spec: "report the primary estimator as exact,
/// fixed-30-block, non-iterative, and no-early-stop"). These are preregistered protocol facts of the exact
/// order-statistic Theil–Sen estimator, identical for every cell, so the report states them once at the
/// campaign root.
///
/// It is a zero-field closed marker whose *single* variant's serialized tag states the entire protocol, so
/// no impossible partial state (e.g. "iterative but no early stop", or a block count disagreeing with the
/// fixed sample) is representable — unlike independent boolean/count fields, which could encode a protocol
/// the estimator never runs. The "thirty" is pinned to [`REPETITION_BLOCKS`] by the compile-time assertion
/// above.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum EstimatorConvergenceReport {
    /// The exact, fixed 30-block, non-iterative, no-early-stop order-statistic estimator protocol.
    ExactFixedThirtyBlockNonIterativeNoEarlyStop,
}

impl EstimatorConvergenceReport {
    /// State the primary estimator's fixed convergence protocol — the sole marker variant. Takes no input:
    /// the protocol is a preregistered constant.
    pub(crate) fn preregistered() -> Self {
        Self::ExactFixedThirtyBlockNonIterativeNoEarlyStop
    }
}
