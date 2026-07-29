//! Which side of the spec's retry rule a failure falls on.

use serde::Serialize;

/// Whether a failure was infrastructural or an application/semantic result.
///
/// The spec draws retry eligibility along exactly this line: a *prospective* infrastructure failure
/// is an interruption of measurement, while a semantic failure, security failure, application error,
/// timeout, or nonpositive statistic is a measurement outcome. Retrying the latter would be
/// re-rolling a result until it came out favourably.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum FailurePhase {
    /// The environment or plumbing failed, independently of what was being measured. The only
    /// phase a retry can be earned from, and only before the first measured sample.
    Infrastructure,
    /// The system under test produced this outcome. Never retryable, whenever it occurred.
    ApplicationOrSemantic,
    /// The harness could not read the host observations the frozen method requires around a
    /// measurement.
    ///
    /// Its own phase because it is neither: `/proc` becoming unreadable is not the system under
    /// test producing a result, but neither is it a prospective interruption that a retry may
    /// silently replace. The spec makes both host-observation failures non-retryable, and giving
    /// them a distinct phase records that as a fact about the failure rather than mislabelling a
    /// harness fault as an application one to reach the same eligibility.
    HarnessObservation,
}
