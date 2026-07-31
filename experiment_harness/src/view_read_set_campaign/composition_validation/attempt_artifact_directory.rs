//! The one directory an attempt's retained observations live in.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its whole content is
//! the claim that this directory was *exclusively created* for exactly one attempt identity. A
//! private field is visible to its declaring module **and every descendant**, so a `#[cfg(test)] mod
//! tests` child — or any child added later — could write the struct literal around an arbitrary
//! path and hand it to [`ObservedRowSet::persisted`](super::observed_row_set::ObservedRowSet),
//! filing one attempt's rows under another attempt's identity or outside the campaign root
//! altogether. `sealed` has no children, so [`AttemptArtifactDirectory::create`] really is the only
//! door.

mod sealed {
    use std::fs;
    use std::path::{Path, PathBuf};

    use anyhow::{ensure, Context, Result};

    use crate::view_read_set_campaign::attempt_key::AttemptKey;

    /// The exclusively-created child of the campaign artifact root that one attempt's observed row
    /// sets are retained in.
    ///
    /// **Why a per-attempt directory at all.** The CLI supplies one campaign-wide artifact root,
    /// and the calibration inventory is sixty attempts each retaining two observations. The phase
    /// label distinguishes the two *within* an attempt, so without an identity-derived child every
    /// attempt after the first would collide on the same two names.
    ///
    /// **Why exclusive creation.** `create_dir` rather than `create_dir_all`: a repeated
    /// [`AttemptKey`] fails loud here instead of silently sharing a directory with the evidence
    /// already written under that identity, where the second attempt's exclusive artifact creation
    /// would fail anyway — but further along, with the first attempt's files now ambiguous. The
    /// campaign root is likewise required to exist rather than invented, so a mistyped
    /// `--artifacts-dir` fails at this call instead of scattering evidence into a fresh tree.
    ///
    /// **What that failure is and is not early enough to prevent.** In the frozen call flow this
    /// directory is created during `measure_attempt`, which runs *after* the attempt's fresh server
    /// has been provisioned and published — so a bad root does not fail before provisioning, and
    /// this type must not be read as a preflight check on the operator's arguments. What it does
    /// guarantee is that no row set is persisted and no composition finding is admitted under a
    /// root that does not exist or an identity that already has one.
    ///
    /// **Why not a temporary directory.** [`FreshDataDir`](crate::provision::fresh_data_dir) uses a
    /// random-suffixed temp dir because a server's data is disposable. These artifacts are the
    /// opposite: they are the retained evidence a finding points at, so their location must be
    /// derivable from the recorded identity by a reader who was not there. A random suffix would
    /// make the path an unreproducible fact about one process.
    ///
    /// **Why it holds no cleanup obligation.** Unlike `FreshDataDir`, this directory is meant to
    /// outlive the campaign, so there is no `cleanup()` and no `Drop` assertion — dropping the
    /// handle drops a path, never the evidence.
    #[derive(Debug)]
    pub(crate) struct AttemptArtifactDirectory {
        path: PathBuf,
    }

    impl AttemptArtifactDirectory {
        /// Exclusively create `root`'s child named by `attempt`'s canonical identity spelling.
        ///
        /// `root` must already exist: it is the operator-supplied campaign artifact directory, and
        /// creating it here would turn a wrong path into a plausible-looking empty campaign.
        ///
        /// The child component is [`AttemptKey::canonical_tag`], which is built from explicit
        /// canonical accessors on every one of the identity's six components — so the name contains
        /// no path separator and cannot be `.` or `..`, and traversal out of `root` is
        /// unrepresentable rather than rejected.
        ///
        /// The root is checked separately from the child so the two failures stay distinguishable:
        /// a missing or non-directory root is reported as exactly that, and the `create_dir` error
        /// is reported with its own IO cause intact rather than being asserted to mean an identity
        /// collision — `AlreadyExists` does mean one, but a permission or storage failure does not,
        /// and a message that named only the collision would misdiagnose both.
        pub(crate) fn create(root: &Path, attempt: AttemptKey) -> Result<Self> {
            ensure!(
                root.is_dir(),
                "the campaign artifact root {} is not an existing directory; it is supplied by the \
                 operator and is never created here, so evidence is not scattered into a fresh tree \
                 on a mistyped path",
                root.display(),
            );

            let path = root.join(attempt.canonical_tag());
            fs::create_dir(&path).with_context(|| {
                format!(
                    "exclusively creating the attempt artifact directory {}; an AlreadyExists cause \
                     means this attempt identity has already retained evidence",
                    path.display()
                )
            })?;
            Ok(Self { path })
        }

        /// Where this attempt's observed row sets are written.
        pub(crate) fn path(&self) -> &Path {
            &self.path
        }
    }
}

pub(crate) use sealed::AttemptArtifactDirectory;

#[cfg(test)]
mod tests;
