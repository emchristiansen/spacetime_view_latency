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
    /// The environment or plumbing failed, independently of what was being measured.
    Infrastructure,
    /// The system under test produced this outcome. Never retryable, whenever it occurred.
    ApplicationOrSemantic,
}
