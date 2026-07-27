//! Shared fixture: a [`DurableLineWriter`] that succeeds a fixed number of times, then fails, and
//! counts every call it was handed.

use std::cell::RefCell;
use std::rc::Rc;

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;

/// A writer that returns success for its first `remaining_ok` `write_line` calls and an error on
/// every call after that, so a test can drive the sink to a persist failure at a chosen point
/// without a real I/O fault. The sink's own tests use the same fixture with an extra scripted
/// `finalize` failure; these tests never finalize, so only the write seam is copied.
///
/// It also counts the calls, sharing the count through the `Rc<RefCell<…>>` handle
/// [`CapturingWriter`](super::capturing_writer::CapturingWriter) uses for its lines, because the sink
/// takes ownership of the boxed writer. A test that must prove a write was never *attempted* — not
/// merely that its error did not surface — needs the count; an error-text assertion cannot
/// distinguish a call that was never made from one whose failure was discarded.
pub(super) struct ScriptedWriter {
    remaining_ok: usize,
    calls: Rc<RefCell<usize>>,
}

impl ScriptedWriter {
    /// A writer whose first `remaining_ok` line writes succeed, for a test that does not read the
    /// count.
    pub(super) fn ok_for(remaining_ok: usize) -> Self {
        Self::counted(remaining_ok).0
    }

    /// The same writer plus the shared handle through which the test reads how many lines the sink
    /// actually tried to write.
    pub(super) fn counted(remaining_ok: usize) -> (Self, Rc<RefCell<usize>>) {
        let calls = Rc::new(RefCell::new(0));
        (
            Self {
                remaining_ok,
                calls: Rc::clone(&calls),
            },
            calls,
        )
    }
}

impl DurableLineWriter for ScriptedWriter {
    fn write_line(&mut self, _bytes: &[u8]) -> std::io::Result<()> {
        *self.calls.borrow_mut() += 1;
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
