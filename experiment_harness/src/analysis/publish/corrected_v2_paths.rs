//! Deterministic filesystem paths for Analyze's corrected-v2 staging and final bundle.

use std::path::PathBuf;

use crate::campaign::CampaignRoot;
use crate::params::{
    CORRECTED_V2_DIRNAME, CORRECTED_V2_NDJSON_FILENAME, CORRECTED_V2_REPORT_FILENAME,
    CORRECTED_V2_STAGING_SUFFIX, SHA256_SIDECAR_SUFFIX,
};

/// The deterministic path family Analyze stages under and finally publishes to, all purely derived from
/// the shared `--campaign-root`. Single-sources every filename and directory-naming rule (the
/// [`CORRECTED_V2_STAGING_SUFFIX`] sibling-directory rule and the [`SHA256_SIDECAR_SUFFIX`] digest
/// suffix), so no call site can derive a path or a sidecar name that drifts out of sync with its
/// artifact.
#[derive(Debug, Clone)]
pub(crate) struct CorrectedV2Paths {
    campaign_root: CampaignRoot,
}

impl CorrectedV2Paths {
    pub(crate) fn under(campaign_root: CampaignRoot) -> Self {
        Self { campaign_root }
    }

    /// `<campaign-root>/campaign-seed20260717-corrected-v2.staging/` — Analyze's exclusively-created,
    /// not-yet-discoverable staging directory (spec: "exclusively creates deterministic
    /// `<campaign-root>/campaign-seed20260717-corrected-v2.staging/`").
    pub(crate) fn staging_dir(&self) -> PathBuf {
        self.campaign_root
            .path()
            .join(format!("{CORRECTED_V2_DIRNAME}{CORRECTED_V2_STAGING_SUFFIX}"))
    }

    /// `<campaign-root>/campaign-seed20260717-corrected-v2/` — the deterministic final bundle directory
    /// the staging directory is atomically renamed to.
    pub(crate) fn final_dir(&self) -> PathBuf {
        self.campaign_root.path().join(CORRECTED_V2_DIRNAME)
    }

    pub(crate) fn staged_ndjson(&self) -> PathBuf {
        self.staging_dir().join(CORRECTED_V2_NDJSON_FILENAME)
    }

    pub(crate) fn staged_ndjson_sha256(&self) -> PathBuf {
        self.staging_dir()
            .join(format!("{CORRECTED_V2_NDJSON_FILENAME}{SHA256_SIDECAR_SUFFIX}"))
    }

    pub(crate) fn staged_report(&self) -> PathBuf {
        self.staging_dir().join(CORRECTED_V2_REPORT_FILENAME)
    }

    pub(crate) fn staged_report_sha256(&self) -> PathBuf {
        self.staging_dir()
            .join(format!("{CORRECTED_V2_REPORT_FILENAME}{SHA256_SIDECAR_SUFFIX}"))
    }

    /// The published corrected-v2 NDJSON's path once the staging directory has been renamed to
    /// [`Self::final_dir`] — identical filename to [`Self::staged_ndjson`] (spec: "identical inside
    /// Analyze's staging directory and in the final published bundle").
    pub(crate) fn final_ndjson(&self) -> PathBuf {
        self.final_dir().join(CORRECTED_V2_NDJSON_FILENAME)
    }

    pub(crate) fn final_ndjson_sha256(&self) -> PathBuf {
        self.final_dir()
            .join(format!("{CORRECTED_V2_NDJSON_FILENAME}{SHA256_SIDECAR_SUFFIX}"))
    }

    /// The published report JSON's path once the staging directory has been renamed to
    /// [`Self::final_dir`] — identical filename to [`Self::staged_report`].
    pub(crate) fn final_report(&self) -> PathBuf {
        self.final_dir().join(CORRECTED_V2_REPORT_FILENAME)
    }

    pub(crate) fn final_report_sha256(&self) -> PathBuf {
        self.final_dir()
            .join(format!("{CORRECTED_V2_REPORT_FILENAME}{SHA256_SIDECAR_SUFFIX}"))
    }
}
