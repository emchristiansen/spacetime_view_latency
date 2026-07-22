//! Analyze's corrected-v2 bundle discoverable under its final name, parent-fsync durability pending.

use crate::analysis::publish::corrected_v2_paths::CorrectedV2Paths;
use crate::analysis::publish::corrected_v2_staging::CorrectedV2Staging;
use crate::analysis::publish::promotion_failure::PromotionFailure;

/// The corrected-v2 bundle has been atomically renamed to its deterministic final directory — the
/// discoverability commit point has passed — but the parent-directory fsync that makes that rename
/// crash-durable has not yet completed. [`Self::new`] is private to this module and called only from
/// [`CorrectedV2Staging::promote`] below — the sole place a `RenameCommitted` is ever constructed,
/// mirroring `HarnessCommit`'s private-constructor-plus-colocated-mint pattern — so nothing can forge a
/// `RenameCommitted` without having genuinely staged and renamed a bundle. A parent-fsync failure from
/// here is [`IndeterminatePublication`](super::indeterminate_publication::IndeterminatePublication), never
/// a rollback or a replacement; publishing durably is `Self::fsync_parent`, implemented in
/// `published_corrected_v2.rs` for the same colocation reason.
pub(crate) struct RenameCommitted {
    paths: CorrectedV2Paths,
}

impl RenameCommitted {
    fn new(paths: CorrectedV2Paths) -> Self {
        Self { paths }
    }

    /// Consumes `self` to hand its paths to the publish step colocated in `published_corrected_v2.rs`.
    pub(in crate::analysis::publish) fn into_paths(self) -> CorrectedV2Paths {
        self.paths
    }
}

impl CorrectedV2Staging {
    /// Refuse to replace an existing final bundle, then atomically rename the complete staging directory
    /// to its deterministic final name on the same filesystem — the discoverability commit point.
    /// Colocated with [`RenameCommitted`] (rather than with the `CorrectedV2Staging` struct declaration)
    /// so this is the only code in the crate with module access to `RenameCommitted::new`.
    pub(crate) fn promote(self) -> Result<RenameCommitted, PromotionFailure> {
        let paths = self.into_paths();
        let final_dir = paths.final_dir();
        let staging_dir = paths.staging_dir();
        todo!(
            "Phase 2: refuse if {final_dir:?} already exists, else atomically rename {staging_dir:?} to \
             it, then RenameCommitted::new(paths) on success"
        )
    }
}
