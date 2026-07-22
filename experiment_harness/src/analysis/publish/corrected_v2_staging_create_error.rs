//! Why Analyze's exclusive corrected-v2 staging-directory creation did not succeed.

/// The exhaustive typed reason Analyze's exclusive corrected-v2 staging-directory creation failed.
/// Every variant means no final bundle is discoverable under the deterministic final directory name by
/// this call (spec: "failure before it leaves no final bundle") — it does *not* mean the staging
/// directory is clean: a partial-write/partial-fsync failure can leave non-authoritative staging residue
/// behind. Retry after either variant requires removing or renaming the stale staging directory out of
/// band; the harness never deletes, resumes, overwrites, or reuses it (spec: "retry after a failed
/// pre-rename analysis requires removing or renaming the stale staging directory out of band").
#[derive(Debug)]
pub(crate) enum CorrectedV2StagingCreateError {
    /// The deterministic staging directory already exists; Analyze refuses to reuse, resume, or
    /// overwrite stale staging state (spec: "`Analyze` refuses to start if that staging directory
    /// exists").
    StagingDirectoryAlreadyExists { diagnostic: String },
    /// The staging directory's exclusive creation, one of its four required files (the corrected-v2
    /// NDJSON, the report JSON, and their two `.sha256` sidecars), or a required fsync failed for a
    /// reason other than a pre-existing staging directory. The staging directory may now contain partial,
    /// non-authoritative residue.
    StagingIncomplete { diagnostic: String },
}

impl CorrectedV2StagingCreateError {
    pub(crate) fn staging_directory_already_exists(diagnostic: String) -> Self {
        Self::StagingDirectoryAlreadyExists { diagnostic }
    }

    pub(crate) fn staging_incomplete(diagnostic: String) -> Self {
        Self::StagingIncomplete { diagnostic }
    }

    /// The human-readable diagnostic for the failure.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            Self::StagingDirectoryAlreadyExists { diagnostic } | Self::StagingIncomplete { diagnostic } => {
                diagnostic
            }
        }
    }
}
