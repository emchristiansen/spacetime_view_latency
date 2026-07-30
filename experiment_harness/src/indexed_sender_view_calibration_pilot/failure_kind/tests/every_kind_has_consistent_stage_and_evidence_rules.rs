//! Every failure kind lands in one stage, and its evidence rules cannot contradict that stage.

use crate::indexed_sender_view_calibration_pilot::attempt_stage::AttemptStage;
use crate::indexed_sender_view_calibration_pilot::failure_kind::FailureKind;

/// Coverage: the cross-field rules that let a record shape be validated from a stage alone.
///
/// - **Exactly one stage per kind, and every stage reachable.** `AttemptStage::accepts` is defined as
///   `kind.stage() == self`, so totality is what makes "no failure kind may inhabit two shapes" a
///   checked property. An unreachable stage would be a record shape nothing can ever be filed under.
/// - **`requires_series` equals `can_follow_first_sample`.** They are equal today but they are
///   *different claims* — one about position relative to the measurement window, the other about what
///   evidence a record must carry — so `requires_series` is written out as its own exhaustive match
///   rather than derived. This assertion is what keeps them in step: move a kind's stage without
///   revisiting its evidence rule and this fails, instead of the evidence rule changing silently.
/// - **`requires_observed_composition` implies `can_observe_composition`.** A kind required to carry
///   a mismatch it could not have observed would be unconstructible in a way no compiler warns about.
#[test]
fn every_kind_has_consistent_stage_and_evidence_rules() {
    for kind in FailureKind::ALL {
        let stage = kind.stage();

        let accepting: Vec<AttemptStage> = AttemptStage::ALL
            .into_iter()
            .filter(|candidate| candidate.accepts(kind))
            .collect();
        assert_eq!(
            accepting,
            vec![stage],
            "{kind:?} must be admitted by exactly one stage"
        );

        assert_eq!(
            kind.can_follow_first_sample(),
            matches!(stage, AttemptStage::Bracketed | AttemptStage::Unbracketed),
            "{kind:?} must report sampling progress that agrees with its stage"
        );

        assert_eq!(
            kind.requires_series(),
            kind.can_follow_first_sample(),
            "{kind:?} must require a retained series exactly when it can follow the first append; \
             these are stated independently on purpose, and this is where they are held equal"
        );

        if kind.requires_observed_composition() {
            assert!(
                kind.can_observe_composition(),
                "{kind:?} requires an observed composition but cannot have observed one"
            );
        }
    }

    for stage in AttemptStage::ALL {
        assert!(
            FailureKind::ALL.into_iter().any(|kind| kind.stage() == stage),
            "{stage:?} is reachable by no failure kind, so it is a record shape nothing can fill"
        );
    }

    // The six kinds at or after batch entry, named explicitly. The assertion above ties
    // `requires_series` to a derived predicate; this ties it to the actual control-flow points, so a
    // stage remapping that moved both together would still be caught here.
    for kind in FailureKind::ALL {
        let at_or_after_batch_entry = matches!(
            kind,
            FailureKind::PacedBatch
                | FailureKind::HostObservationAfterBatchFailure
                | FailureKind::HostObservationAfter
                | FailureKind::WitnessSubscription
                | FailureKind::Sample
                | FailureKind::Semantics
        );
        assert_eq!(
            kind.requires_series(),
            at_or_after_batch_entry,
            "{kind:?} must require a series exactly when its control-flow point is at or after the \
             first append; note HostObservationBefore precedes the batch and must not"
        );
    }
}
