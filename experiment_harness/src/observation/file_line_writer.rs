//! The production [`DurableLineWriter`] backed by the required output file and its directory.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;
use crate::observation::output_path::OutputPath;
use crate::observation::sink_create_error::SinkCreateError;

/// Appends NDJSON lines to the required output file, `sync_data`ing each and holding a handle to the
/// containing directory so finalization can fsync the directory entry.
///
/// Durability boundary (POSIX): a newly created file's directory entry is not crash-durable until the
/// containing directory is fsync'd, and file data is not durable until `sync_data`. [`Self::create`]
/// completes the directory-entry sync contract once the file exists; [`Self::write_line`]
/// `sync_data`s each line; [`Self::finalize`] syncs both the file metadata and the directory. These
/// exercise the fsync contract — crash durability itself is the platform honoring that contract, not
/// something the process can observe.
pub(crate) struct FileLineWriter {
    file: File,
    /// A handle to the containing directory, kept so finalization can fsync it.
    parent: File,
}

impl FileLineWriter {
    /// Create the required output file *exclusively* (`create_new`, i.e. `O_EXCL`): opening an
    /// existing path for overwrite is a fail-fast error rather than a truncation. `create_new`
    /// prevents *this process* from clobbering an existing file, subject to filesystem semantics.
    /// Once the file exists, the sync contract for its directory entry is completed by fsync of the
    /// containing directory before any record is written.
    ///
    /// The typed error distinguishes the two failure regimes the campaign frontier needs: a failing
    /// create/open produced no file by this call ([`SinkCreateError::NoFileCreatedByThisCall`]); a
    /// failing directory open/sync leaves the just-created file in place with ambiguous presence
    /// durability ([`SinkCreateError::PresenceDurabilityAmbiguous`]) — the file is never removed.
    pub(crate) fn create(output: &OutputPath) -> std::result::Result<Self, SinkCreateError> {
        let path = output.path();
        let parent_path = match path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| {
                SinkCreateError::no_file_created_by_this_call(format!(
                    "creating the observation output file exclusively at {} created no file by this \
                     call: {e}",
                    path.display()
                ))
            })?;
        // The file now exists. If the directory fsync fails, report presence ambiguity and leave the
        // file so the operator can see the partial state, rather than silently removing it.
        let parent = File::open(&parent_path).map_err(|e| {
            SinkCreateError::presence_durability_ambiguous(format!(
                "the output file {} was created but its directory {} could not be opened to fsync \
                 the new entry; the file is left in place and its durability is unknown: {e}",
                path.display(),
                parent_path.display()
            ))
        })?;
        parent.sync_all().map_err(|e| {
            SinkCreateError::presence_durability_ambiguous(format!(
                "the output file {} was created but syncing its directory entry failed; the file is \
                 left in place and its presence may not be crash-durable: {e}",
                path.display()
            ))
        })?;
        Ok(Self { file, parent })
    }
}

impl DurableLineWriter for FileLineWriter {
    fn write_line(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.file.write_all(bytes)?;
        self.file.flush()?;
        self.file.sync_data()?;
        Ok(())
    }

    fn finalize(&mut self) -> std::result::Result<(), FinalSyncFailures> {
        // Attempt both syncs unconditionally and retain both errors in independent typed slots — a
        // failed file sync must not skip the directory sync, and neither error is merged or discarded
        // (mirrors the run driver's primary+teardown aggregation). The two-call sequencing lives in
        // the shared, unit-tested `from_attempts` helper rather than being duplicated here.
        FinalSyncFailures::from_attempts(|| self.file.sync_all(), || self.parent.sync_all())
    }
}
