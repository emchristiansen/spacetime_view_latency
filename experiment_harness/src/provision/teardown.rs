//! Aggregating teardown helpers for provisioning capabilities.
//!
//! Provisioning owns two capabilities whose cleanup must be *attempted on every path*
//! (success and failure alike) and whose cleanup errors must never be silently lost: a
//! running server [`Child`] and an owned [`FreshDataDir`]. These helpers push each cleanup
//! error into a caller-owned accumulator instead of returning early, so a caller can attempt
//! all outstanding teardowns and then aggregate the primary error together with every cleanup
//! error into one returned [`Error`] (spec/Control: "attempt all outstanding teardowns …
//! aggregate primary + every cleanup error").

use std::process::Child;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Error};

use crate::provision::fresh_data_dir::FreshDataDir;

/// Bounded post-kill reap: at most `REAP_MAX_ATTEMPTS` non-blocking `try_wait` polls,
/// `REAP_POLL_INTERVAL` apart (~5s ceiling). A blocking `wait` could hang forever if the kill
/// did not take effect, so reaping is always bounded.
const REAP_MAX_ATTEMPTS: u32 = 100;
const REAP_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Kill `child` only if it is still running, then reap it within a bounded deadline, pushing any
/// error into `errors`.
///
/// `try_wait` reaps the process on Unix the moment it observes the exit and is documented to keep
/// returning that status; an already-reaped child is represented explicitly (no further wait).
/// If the child is still running it is killed and then reaped by a **bounded** `try_wait` loop —
/// never a blocking `wait`, which could hang forever if the kill did not take effect. The
/// invariant is preserved honestly: termination and reaping are always attempted, and a survivor
/// (or an indeterminate OS state) is reported as a cleanup failure rather than assumed gone.
pub(crate) fn reap_child(mut child: Child, errors: &mut Vec<Error>) {
    let pid = child.id();
    match child.try_wait() {
        // Exited and reaped by `try_wait`; nothing left to kill or reap.
        Ok(Some(_status)) => return,
        // Still running — request termination; the bounded reap below collects it.
        Ok(None) => {
            if let Err(e) = child.kill() {
                errors.push(
                    Error::new(e).context(format!("killing the server child process (pid {pid})")),
                );
            }
        }
        // Unknown state — record the poll error, then still attempt termination. The `Child` is
        // uniquely owned and not yet reaped, so its pid cannot have been recycled; never leave a
        // possible survivor un-killed just because status inspection failed.
        Err(e) => {
            errors.push(
                Error::new(e)
                    .context(format!("polling the server child process (pid {pid}) before kill")),
            );
            if let Err(e) = child.kill() {
                errors.push(
                    Error::new(e).context(format!(
                        "killing the server child process (pid {pid}) after a poll error"
                    )),
                );
            }
        }
    }
    bounded_reap(&mut child, pid, errors);
}

/// Bounded non-blocking reap. Returns once the child is reaped; on a poll error or on exceeding
/// the deadline it records a cleanup failure that explicitly does **not** claim the process is
/// gone.
fn bounded_reap(child: &mut Child, pid: u32, errors: &mut Vec<Error>) {
    for _ in 0..REAP_MAX_ATTEMPTS {
        match child.try_wait() {
            Ok(Some(_status)) => return,
            Ok(None) => thread::sleep(REAP_POLL_INTERVAL),
            Err(e) => {
                errors.push(Error::new(e).context(format!(
                    "polling the server child process (pid {pid}) during reap; \
                     it may still be running"
                )));
                return;
            }
        }
    }
    errors.push(anyhow!(
        "server child process (pid {pid}) was not reaped within {REAP_MAX_ATTEMPTS} attempts; \
         it may still be running"
    ));
}

/// Attempt to remove `data_dir`, pushing any error into `errors`.
pub(crate) fn cleanup_data_dir(data_dir: FreshDataDir, errors: &mut Vec<Error>) {
    if let Err(e) = data_dir.cleanup() {
        errors.push(e);
    }
}

/// Collapse a **non-empty** error list into one aggregate [`Error`]. A single error is returned
/// unchanged (preserving its chain); multiple errors are enumerated (each rendered with its full
/// chain via `{:#}`) so no cleanup failure is hidden behind the primary one.
pub(crate) fn into_error(mut errors: Vec<Error>) -> Error {
    assert!(
        !errors.is_empty(),
        "into_error called with no errors — a caller returned failure without a cause"
    );
    if errors.len() == 1 {
        return errors.pop().expect("length checked to be exactly 1");
    }
    let rendered = errors
        .iter()
        .enumerate()
        .map(|(i, e)| format!("  [{}] {e:#}", i + 1))
        .collect::<Vec<_>>()
        .join("\n");
    anyhow!("{} errors during provisioning:\n{rendered}", errors.len())
}
