//! A kind carries a series exactly when it follows the first append, and a mismatch exactly when it
//! is a composition mismatch.

use anyhow::anyhow;

use crate::indexed_sender_view_calibration_pilot::attempt_failure::AttemptFailure;
use crate::indexed_sender_view_calibration_pilot::composition_mismatch::CompositionMismatch;
use crate::indexed_sender_view_calibration_pilot::diagnostic_artifact::DiagnosticArtifact;
use crate::indexed_sender_view_calibration_pilot::failure_kind::FailureKind;
use crate::indexed_sender_view_calibration_pilot::partial_evidence::PartialEvidence;
use crate::indexed_sender_view_calibration_pilot::rejected_series::RejectedSeries;

/// Coverage: both evidence invariants, in both directions, over every kind.
///
/// Two earlier drafts were slack in exactly the directions this pins.
///
/// **Series.** The rule was a pair of implications, so `PacedBatch` — the kind that *names* the batch
/// stopping — could be recorded with `NothingObserved`, which means the batch was never reached. The
/// record then said both that the batch failed and that it never began, and `retained_samples()`
/// returned `None` under a kind that implies it ran. Partiality belongs in an empty or short
/// `RejectedSeries`, never in an absent one, so the empty container is asserted to be *accepted*
/// here — the fix must not have closed the hole by making a first-append failure unrecordable.
///
/// **Mismatch.** Only `Semantics ⇒ mismatch` was enforced, never the converse, so a `Sample` failure
/// could carry a composition mismatch — filing a sender-scope leak under a label that says the series
/// failed to seal. Both directions now hold.
#[test]
fn evidence_shape_is_a_biconditional_with_the_kind() {
    for kind in FailureKind::ALL {
        let diagnostic = || DiagnosticArtifact::of_error(&anyhow!("{kind:?} failed"));
        let empty_series = || RejectedSeries::of(Vec::new());
        let mismatch = || {
            CompositionMismatch::of(vec!["the arm holds a row owned by another identity".to_string()])
                .expect("the fault list is nonempty")
        };

        // No series retained.
        let nothing = AttemptFailure::observed(kind, PartialEvidence::NothingObserved, diagnostic());
        assert_eq!(
            nothing.is_ok(),
            !kind.requires_series(),
            "{kind:?}: NothingObserved is admissible exactly when the kind precedes the first append"
        );

        // A series retained, empty — the batch began and produced nothing.
        let series = AttemptFailure::observed(
            kind,
            PartialEvidence::RejectedSeries {
                series: empty_series(),
            },
            diagnostic(),
        );
        assert_eq!(
            series.is_ok(),
            kind.requires_series() && !kind.requires_observed_composition(),
            "{kind:?}: a retained series without a mismatch is admissible exactly for a \
             post-first-append kind that is not itself a composition mismatch"
        );

        // A series and a mismatch — only a semantic failure may say this.
        let with_mismatch = AttemptFailure::observed(
            kind,
            PartialEvidence::RejectedSeriesAndMismatch {
                series: empty_series(),
                mismatch: mismatch(),
            },
            diagnostic(),
        );
        assert_eq!(
            with_mismatch.is_ok(),
            kind.requires_observed_composition(),
            "{kind:?}: a composition mismatch is admissible exactly for the kind that is one"
        );
    }

    // The two cases the slack forms admitted, named so a regression is unmistakable.
    //
    // Asserted with `is_err` rather than `expect_err`: `AttemptFailure` deliberately has no `Debug`,
    // inherited from the raw samples it retains, and `expect_err` would require one. The test bends
    // rather than the type.
    assert!(
        AttemptFailure::observed(
            FailureKind::PacedBatch,
            PartialEvidence::NothingObserved,
            DiagnosticArtifact::of_error(&anyhow!("batch stopped")),
        )
        .is_err(),
        "PacedBatch must not claim the batch was never reached"
    );

    assert!(
        AttemptFailure::observed(
            FailureKind::Sample,
            PartialEvidence::RejectedSeriesAndMismatch {
                series: RejectedSeries::of(Vec::new()),
                mismatch: CompositionMismatch::of(vec!["leak".to_string()]).expect("nonempty"),
            },
            DiagnosticArtifact::of_error(&anyhow!("sealing failed")),
        )
        .is_err(),
        "Sample must not carry a composition mismatch"
    );
}
