//! Shared fixture: a no-I/O [`DurableLineWriter`] scripted to drive the campaign cursor's paths.

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;

/// A writer that performs no real I/O: it *returns* success for its first `remaining_ok`
/// [`Self::write_line`] calls and an error on every call after that, and its [`Self::finalize`] returns
/// success or a scripted file+directory sync failure. This lets a campaign test drive the whole cursor
/// tree — and inject a persist failure at a chosen record or a finalize failure — without a real fault.
///
/// A successful return proves only that the sink treats the seam return as advancing its
/// sequence/marker; it never proves bytes were written or synced (see the sink-tests note). Mirrors the
/// sink tests' scripted writer, kept local to the campaign tests.
pub(super) struct DrivingWriter {
    remaining_ok: usize,
    finalize_fails: bool,
}

impl DrivingWriter {
    /// A writer whose every line write and whose finalize return success — used to drive a run, block,
    /// or the whole schedule to completion.
    pub(super) fn always_ok() -> Self {
        Self {
            remaining_ok: usize::MAX,
            finalize_fails: false,
        }
    }

    /// A writer whose every line write returns success but whose finalize fails, to drive the
    /// finalization-only failure path after a fully-executed schedule.
    pub(super) fn finalize_failing() -> Self {
        Self {
            remaining_ok: usize::MAX,
            finalize_fails: true,
        }
    }

    /// A writer whose first `remaining_ok` line writes return success and whose next write fails, to
    /// drive a manifest- or dose-write failure at an exact record.
    pub(super) fn ok_for(remaining_ok: usize) -> Self {
        Self {
            remaining_ok,
            finalize_fails: false,
        }
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
        if self.finalize_fails {
            return FinalSyncFailures::combine(
                Err(std::io::Error::other("scripted file sync failure")),
                Err(std::io::Error::other("scripted directory sync failure")),
            );
        }
        Ok(())
    }
}
