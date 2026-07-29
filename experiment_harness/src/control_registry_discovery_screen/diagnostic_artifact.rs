//! The retained diagnostic for a failed attempt.

use serde::Serialize;

/// The full error chain of a failed attempt, retained verbatim.
///
/// A newtype rather than a bare `String` so a diagnostic cannot be confused with, or silently
/// substituted for, evidence: the type system distinguishes "why this attempt produced nothing" from
/// "what this attempt measured". Built only from a real error via [`Self::of_error`], so a
/// diagnostic can never be a hand-written narrative of what someone believed went wrong.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub(crate) struct DiagnosticArtifact(String);

impl DiagnosticArtifact {
    /// Capture an error's full context chain.
    pub(crate) fn of_error(error: &anyhow::Error) -> Self {
        Self(format!("{error:#}"))
    }
}
