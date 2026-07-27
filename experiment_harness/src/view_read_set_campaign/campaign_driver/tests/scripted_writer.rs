//! Shared fixture: a [`DurableLineWriter`] that succeeds a fixed number of times, then fails.

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;

/// A writer that returns success for its first `remaining_ok` `write_line` calls and an error on
/// every call after that, so a test can drive the sink to a persist failure at a chosen point
/// without a real I/O fault. The sink's own tests use the same fixture with an extra scripted
/// `finalize` failure; these tests never finalize, so only the write seam is copied.
pub(super) struct ScriptedWriter {
    remaining_ok: usize,
}

impl ScriptedWriter {
    /// A writer whose first `remaining_ok` line writes succeed.
    pub(super) fn ok_for(remaining_ok: usize) -> Self {
        Self { remaining_ok }
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
        Ok(())
    }
}
