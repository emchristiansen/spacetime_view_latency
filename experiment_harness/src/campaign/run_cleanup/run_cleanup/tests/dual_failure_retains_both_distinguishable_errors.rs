//! Both effects fail: the outcome is a `Disconnect` failure that retains *both* the disconnect error
//! and the teardown error, and the two are distinguishable — neither masks the other.

use anyhow::anyhow;

use crate::campaign::run_cleanup_failure::RunCleanupFailure;
use crate::campaign::run_cleanup_outcome::RunCleanupOutcome;

use super::super::ordered_cleanup;

/// The simultaneous-cleanup-failure case the provisional reported-marker states could not represent:
/// disconnect fails *and* teardown fails. The `Disconnect` variant carries the disconnect error and
/// the teardown result, and here that result is the teardown's own `Err` — both errors survive, each
/// with its own message, so a diagnosis can tell exactly what failed on each step.
#[test]
fn dual_failure_retains_both_distinguishable_errors() {
    let outcome = ordered_cleanup(
        || Err(anyhow!("disconnect boom")),
        || Err(anyhow!("teardown boom")),
    );

    match outcome {
        RunCleanupOutcome::Failed(RunCleanupFailure::Disconnect { error, teardown }) => {
            assert!(
                error.to_string().contains("disconnect boom"),
                "the disconnect error is retained: {error}"
            );
            let teardown =
                teardown.expect_err("the teardown also failed, so its error is retained");
            assert!(
                teardown.to_string().contains("teardown boom"),
                "the teardown error is retained: {teardown}"
            );
            assert_ne!(
                error.to_string(),
                teardown.to_string(),
                "the two retained errors are distinguishable, so neither masks the other"
            );
        }
        other => {
            panic!("expected a Disconnect failure retaining the teardown error, got {other:?}")
        }
    }
}
