//! Which record shape a failed attempt's evidence can honestly support.

use anyhow::{ensure, Result};
use serde::Serialize;

use crate::indexed_sender_view_calibration_pilot::attempt_failure::AttemptFailure;
use crate::indexed_sender_view_calibration_pilot::failure_kind::FailureKind;

/// The four closed stages a failed attempt can terminate in, named for the evidence that exists
/// rather than for the instant the failure occurred.
///
/// Each [`FailureKind`] maps to exactly one stage, and each stage to exactly one
/// [`CalibrationRecord`](super::calibration_record::CalibrationRecord) shape. That pair of total
/// functions is what makes "no failure kind may inhabit two shapes" a checked property rather than a
/// convention, and it is why the record constructors validate against a stage instead of enumerating
/// kinds themselves.
///
/// **Deliberately not a timeline.** `PacedBatch` is `Bracketed` even though it strikes before the
/// `after` observation is taken, because in that case the driver still closed the bracket afterwards
/// and the record ends up holding both observations. Where that same observation *also* failed, the
/// failure is `HostObservationAfterBatchFailure` and lands in `Unbracketed` instead — same instant,
/// different stage, because the stage names the evidence the record carries rather than the
/// chronological position of the failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum AttemptStage {
    /// No instance exists. Only the monotone partial-provisioning prefix is available.
    Unprovisioned,
    /// An instance was published and is fully identified, but no measurement window opened, so there
    /// is no host observation.
    Unmeasured,
    /// The `after` observation failed, so the bracket could not be closed and only the `before`
    /// observation survives.
    Unbracketed,
    /// The window opened and closed with both host observations.
    Bracketed,
}

impl AttemptStage {
    /// Every stage, for exhaustive checks.
    pub(crate) const ALL: [AttemptStage; 4] = [
        AttemptStage::Unprovisioned,
        AttemptStage::Unmeasured,
        AttemptStage::Unbracketed,
        AttemptStage::Bracketed,
    ];

    /// Whether a failure of `kind` belongs in this stage's record shape.
    ///
    /// Exactly one stage accepts any given kind, because [`FailureKind::stage`] is a total function
    /// into this enum.
    pub(crate) fn accepts(self, kind: FailureKind) -> bool {
        kind.stage() == self
    }

    /// Fail loud unless `failure` belongs in this stage's record shape.
    ///
    /// The single validation every failure-bearing record constructor runs, so a shape can never be
    /// built around a failure whose evidence it does not actually hold.
    pub(crate) fn ensure_admits(self, failure: &AttemptFailure) -> Result<()> {
        let kind = failure.kind();
        ensure!(
            self.accepts(kind),
            "{kind:?} is a {:?} failure and cannot be recorded in the {self:?} record shape",
            kind.stage(),
        );
        Ok(())
    }
}
