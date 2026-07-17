//! The durability I/O seam the sink writes each record line through.

use crate::observation::final_sync_failures::FinalSyncFailures;

/// The append-and-sync boundary the [`ObservationSink`](super::observation_sink::ObservationSink)
/// drives, separated from the sink's sequencing/poisoning logic so a failing implementation can be
/// injected in tests. The production implementation is
/// [`FileLineWriter`](super::file_line_writer::FileLineWriter); a test implementation scripts write
/// failures to prove the sink's terminal-on-failure behavior without needing a real I/O fault.
///
/// A [`Self::write_line`] call performs the whole post-serialization step — write, flush, and
/// `sync_data` — and returns success only when all three return success. The sink therefore treats
/// any error from it as a *post-write* failure whose durability is ambiguous — a full line may
/// already be durable even when a later sub-step reports an error — never as "definitely absent."
///
/// A successful return means the sync calls the writer issued returned success: the sink may take
/// that as operational evidence the records are durable, but that is distinct from physical crash
/// proof, which is the platform honoring the fsync contract rather than anything this process can
/// observe.
pub(crate) trait DurableLineWriter {
    /// Append one already-serialized line (its trailing newline included), then flush and `sync_data`
    /// it. Returns success only when the write, flush, and `sync_data` all return success; any error
    /// leaves the record's durability ambiguous.
    fn write_line(&mut self, bytes: &[u8]) -> std::io::Result<()>;

    /// Sync the file metadata and the containing directory entry. Both `sync_all` calls are attempted
    /// unconditionally — a failed file sync must not skip the directory sync — and each failure is
    /// retained in its own [`FinalSyncFailures`] slot. Returns success only when both succeed; a
    /// failure leaves finalization ambiguous.
    fn finalize(&mut self) -> std::result::Result<(), FinalSyncFailures>;
}
