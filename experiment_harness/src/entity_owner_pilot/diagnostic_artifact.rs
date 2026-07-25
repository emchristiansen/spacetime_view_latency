//! The retained diagnostic text of a failed attempt.

use serde::Serialize;

/// The full diagnostic chain retained for a failed attempt.
///
/// The spec requires a failed attempt to be recorded "with partial evidence and diagnostics", and
/// the repo's error policy is to fail loud with the whole `anyhow` context chain rather than a
/// summarized message. [`Self::of_error`] therefore formats with `{:#}` to retain every context
/// layer — the difference between "measuring rung 3" and "measuring rung 3: waiting for a confirmed
/// measured-batch callback: timed out" is the entire diagnostic value.
///
/// This is deliberately opaque free text: the machine-readable classification of *what kind* of
/// failure occurred lives in [`FailureKind`](super::failure_kind::FailureKind), so a reader never
/// has to parse this string to route or count failures.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct DiagnosticArtifact(String);

impl DiagnosticArtifact {
    /// Retain an error's complete context chain.
    pub(crate) fn of_error(error: &anyhow::Error) -> Self {
        Self(format!("{error:#}"))
    }

    /// The retained diagnostic text.
    pub(crate) fn get(&self) -> &str {
        &self.0
    }
}
