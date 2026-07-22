//! The one indeterminate outcome: parent-directory fsync failed after the corrected-v2 rename committed.

use std::path::{Path, PathBuf};

/// The corrected-v2 bundle's atomic rename succeeded — it may already be visible under its final
/// deterministic name — but the subsequent parent-directory fsync failed, so whether that visibility is
/// crash-durable is unknown. There is no rollback, no replacement, and no success output for this outcome
/// (spec: "parent-fsync failure after [rename] is an indeterminate committed outcome that may leave the
/// complete final bundle visible, must never trigger rollback or replacement, and emits no success
/// output").
#[derive(Debug)]
pub(crate) struct IndeterminatePublication {
    final_dir: PathBuf,
    diagnostic: String,
}

impl IndeterminatePublication {
    pub(crate) fn new(final_dir: PathBuf, diagnostic: String) -> Self {
        Self {
            final_dir,
            diagnostic,
        }
    }

    /// The deterministic final directory whose crash-durability is unknown.
    pub(crate) fn final_dir(&self) -> &Path {
        &self.final_dir
    }

    /// The human-readable diagnostic for the parent-fsync failure.
    pub(crate) fn diagnostic(&self) -> &str {
        &self.diagnostic
    }
}
