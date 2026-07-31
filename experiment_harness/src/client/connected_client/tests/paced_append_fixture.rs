//! Shared instant arithmetic for the paced append barrier's tests.
//!
//! The fail-fast policy is not relaxed for test code. A deadline or an observed instant that wrapped
//! silently would hand the barrier a scenario nobody intended — a deadline already past, or an
//! endpoint before its own start — and the result would then be reported as a property of the
//! barrier rather than of the fixture. The production path derives its deadline the same way, so the
//! tests and the code under test share one arithmetic discipline.

use std::time::{Duration, Instant};

/// `start` advanced by `after`, failing loud rather than wrapping.
pub(super) fn checked_after(start: Instant, after: Duration) -> Instant {
    start
        .checked_add(after)
        .expect("a fixture instant must be representable on the monotonic clock")
}
