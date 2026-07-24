//! A disconnect failure with a clean teardown: the classification is `Disconnect`, and the
//! still-attempted teardown's `Ok(())` is retained verbatim (not dropped, not treated as absent).

use anyhow::anyhow;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;

use super::super::ordered_cleanup;

/// Disconnect fails, teardown succeeds. The `Disconnect` variant must carry the disconnect error and
/// the teardown result, and that retained teardown result must be the `Ok(())` the teardown actually
/// returned — the shape that lets a caller see "the client leaked but the server came down cleanly."
#[test]
fn a_disconnect_failure_retains_the_ok_teardown() {
    let outcome = ordered_cleanup(|| Err(anyhow!("disconnect boom")), || Ok(()));

    match outcome {
        RunCleanupOutcome::Failed(RunCleanupFailure::Disconnect { error, teardown }) => {
            assert!(
                error.to_string().contains("disconnect boom"),
                "the disconnect error is retained: {error}"
            );
            assert!(
                teardown.is_ok(),
                "the still-attempted teardown returned success and is retained verbatim, not dropped"
            );
        }
        other => panic!("expected a Disconnect failure retaining an Ok teardown, got {other:?}"),
    }
}
