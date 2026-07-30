//! Every failure kind lands in one stage, and its evidence rules cannot contradict that stage.

use crate::indexed_sender_view_calibration_pilot::attempt_stage::AttemptStage;
use crate::indexed_sender_view_calibration_pilot::failure_kind::FailureKind;

/// Coverage: the cross-field implications that let a record shape be validated from a stage alone.
///
/// Four properties, each of which a record constructor relies on rather than re-deriving:
///
/// - **Exactly one stage per kind, and every stage reachable.** `AttemptStage::accepts` is defined as
///   `kind.stage() == self`, so totality is what makes "no failure kind may inhabit two shapes" a
///   checked property. An unreachable stage would mean a record shape nothing can ever be filed
///   under.
/// - **`requires_series` implies `permits_series`.** A kind that must retain samples but is forbidden
///   to carry them is unconstructible — `AttemptFailure::observed` would reject both branches, so the
///   kind would be dead in a way no compiler warns about.
/// - **`requires_observed_composition` implies `can_observe_composition`.** Same shape: a kind
///   required to carry a mismatch it is forbidden to have observed cannot be built.
/// - **Sampling progress agrees with the stage.** `can_follow_first_sample` is derived from the stage
///   precisely so a ledger line cannot say a failure struck before the first append while sitting in
///   a shape that only exists after one.
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

        if kind.requires_series() {
            assert!(
                kind.permits_series(),
                "{kind:?} requires a retained series but is not permitted one, so no failure of \
                 this kind could ever be constructed"
            );
        }
        if kind.requires_observed_composition() {
            assert!(
                kind.can_observe_composition(),
                "{kind:?} requires an observed composition but cannot have observed one"
            );
        }

        assert_eq!(
            kind.can_follow_first_sample(),
            matches!(stage, AttemptStage::Bracketed | AttemptStage::Unbracketed),
            "{kind:?} must report sampling progress that agrees with its stage"
        );
    }

    for stage in AttemptStage::ALL {
        assert!(
            FailureKind::ALL.into_iter().any(|kind| kind.stage() == stage),
            "{stage:?} is reachable by no failure kind, so it is a record shape nothing can fill"
        );
    }
}
