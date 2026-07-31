//! The retained diagnostic text of a failed attempt.

use serde::Serialize;

/// The full diagnostic chain retained for a failed attempt.
///
/// [`Self::of_error`] formats with `{:#}` to retain every `anyhow` context layer — the difference
/// between "measuring rung 3" and "measuring rung 3: waiting for a confirmed measured-batch
/// callback: timed out" is the entire diagnostic value.
///
/// Deliberately opaque free text; the machine-readable classification lives in
/// [`FailureKind`](super::failure_kind::FailureKind), so nothing has to parse this string to route
/// or count failures.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct DiagnosticArtifact(String);

impl DiagnosticArtifact {
    /// Retain an error's complete context chain.
    pub(crate) fn of_error(error: &anyhow::Error) -> Self {
        Self(format!("{error:#}"))
    }
}
