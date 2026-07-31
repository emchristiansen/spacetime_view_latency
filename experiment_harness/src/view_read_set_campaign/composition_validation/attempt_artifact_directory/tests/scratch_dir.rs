//! Shared fixture: a fresh, empty scratch directory under the system temp dir.
//!
//! Duplicated from the observation sink's fixture of the same name rather than shared across
//! subtrees, which is the established pattern here: each test tree owns its own label namespace, so
//! a rename in one cannot silently collide another's directories.

use std::fs;
use std::path::PathBuf;

/// A fresh scratch directory unique to `label` and this process, created empty (any prior contents
/// are removed first). Distinct labels keep tests in the same process from colliding.
///
/// Standing in for the operator-supplied campaign artifact root, which
/// [`AttemptArtifactDirectory::create`](crate::view_read_set_campaign::composition_validation::attempt_artifact_directory::AttemptArtifactDirectory::create)
/// requires to already exist.
pub(super) fn scratch_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "attempt-artifact-dir-{label}-{}",
        std::process::id()
    ));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("clearing a prior scratch dir");
    }
    fs::create_dir_all(&dir).expect("creating the scratch dir");
    dir
}
