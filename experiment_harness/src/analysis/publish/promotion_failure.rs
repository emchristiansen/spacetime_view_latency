//! Why Analyze's atomic corrected-v2 rename did not commit.

/// The exhaustive typed reason Analyze's atomic staging-to-final rename did not commit. This is a
/// pre-rename failure, so no final bundle is discoverable under the deterministic final directory name
/// (spec: "failure before it leaves no final bundle").
#[derive(Debug)]
pub(crate) enum PromotionFailure {
    /// The deterministic final directory already exists; Analyze refuses to replace an existing final
    /// bundle rather than renaming over it (spec: "refuse to replace an existing final bundle").
    FinalBundleAlreadyExists { diagnostic: String },
    /// The atomic rename call itself failed for a reason other than a pre-existing final bundle.
    RenameFailed { diagnostic: String },
}

impl PromotionFailure {
    pub(crate) fn final_bundle_already_exists(diagnostic: String) -> Self {
        Self::FinalBundleAlreadyExists { diagnostic }
    }

    pub(crate) fn rename_failed(diagnostic: String) -> Self {
        Self::RenameFailed { diagnostic }
    }

    /// The human-readable diagnostic for the failure.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            Self::FinalBundleAlreadyExists { diagnostic } | Self::RenameFailed { diagnostic } => {
                diagnostic
            }
        }
    }
}
