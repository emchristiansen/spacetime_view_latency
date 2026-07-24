//! Shared fixture: a channel-backed no-I/O [`DurableLineWriter`] that hands each written line's bytes to a
//! retained receiver, so a test can parse exactly the bytes the sink serialized.

use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use crate::observation::durable_line_writer::DurableLineWriter;
use crate::observation::final_sync_failures::FinalSyncFailures;

/// A writer that performs no I/O but *sends* every line handed to [`Self::write_line`] to a retained
/// [`Receiver`], so a test can inspect exactly the bytes the sink serialized — without shared interior
/// mutability or a mutex. The sink takes ownership of the boxed writer; the test keeps the receiver end,
/// returned alongside the writer from [`Self::new`].
pub(super) struct CapturingLineWriter {
    lines: Sender<Vec<u8>>,
}

impl CapturingLineWriter {
    /// A fresh capturing writer plus the receiver through which the test reads the captured lines after the
    /// sink has written to it.
    pub(super) fn new() -> (Self, Receiver<Vec<u8>>) {
        let (lines, rx) = std::sync::mpsc::channel();
        (Self { lines }, rx)
    }
}

impl DurableLineWriter for CapturingLineWriter {
    fn write_line(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.lines
            .send(bytes.to_vec())
            .expect("the capturing receiver is retained for the writer's lifetime");
        Ok(())
    }

    fn finalize(&mut self) -> std::result::Result<(), FinalSyncFailures> {
        Ok(())
    }
}
