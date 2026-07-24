//! The required output path for the durable NDJSON observation stream.

use std::path::{Path, PathBuf};

/// The required filesystem path of the campaign's durable NDJSON output. A newtype so the required
/// output destination is never confused with an arbitrary path; there is no default — the caller
/// must supply one explicitly (spec: records go "to a required output path", never stdout).
#[derive(Debug, Clone)]
pub(crate) struct OutputPath(PathBuf);

impl OutputPath {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}
