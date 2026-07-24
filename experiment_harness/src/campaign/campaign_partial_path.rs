//! The deterministic path of Campaign's one fixed, non-authoritative staged artifact.

use std::path::{Path, PathBuf};

use crate::campaign::campaign_root::CampaignRoot;
use crate::params::{CAMPAIGN_STAGING_DIRNAME, PARTIAL_CAMPAIGN_FILENAME};

/// `<campaign-root>/staging/campaign-seed20260717.ndjson.partial` — Campaign's one fixed,
/// non-authoritative staged output path, purely derived from `--campaign-root` (spec: "Campaign ...
/// exclusively creates `<campaign-root>/staging/campaign-seed20260717.ndjson.partial`, failing if it
/// already exists"). Campaign owns only this path; it never creates or touches the corrected-v2 bundle.
#[derive(Debug, Clone)]
pub(crate) struct CampaignPartialPath(PathBuf);

impl CampaignPartialPath {
    pub(crate) fn under(root: &CampaignRoot) -> Self {
        Self(
            root.path()
                .join(CAMPAIGN_STAGING_DIRNAME)
                .join(PARTIAL_CAMPAIGN_FILENAME),
        )
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}
