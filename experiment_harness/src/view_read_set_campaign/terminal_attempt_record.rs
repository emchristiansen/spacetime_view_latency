//! One attempt's identity bound to the single terminal outcome it produced.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because the pairing check is
//! the type's entire content. A private field is visible to its declaring module **and every
//! descendant**, so a `#[cfg(test)] mod tests` child, or any child added later, could write the
//! struct literal and file one attempt's evidence under another attempt's identity — the exact
//! mismatch [`TerminalAttemptRecord::sealed`] exists to refuse. `sealed` has no children, so that
//! constructor really is the only door.

mod sealed {
    use anyhow::{ensure, Result};
    use serde::Serialize;

    use crate::view_read_set_campaign::attempt_key::AttemptKey;
    use crate::view_read_set_campaign::attempt_outcome::AttemptOutcome;
    use crate::view_read_set_campaign::retry_eligibility::RetryEligibility;

    /// An attempt identity and its terminal outcome, proven to describe each other.
    ///
    /// **Why the pair is a type.** A detached `(AttemptKey, AttemptOutcome)` lets any key be read
    /// against any outcome — so a rule that consults the key's retry ordinal, or the axis its
    /// evidence must match, could be applied to a different attempt's disposition entirely. Every
    /// such rule therefore takes this record instead, and the pairing is checked once, here, rather
    /// than assumed at each of them.
    ///
    /// **Who can construct it.** Both fields are private to this childless module and
    /// [`Self::sealed`] is the only constructor. No other module — sibling, parent, or elsewhere in
    /// the crate — can pair a key with an outcome without passing the same-record invariants.
    ///
    /// **What sealing proves.** That the outcome's evidence, where it has any, was measured at
    /// exactly the scale point the key names, and under the axis the key names. A `Complete` outcome
    /// carrying another rung's evidence, or a `Failed` outcome whose partial evidence came from
    /// another scale point, is refused. [`AttemptOutcome::PreflightRejected`] and
    /// [`AttemptOutcome::NotRun`] have no evidence at all, so for them the pairing is unconstrained —
    /// correctly, since there is nothing that could disagree.
    ///
    /// **What it does not prove.** That this is the *only* terminal record for the slot, or that no
    /// lower retry ordinal exists elsewhere in the ledger. Those are facts about a reconciled ledger
    /// view, not about one pair, and any selection rule needs that view rather than this record
    /// alone.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct TerminalAttemptRecord {
        key: AttemptKey,
        outcome: AttemptOutcome,
    }

    impl TerminalAttemptRecord {
        /// Bind one attempt's identity to its terminal outcome, failing loud unless the outcome's
        /// evidence belongs to that identity.
        pub(crate) fn sealed(key: AttemptKey, outcome: AttemptOutcome) -> Result<Self> {
            match &outcome {
                AttemptOutcome::Complete { artifact, .. } => {
                    let measured = artifact.scale_point_evidence().scale();
                    ensure!(
                        measured == key.scale(),
                        "the complete artifact was measured at {measured:?} but its attempt \
                         identity names {:?}",
                        key.scale(),
                    );
                    ensure!(
                        artifact.axis() == key.scale().axis(),
                        "the complete artifact's axis {:?} disagrees with its attempt identity's \
                         {:?}",
                        artifact.axis(),
                        key.scale().axis(),
                    );
                }
                AttemptOutcome::Failed { partial, .. } => {
                    ensure!(
                        partial.scale() == key.scale(),
                        "the retained partial evidence was measured at {:?} but its attempt \
                         identity names {:?}",
                        partial.scale(),
                        key.scale(),
                    );
                }
                // Neither carries evidence, so neither can contradict the identity it is paired
                // with.
                AttemptOutcome::PreflightRejected { .. } | AttemptOutcome::NotRun { .. } => {}
            }
            Ok(Self { key, outcome })
        }

        /// The attempt's full identity, retry ordinal included.
        pub(crate) fn key(&self) -> AttemptKey {
            self.key
        }

        /// The single terminal disposition this attempt produced.
        pub(crate) fn outcome(&self) -> &AttemptOutcome {
            &self.outcome
        }

        /// Whether the spec's retry rule permits one more attempt at this record's logical slot.
        ///
        /// A method on the bound record rather than a free function, so the retry ordinal it reads
        /// is necessarily the ordinal of the attempt that produced the outcome it reads.
        ///
        /// **Phase 1 boundary.** Every input the rule needs is now in scope — the outcome's variant,
        /// the failure cause, the driver-stated
        /// [`FailureStage`](crate::view_read_set_campaign::failure_stage::FailureStage) that
        /// [`AttemptOutcome::Failed`] carries, and this record's own
        /// [`RetryOrdinal`](crate::view_read_set_campaign::retry_ordinal::RetryOrdinal) — so the
        /// decision is implementable here and is deferred to Phase 2 rather than to a missing type.
        ///
        /// The timing term is *derived*, not stored: there is no
        /// [`MeasuredSampleBoundary`](crate::view_read_set_campaign::measured_sample_boundary::MeasuredSampleBoundary)
        /// field to read, only
        /// [`FailureStage::measured_sample_boundary`](crate::view_read_set_campaign::failure_stage::FailureStage::measured_sample_boundary),
        /// so the boundary cannot disagree with the stage it came from.
        pub(crate) fn retry_eligibility(&self) -> RetryEligibility {
            todo!(
                "Phase 2: total-match the outcome — PreflightRejected is categorically eligible \
                 because the prospective gate ends before the first measured sample; Failed with \
                 FailureKind::Infrastructure is eligible only when its stage's derived \
                 FailureStage::measured_sample_boundary is MeasuredSampleBoundary::BeforeFirst; \
                 Application, Timeout, NonpositiveStatistic and SemanticsOrSecurity are \
                 categorically ineligible as measured outcomes; Complete and NotRun have nothing to \
                 retry. Conjoin with this record's retry ordinal being RetryOrdinal::ORIGINAL, since \
                 at most one retry is permitted per logical slot"
            )
        }
    }
}

pub(crate) use sealed::TerminalAttemptRecord;
