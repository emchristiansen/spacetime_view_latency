//! The independent file and directory failures of a best-effort sink finalization.

/// The two final-sync results retained independently: the file-metadata `sync_all` and the
/// directory-entry `sync_all` are each attempted unconditionally, and each slot holds that call's
/// error if it failed. At least one slot is populated — [`Self::combine`] returns `Ok(())` when both
/// syncs succeed — so a both-succeeded failure value is unrepresentable. Neither error is merged into
/// the other, so a caller can inspect each independently.
#[derive(Debug)]
pub(crate) struct FinalSyncFailures {
    file: Option<std::io::Error>,
    directory: Option<std::io::Error>,
}

impl FinalSyncFailures {
    /// Attempt the file-metadata sync and then the directory-entry sync, calling **both** closures
    /// unconditionally — the directory sync runs even when the file sync fails — and retaining each
    /// failure in its own slot. This is the sole home of the two-call finalization sequencing so that
    /// "both are always attempted" is one code path (used by
    /// [`FileLineWriter::finalize`](super::file_line_writer::FileLineWriter) and unit-tested here),
    /// not a discipline duplicated at the call site.
    pub(crate) fn from_attempts<F, D>(file: F, directory: D) -> std::result::Result<(), Self>
    where
        F: FnOnce() -> std::io::Result<()>,
        D: FnOnce() -> std::io::Result<()>,
    {
        // Evaluate both before combining: the directory attempt must not be short-circuited by a
        // failed file attempt.
        let file = file();
        let directory = directory();
        Self::combine(file, directory)
    }

    /// Combine already-evaluated file and directory sync results: `Ok(())` iff both succeeded, else a
    /// value holding exactly the failed slots. Neither error is discarded or merged. The shared core
    /// of [`Self::from_attempts`].
    pub(crate) fn combine(
        file: std::io::Result<()>,
        directory: std::io::Result<()>,
    ) -> std::result::Result<(), Self> {
        match (file.err(), directory.err()) {
            (None, None) => Ok(()),
            (file, directory) => Err(Self { file, directory }),
        }
    }

    /// The file-metadata `sync_all` error, if that sync failed.
    pub(crate) fn file(&self) -> Option<&std::io::Error> {
        self.file.as_ref()
    }

    /// The directory-entry `sync_all` error, if that sync failed.
    pub(crate) fn directory(&self) -> Option<&std::io::Error> {
        self.directory.as_ref()
    }
}

#[cfg(test)]
mod tests;
