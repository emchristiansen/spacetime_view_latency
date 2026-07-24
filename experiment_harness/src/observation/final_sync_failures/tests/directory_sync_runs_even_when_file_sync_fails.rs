//! The directory sync is attempted even when the file sync fails first, and both failures are
//! retained in their own typed slots. This exercises the exact `from_attempts` sequencing that
//! [`FileLineWriter::finalize`](crate::observation::file_line_writer::FileLineWriter) uses, with
//! attempt counters standing in for the real `sync_all` calls.

use std::cell::Cell;
use std::io;

use crate::observation::final_sync_failures::FinalSyncFailures;

#[test]
fn directory_sync_runs_even_when_file_sync_fails() {
    let file_attempts = Cell::new(0u32);
    let directory_attempts = Cell::new(0u32);

    let result = FinalSyncFailures::from_attempts(
        || {
            file_attempts.set(file_attempts.get() + 1);
            Err(io::Error::other("file sync failed"))
        },
        || {
            directory_attempts.set(directory_attempts.get() + 1);
            Err(io::Error::other("directory sync failed"))
        },
    );

    assert_eq!(
        file_attempts.get(),
        1,
        "the file sync is attempted exactly once"
    );
    assert_eq!(
        directory_attempts.get(),
        1,
        "the directory sync runs even though the file sync failed first"
    );

    let failures = result.expect_err("both syncs failed");
    assert!(
        failures.file().is_some(),
        "the file sync failure is retained in its own slot"
    );
    assert!(
        failures.directory().is_some(),
        "the directory sync failure is retained in its own slot"
    );
}
