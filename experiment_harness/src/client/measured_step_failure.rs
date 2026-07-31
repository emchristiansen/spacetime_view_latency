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

    /// Add `context` to the cause, keeping the classification.
    ///
    /// The reason this exists rather than callers using [`anyhow::Context`]: that trait's methods
    /// are reached through `Result` and yield an `Error`, so adding the entity a write names would
    /// discard exactly the distinction this type carries. Every variant is mapped in turn, so a new
    /// one cannot be silently reclassified here.
    pub(crate) fn context<C: std::fmt::Display + Send + Sync + 'static>(self, context: C) -> Self {
        match self {
            Self::Infrastructure(error) => Self::Infrastructure(error.context(context)),
            Self::Application(error) => Self::Application(error.context(context)),
            Self::Timeout(error) => Self::Timeout(error.context(context)),
        }
    }
}
