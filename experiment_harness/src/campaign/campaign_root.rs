//! The required campaign-root directory both `Campaign` and `Analyze` operate under.

use std::path::{Path, PathBuf};

/// The validated `--campaign-root` directory argument shared by `Campaign` and `Analyze`. A newtype so
/// the campaign's fixed artifact family is never confused with an arbitrary caller-named path; there is
/// no default — the caller must supply one explicitly (spec: "Campaign takes one required
/// `--campaign-root <dir>` argument, never a caller-named output file").
#[derive(Debug, Clone)]
pub(crate) struct CampaignRoot(PathBuf);

impl CampaignRoot {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}
