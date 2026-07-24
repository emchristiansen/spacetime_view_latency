//! The durably published, discoverable corrected-v2 bundle.

use crate::analysis::publish::corrected_v2_paths::CorrectedV2Paths;
use crate::analysis::publish::indeterminate_publication::IndeterminatePublication;
use crate::analysis::publish::rename_committed::RenameCommitted;

/// Evidence that the corrected-v2 bundle is discoverable under its deterministic final name *and* that
/// discoverability is crash-durable — its parent directory has been fsynced after the rename that created
/// it. [`Self::new`] is private to this module and called only from [`RenameCommitted::fsync_parent`]
/// below — the sole place a `PublishedCorrectedV2` is ever constructed — so nothing can forge one without
/// a genuine rename-committed bundle and a successful parent fsync.
pub(crate) struct PublishedCorrectedV2 {
    paths: CorrectedV2Paths,
}

impl PublishedCorrectedV2 {
    fn new(paths: CorrectedV2Paths) -> Self {
        Self { paths }
    }

    /// The published bundle's deterministic paths, for `Analyze`'s post-publish stdout report.
    pub(crate) fn paths(&self) -> &CorrectedV2Paths {
        &self.paths
    }
}

impl RenameCommitted {
    /// Fsync the campaign root (the renamed bundle's parent directory) so the rename's durability is no
    /// longer indeterminate. Never rolls back or replaces the already-visible final bundle on failure.
    /// Colocated with [`PublishedCorrectedV2`] (rather than with the `RenameCommitted` struct declaration)
    /// so this is the only code in the crate with module access to `PublishedCorrectedV2::new`.
    pub(crate) fn fsync_parent(self) -> Result<PublishedCorrectedV2, IndeterminatePublication> {
        let paths = self.into_paths();
        let final_dir = paths.final_dir();
        todo!(
            "Phase 2: fsync the parent directory of {final_dir:?}, committing durability of the \
             completed rename, then PublishedCorrectedV2::new(paths) on success"
        )
    }
}
