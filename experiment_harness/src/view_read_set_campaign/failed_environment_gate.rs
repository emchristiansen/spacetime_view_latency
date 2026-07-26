//! A preflight gate reading that refused to launch its attempt.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because its whole content is
//! that its sole constructor recomputed the gate and found it failed. A private field is visible to
//! its declaring module **and every descendant**, so a `#[cfg(test)] mod tests` child — or any child
//! added later — could write the struct literal around a *passing* pair, producing a refusal whose
//! own evidence contradicts it. `sealed` has no children, so [`FailedEnvironmentGate::refused`]
//! really is the only door.

mod sealed {
    use serde::Serialize;

    use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;

    /// Gate evidence that provably fails the frozen mechanical gate.
    ///
    /// **Who can construct it.** Its field is private to this childless module and [`Self::refused`]
    /// is the only constructor, so no other module — sibling, parent, or elsewhere in the crate —
    /// can mint one. And `refused` is not a labelling function: it recomputes
    /// [`EnvironmentGateEvidence::passes`] and returns `None` when the gate held. A *passing* pair
    /// therefore cannot be stored under a rejection, which is the whole point of the type — a refusal
    /// that could carry evidence contradicting it would be worth nothing to a reader.
    ///
    /// **What it makes unrepresentable.** Paired with
    /// [`AttemptOutcome::PreflightRejected`](crate::view_read_set_campaign::attempt_outcome::AttemptOutcome::PreflightRejected),
    /// which carries no evidence field at all: "the preflight refused, but here is the measured
    /// evidence" is not a state that exists. That is the honest shape, because the gate ends before
    /// launch — there is nothing measured to retain.
    ///
    /// **What it does not claim.** That the readings are genuine, which is a property of the driver's
    /// sampling path. It claims exactly that these readings, whatever their provenance, fail the
    /// gate.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    #[serde(transparent)]
    pub(crate) struct FailedEnvironmentGate {
        evidence: EnvironmentGateEvidence,
    }

    impl FailedEnvironmentGate {
        /// The refusal these readings justify, or `None` if they in fact pass the gate.
        ///
        /// Total, and the sole constructor: there is no way to assert a refusal rather than derive
        /// one.
        pub(crate) fn refused(evidence: EnvironmentGateEvidence) -> Option<Self> {
            (!evidence.passes()).then_some(Self { evidence })
        }

        /// The two readings behind the refusal, so a reader can recompute which clause failed.
        pub(crate) fn evidence(self) -> EnvironmentGateEvidence {
            self.evidence
        }
    }
}

pub(crate) use sealed::FailedEnvironmentGate;
