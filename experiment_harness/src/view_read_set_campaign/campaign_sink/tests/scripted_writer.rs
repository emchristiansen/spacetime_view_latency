//! Shared fixture: a [`DurableLineWriter`] that succeeds a fixed number of times, then fails.

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;

/// A writer that returns success for its first `remaining_ok` `write_line` calls and an error on
/// every call after that, so a test can drive the sink to a persist failure at a chosen point
/// without a real I/O fault. `finalize` is scripted independently via `finalize_fails`. The same
/// fixture the historical sink's tests use, over the one shared durability seam.
pub(super) struct ScriptedWriter {
    remaining_ok: usize,
    finalize_fails: bool,
}

impl ScriptedWriter {
    /// A writer whose first `remaining_ok` line writes succeed and whose finalize succeeds.
    pub(super) fn ok_for(remaining_ok: usize) -> Self {
        Self {
            remaining_ok,
            finalize_fails: false,
        }
    }

    /// A writer whose first `remaining_ok` line writes succeed but whose finalize fails, to drive the
    /// sink's final-sync-failure path.
    pub(super) fn finalize_failing(remaining_ok: usize) -> Self {
        Self {
            remaining_ok,
            finalize_fails: true,
        }
    }
}

impl DurableLineWriter for ScriptedWriter {
    fn write_line(&mut self, _bytes: &[u8]) -> std::io::Result<()> {
        if self.remaining_ok == 0 {
            return Err(std::io::Error::other("scripted write_line failure"));
        }
        self.remaining_ok -= 1;
        Ok(())
    }

    fn finalize(&mut self) -> std::result::Result<(), FinalSyncFailures> {
        if self.finalize_fails {
            // Report both the file and directory sync as failed, so a test can assert the sink
            // surfaces the whole final-sync failure rather than one half of it.
            return FinalSyncFailures::combine(
                Err(std::io::Error::other("scripted file sync failure")),
                Err(std::io::Error::other("scripted directory sync failure")),
            );
        }
        Ok(())
    }
}
