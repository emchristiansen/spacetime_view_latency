//! Why one measured step of a channel did not complete.

use anyhow::Error;

/// The classified cause of a measured step's failure, as the client can distinguish it.
///
/// The campaign's [`FailureKind`](crate::view_read_set_campaign::failure_kind::FailureKind) keeps
/// these apart so the retry rule cannot be relaxed by reclassification, and only the code that
/// issued the reducer and owned the deadline knows which happened; recovering it from `anyhow`
/// context downstream would be guessing.
///
/// Three, not the campaign's five: a nonpositive statistic is refused later at the channel's own
/// reduction, and a semantics refusal is a judgement over retained rows. Neither is a step outcome.
/// Lifecycle position is the driver's to add, since one primitive serves more than one channel.
pub(crate) enum MeasuredStepFailure {
    /// The transport, the SDK, or the harness's own measurement path failed.
    Infrastructure(Error),
    /// A measured write's reducer returned an error, or the SDK reported an internal error for it.
    /// Reachable only where a reducer runs.
    Application(Error),
    /// A confirmation, an applied snapshot, or a row's visibility did not arrive within its bound.
    Timeout(Error),
}

impl MeasuredStepFailure {
    /// The underlying cause, for a caller outside any measured window — the historical Pilot's
    /// `connect`, which has no channel to classify for.
    pub(crate) fn into_error(self) -> Error {
        match self {
            Self::Infrastructure(error) | Self::Application(error) | Self::Timeout(error) => error,
        }
    }
}
