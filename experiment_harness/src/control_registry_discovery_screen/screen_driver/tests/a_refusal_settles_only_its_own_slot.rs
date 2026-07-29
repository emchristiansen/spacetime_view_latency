//! A refusal terminally settles exactly one original and leaves the rest runnable, while an
//! inoperable gate settles that same slot and every one after it.

use anyhow::anyhow;

use crate::control_registry_discovery_screen::attempt_inventory::{
    AttemptInventory, SCREEN_ATTEMPT_COUNT,
};
use crate::control_registry_discovery_screen::diagnostic_artifact::DiagnosticArtifact;
use crate::control_registry_discovery_screen::not_run_reason::NotRunReason;
use crate::control_registry_discovery_screen::pinned_artifact_identity::PinnedArtifactIdentity;
use crate::control_registry_discovery_screen::screen_record::ScreenRecord;
use crate::control_registry_discovery_screen::supersession::Supersession;
use crate::manifest::schedule_seed::ScheduleSeed;

use super::super::{settle_remaining, ORIGINAL_HAS_NO_SUPERSESSION};

/// The slot the two stops are compared at: far enough in that a suffix is a proper subset of the
/// inventory, so "settles one" and "settles the rest" cannot coincide by accident.
const REFUSED_INDEX: usize = 4;

/// A diagnostic distinctive enough to find in serialized output.
const UNGATED_DIAGNOSTIC: &str = "the host waiter terminated without a verdict";

/// Coverage: the two ungated outcomes are not interchangeable, and the ledger says which happened.
///
/// This is the defect the typed exit contract exists to close, checked from the settlement side. A
/// refusal is a verdict about one slot: one record, that slot's own identity, and the screen goes on
/// to the next original. An inoperable gate is the absence of a verdict: the current slot and the
/// whole remaining suffix settle and the run stops. Recording the second as the first is exactly
/// what "nonzero means refused" used to do, and it is invisible in a count — both produce terminal
/// records — so the reasons are compared, not merely the totals.
///
/// The refusal carries no diagnostic by construction: `EnvironmentRefused` is a unit variant, so
/// there is no field in which a stop reason could be smuggled onto a slot that merely lost its turn.
#[test]
fn a_refusal_settles_only_its_own_slot() {
    let seed = ScheduleSeed::new(7);
    let inventory = AttemptInventory::frozen(seed).expect("the seed freezes");
    let pinned = PinnedArtifactIdentity::frozen().expect("the pinned constants are well formed");
    let attempts = inventory.attempts();
    let refused_key = attempts[REFUSED_INDEX];

    let refusal = ScreenRecord::not_run(
        refused_key,
        &pinned,
        seed,
        ORIGINAL_HAS_NO_SUPERSESSION,
        NotRunReason::EnvironmentRefused,
    )
    .expect("a refused original is recordable");

    let ScreenRecord::NotRun {
        key, supersession, ..
    } = &refusal
    else {
        panic!("a refused slot never entered acquisition, so it is NotRun");
    };
    assert_eq!(
        *key, refused_key,
        "a refusal settles the identity it refused and no other"
    );
    assert_eq!(
        *supersession,
        Supersession::Original,
        "this screen mints no retry, so a refused slot supersedes nothing"
    );

    let rendered = serde_json::to_string(&refusal).expect("a record serializes");
    assert!(
        rendered.contains("EnvironmentRefused"),
        "a refusal must name the reason that settled it, got {rendered}"
    );
    assert!(
        !rendered.contains("GateInoperable"),
        "a refusal must never serialize as the reason that stops the run, got {rendered}"
    );

    // The same slot, reached the other way: no verdict, so it and everything after it settle.
    let ungated = settle_remaining(
        &attempts[REFUSED_INDEX..],
        &pinned,
        seed,
        &NotRunReason::GateInoperable {
            diagnostic: DiagnosticArtifact::of_error(&anyhow!(UNGATED_DIAGNOSTIC)),
        },
    )
    .expect("every ungated slot settles");

    assert_eq!(
        ungated.len(),
        SCREEN_ATTEMPT_COUNT - REFUSED_INDEX,
        "an inoperable gate settles the current slot and every one after it"
    );
    assert!(
        ungated.len() > 1,
        "the comparison is only meaningful where the suffix is more than the refused slot itself"
    );
    let first = serde_json::to_string(&ungated[0]).expect("a record serializes");
    assert!(
        first.contains(UNGATED_DIAGNOSTIC),
        "an ungated slot must retain why the run stopped, got {first}"
    );
    assert!(
        !first.contains("EnvironmentRefused"),
        "a slot that was never gated must not claim the host was measured and found busy, got \
         {first}"
    );
}
