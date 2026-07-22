//! Analyze's corrected-v2 bundle staged but not yet durably published.

use crate::analysis::publish::corrected_v2_paths::CorrectedV2Paths;
use crate::analysis::publish::corrected_v2_staging_create_error::CorrectedV2StagingCreateError;

/// The exclusively-created, fully-written-and-fsynced corrected-v2 staging directory
/// (`<campaign-root>/campaign-seed20260717-corrected-v2.staging/`), not yet discoverable under its final
/// deterministic name. Its field is private to this module, so no sibling module can fabricate a
/// `CorrectedV2Staging` by struct literal — the only way to obtain one is [`Self::create`]. Promoting one
/// to [`RenameCommitted`](super::rename_committed::RenameCommitted) is `Self::promote`, implemented in
/// `rename_committed.rs` (not here) so only that module has access to `RenameCommitted`'s private
/// constructor — see that file's module doc.
pub(crate) struct CorrectedV2Staging {
    paths: CorrectedV2Paths,
}

impl CorrectedV2Staging {
    /// Exclusively create the staging directory and durably write its four required files (corrected-v2
    /// NDJSON, report JSON, and both `.sha256` sidecars digesting those same written bytes), fsyncing
    /// every staged file and the staging directory itself. `corrected_ndjson` and `report_json` are the
    /// exact bytes to write and digest — explicit content inputs, not a hidden reread — so this signature
    /// names every dependency Phase 2's filesystem/digest/fsync body needs. Phase 1 wires only the
    /// deterministic path derivation and this typed pre-rename failure boundary.
    pub(crate) fn create(
        paths: CorrectedV2Paths,
        corrected_ndjson: &[u8],
        report_json: &[u8],
    ) -> Result<Self, CorrectedV2StagingCreateError> {
        todo!(
            "Phase 2: exclusively create {:?}; write+fsync {:?} ({} corrected-v2 NDJSON bytes) and its \
             {:?} sidecar; write+fsync {:?} ({} report JSON bytes) and its {:?} sidecar; fsync the \
             staging directory itself",
            paths.staging_dir(),
            paths.staged_ndjson(),
            corrected_ndjson.len(),
            paths.staged_ndjson_sha256(),
            paths.staged_report(),
            report_json.len(),
            paths.staged_report_sha256(),
        )
    }

    /// Consumes `self` to hand its paths to the promotion step colocated in `rename_committed.rs`.
    pub(in crate::analysis::publish) fn into_paths(self) -> CorrectedV2Paths {
        self.paths
    }
}
