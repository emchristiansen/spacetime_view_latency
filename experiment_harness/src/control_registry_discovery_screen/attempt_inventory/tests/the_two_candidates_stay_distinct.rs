//! Arm B's attempts are never minted under Arm A's candidate.

use crate::control_registry_discovery_screen::attempt_inventory::AttemptInventory;
use crate::control_registry_discovery_screen::candidate_id::CandidateId;
use crate::control_registry_discovery_screen::screen_rung::ScreenRung;
use crate::control_registry_discovery_screen::screen_target::ScreenTarget;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::plan::run_role::RunRole;

/// Coverage: the two candidates the screen measures stay separable in the ledger.
///
/// This is the reason `ControlActivityLatestByControlView` was added to the spec's `CandidateId`.
/// The registry is a deployable candidate and the procedural comparator is `DiagnosticOnly` and may
/// never support a production recommendation; they owe separate terminal dispositions, so evidence
/// minted under one candidate must never be readable as the other's. Attempt identity binds
/// candidate, so that separation is only real if the inventory actually assigns both.
///
/// Checks the full `candidate × role` product too, since that product is what
/// [`AttemptKey::target`](crate::control_registry_discovery_screen::attempt_key::AttemptKey::target)
/// inverts to recover the measured target — if two identities collapsed onto one target, an attempt
/// would time the wrong subscription.
#[test]
fn the_two_candidates_stay_distinct() {
    let frozen = AttemptInventory::frozen(ScheduleSeed::new(7)).expect("the seed freezes");
    let attempts = frozen.attempts();

    for candidate in [
        CandidateId::ControlRegistry,
        CandidateId::ControlActivityLatestByControlView,
    ] {
        let count = attempts
            .iter()
            .filter(|attempt| attempt.candidate() == candidate)
            .count();
        assert_eq!(
            count, 8,
            "{candidate:?} must own exactly eight of the sixteen attempts, got {count}"
        );
    }

    // Every target is reached, at every rung, exactly twice — once per block.
    for target in ScreenTarget::ALL {
        for rung in ScreenRung::ALL {
            let count = attempts
                .iter()
                .filter(|attempt| attempt.target() == target && attempt.rung() == rung)
                .count();
            assert_eq!(
                count, 2,
                "{target:?} at {rung:?} must run once per block, got {count}"
            );
        }
    }

    // The identity's candidate/role pair recovers the target it denotes, with no two identities
    // collapsing onto one target.
    for attempt in attempts {
        assert_eq!(
            attempt.target(),
            ScreenTarget::of(attempt.candidate(), attempt.role()),
            "an identity must denote exactly one measured target"
        );
        assert_eq!(attempt.target().candidate(), attempt.candidate());
        assert_eq!(attempt.target().role(), attempt.role());
    }

    // Arm A and Arm B are both Arms, and are distinguished only by candidate — the collision the
    // added CandidateId variant exists to prevent.
    assert_eq!(ScreenTarget::ArmA.role(), RunRole::Arm);
    assert_eq!(ScreenTarget::ArmB.role(), RunRole::Arm);
    assert_ne!(
        ScreenTarget::ArmA.candidate(),
        ScreenTarget::ArmB.candidate()
    );
}
