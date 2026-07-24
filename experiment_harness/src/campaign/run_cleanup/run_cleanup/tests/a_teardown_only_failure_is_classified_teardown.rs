//! A clean disconnect with a failing teardown classifies as `Teardown`, carrying the teardown error —
//! not as a `Disconnect` with a hidden teardown result.

use anyhow::anyhow;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;

use super::super::ordered_cleanup;

/// Disconnect succeeds, teardown fails. This is the one asymmetric case: because the disconnect was
/// clean, the failure is attributed to teardown alone (`Teardown(error)`), keeping the two failure
/// shapes distinct rather than folding a teardown-only failure into the `Disconnect` variant.
#[test]
fn a_teardown_only_failure_is_classified_teardown() {
    let outcome = ordered_cleanup(|| Ok(()), || Err(anyhow!("teardown boom")));

    match outcome {
        RunCleanupOutcome::Failed(RunCleanupFailure::Teardown(error)) => {
            assert!(
                error.to_string().contains("teardown boom"),
                "the teardown error is retained: {error}"
            );
        }
        other => panic!("expected a Teardown cleanup failure, got {other:?}"),
    }
}
