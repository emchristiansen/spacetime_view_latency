//! Shared fixture: a [`DurableLineWriter`] that captures each line's bytes for inspection.

use std::cell::RefCell;
use std::rc::Rc;

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;

/// A writer that performs no I/O but records every line handed to [`Self::write_line`], so a test can
/// inspect exactly the bytes the sink serialized. It shares its capture buffer with the test via an
/// `Rc<RefCell<…>>` handle returned from [`Self::new`], because the sink takes ownership of the boxed
/// writer.
pub(super) struct CapturingWriter {
    lines: Rc<RefCell<Vec<Vec<u8>>>>,
}

impl CapturingWriter {
    /// A fresh capturing writer plus the shared handle through which the test reads the captured
    /// lines after the sink has written and been consumed.
    pub(super) fn new() -> (Self, Rc<RefCell<Vec<Vec<u8>>>>) {
        let lines = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                lines: Rc::clone(&lines),
            },
            lines,
        )
    }
}

impl DurableLineWriter for CapturingWriter {
    fn write_line(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.lines.borrow_mut().push(bytes.to_vec());
        Ok(())
    }

    fn finalize(&mut self) -> std::result::Result<(), FinalSyncFailures> {
        Ok(())
    }
}
