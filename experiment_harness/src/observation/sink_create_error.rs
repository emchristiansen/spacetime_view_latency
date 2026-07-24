//! Why creating the durable sink's output file did not cleanly complete.

/// A sink-creation failure, distinguishing whether *this call* created a file. This is machine state
/// a future campaign's typed output-creation frontier needs, not just prose:
/// `NoFileCreatedByThisCall` means the exclusive create/open returned an error before this call
/// created anything — it left the path unchanged, though a file may already exist (e.g. exclusive
/// create rejected a pre-existing path); `PresenceDurabilityAmbiguous` means this call created the
/// file but syncing its directory entry (or opening the directory to do so) failed, so whether the
/// new file's presence is crash-durable is unknown and the file is left in place.
#[derive(Debug)]
pub(crate) enum SinkCreateError {
    /// The exclusive create/open returned an error, so this call created no file and left the path
    /// unchanged. A file may already exist at the path — the underlying cause (e.g. "already
    /// exists") is preserved in the diagnostic.
    NoFileCreatedByThisCall { diagnostic: String },
    /// This call created the file, but the directory-entry sync contract did not complete; the file
    /// is left in place and whether its presence is crash-durable is unknown.
    PresenceDurabilityAmbiguous { diagnostic: String },
}

impl SinkCreateError {
    pub(crate) fn no_file_created_by_this_call(diagnostic: String) -> Self {
        Self::NoFileCreatedByThisCall { diagnostic }
    }

    pub(crate) fn presence_durability_ambiguous(diagnostic: String) -> Self {
        Self::PresenceDurabilityAmbiguous { diagnostic }
    }

    /// The human-readable diagnostic for the failure.
    pub(crate) fn diagnostic(&self) -> &str {
        match self {
            SinkCreateError::NoFileCreatedByThisCall { diagnostic }
            | SinkCreateError::PresenceDurabilityAmbiguous { diagnostic } => diagnostic,
        }
    }
}
