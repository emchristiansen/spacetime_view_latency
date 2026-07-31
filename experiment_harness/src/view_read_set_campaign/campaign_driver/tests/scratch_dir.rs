//! Shared fixture: a fresh, empty scratch directory under the system temp dir.
//!
//! Duplicated per test tree rather than shared across them, exactly as the composition check's
//! fixture of the same name is, so each tree owns its own label namespace.

use std::fs;
use std::path::PathBuf;

/// A fresh scratch directory unique to `label` and this process, created empty (any prior contents
/// are removed first). Distinct labels keep tests in the same process from colliding.
pub(super) fn scratch_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("campaign-driver-{label}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clearing a prior scratch dir");
    }
    fs::create_dir_all(&dir).expect("creating the scratch dir");
    dir
}
