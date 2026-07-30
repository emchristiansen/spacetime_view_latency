//! Settling a suffix yields exactly one `NotRun` record per slot, for every suffix including empty.

use crate::indexed_sender_view_calibration_pilot::attempt_inventory::AttemptInventory;
use crate::indexed_sender_view_calibration_pilot::calibration_driver::settle_remaining;
use crate::indexed_sender_view_calibration_pilot::not_run_reason::NotRunReason;
use crate::indexed_sender_view_calibration_pilot::pinned_artifact_identity::PinnedArtifactIdentity;

/// Coverage: the promise the frozen inventory makes — one terminal record per predeclared attempt,
/// including the attempts a run that stopped early never reached.
///
/// **The signature is half the property.** `settle_remaining` returns a `Vec`, not a `Result`, so
/// suffix closure cannot fail for any reason at all; this test would not compile if that regressed.
/// Before this correction the function was fallible only because `CalibrationRecord::not_run`
/// returned an unearned `Result` copied from a sibling whose equivalent validates a supersession link
/// this module does not have — which put a `?` on precisely the path that exists to keep the promise.
///
/// Every suffix is exercised, empty included: the last slot's release-failure path settles
/// `&attempts[index + 1..]`, which is empty when the failure lands on the final attempt, and that
/// must append nothing rather than misbehave.
///
/// The shape is asserted through the serialized form rather than a discriminant accessor, because
/// what a later reader actually gets is the ledger line, and `NotRun` is the only shape a slot that
/// never entered acquisition may take.
#[test]
fn suffix_settlement_records_every_remaining_slot() {
    let inventory = AttemptInventory::frozen().expect("the frozen inventory seals");
    let attempts = inventory.attempts();
    let pinned = PinnedArtifactIdentity::frozen().expect("the pinned constants resolve");
    let reason = NotRunReason::EnvironmentRefused;

    for start in 0..=attempts.len() {
        let expected = attempts
            .len()
            .checked_sub(start)
            .expect("start never exceeds the inventory length");

        let settled = settle_remaining(&attempts[start..], &pinned, 7, &reason);
        assert_eq!(
            settled.len(),
            expected,
            "settling the suffix from {start} must record every slot it was given"
        );

        for record in &settled {
            let line = serde_json::to_value(record).expect("a record serializes");
            assert!(
                line.get("NotRun").is_some(),
                "a slot that never entered acquisition must be recorded as NotRun, got {line}"
            );
        }
    }
}
