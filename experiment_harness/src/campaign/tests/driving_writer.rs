//! Shared fixture: a no-I/O [`DurableLineWriter`] scripted to drive the run cursor's write paths.

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;

/// A writer that performs no real I/O: it *returns* success for its first `remaining_ok`
/// [`Self::write_line`] calls and an error on every call after that; its [`Self::finalize`] always
/// returns success. This lets a run-cursor test drive the manifest/dose write path — and inject a persist
/// failure at a chosen record — without a real fault.
///
/// A successful return proves only that the sink treats the seam return as advancing its
/// sequence/marker; it never proves bytes were written or synced (see the sink-tests note). Mirrors the
/// sink tests' scripted writer, kept local to the campaign tests.
pub(super) struct DrivingWriter {
    remaining_ok: usize,
}

impl DrivingWriter {
    /// A writer whose every line write returns success — used to drive a run through its full ladder.
    pub(super) fn always_ok() -> Self {
        Self {
            remaining_ok: usize::MAX,
        }
    }

    /// A writer whose first `remaining_ok` line writes return success and whose next write fails, to
    /// drive a manifest- or dose-write failure at an exact record.
    pub(super) fn ok_for(remaining_ok: usize) -> Self {
        Self { remaining_ok }
    }
}

impl DurableLineWriter for DrivingWriter {
    fn write_line(&mut self, _bytes: &[u8]) -> std::io::Result<()> {
        if self.remaining_ok == 0 {
            return Err(std::io::Error::other("scripted write_line failure"));
        }
        self.remaining_ok -= 1;
        Ok(())
    }

    fn finalize(&mut self) -> std::result::Result<(), FinalSyncFailures> {
        Ok(())
    }
}
