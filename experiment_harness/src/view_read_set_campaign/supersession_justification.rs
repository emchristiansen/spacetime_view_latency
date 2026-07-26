//! The stated reason a method invalidation was recorded.

use anyhow::{ensure, Result};
use serde::Serialize;

/// Nonempty free text explaining why evidence was superseded.
///
/// **Who can construct it.** Its field is private and [`Self::parsed`] is the only constructor,
/// declared in this file, so no other module — sibling, parent, or elsewhere in the crate — can mint
/// one. Parse-don't-validate: past this door the value *is* a justification, and no consumer needs a
/// nonemptiness check of its own.
///
/// **What it rules out.** A supersession recorded with no reason. An invalidation removes evidence
/// from consideration, so one that states nothing leaves a reader with a deletion they cannot
/// dispute — the failure mode a plain `String` field admits, since `""` and `"   "` are both
/// well-typed. Emptiness is judged *after* trimming, because whitespace is not a reason.
///
/// **What is stored is the trimmed text**, not the input. Normalizing rather than preserving is
/// deliberate: the recorded value is then exactly the text that was validated, so a reader cannot be
/// shown padding that the check ignored. Interior whitespace is untouched — only the ends are.
///
/// **What it does not claim.** That the reason is true, sufficient, or authored by anyone entitled
/// to record it. Those are outside what any constructor can establish, and
/// [`MethodSupersession`](super::method_supersession::MethodSupersession) disclaims them explicitly.
/// This type establishes exactly one thing: that a reason was given.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(crate) struct SupersessionJustification {
    text: String,
}

impl SupersessionJustification {
    /// Parse stated free text into a justification, failing loud when it is empty after trimming.
    pub(crate) fn parsed(text: &str) -> Result<Self> {
        let trimmed = text.trim();
        ensure!(
            !trimmed.is_empty(),
            "a method supersession must state a reason; the justification was empty after trimming",
        );
        Ok(Self {
            text: trimmed.to_string(),
        })
    }

    /// The recorded reason, trimmed.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}
