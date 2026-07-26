//! A preflight gate reading that refused to launch its attempt.

use serde::Serialize;

use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;

/// Gate evidence that provably fails the frozen mechanical gate.
///
/// **Who can construct it.** Its field is private and [`Self::refused`] is the only constructor,
/// declared in this file, so no other module — sibling, parent, or elsewhere in the crate — can mint
/// one. And `refused` is not a labelling function: it recomputes
/// [`EnvironmentGateEvidence::passes`] and returns `None` when the gate held. A *passing* pair
/// therefore cannot be stored under a rejection, which is the whole point of the type — a refusal
/// that could carry evidence contradicting it would be worth nothing to a reader.
///
/// **What it makes unrepresentable.** Paired with
/// [`AttemptOutcome::PreflightRejected`](super::attempt_outcome::AttemptOutcome::PreflightRejected),
/// which carries no evidence field at all: "the preflight refused, but here is the measured
/// evidence" is not a state that exists. That is the honest shape, because the gate ends before
/// launch — there is nothing measured to retain.
///
/// **What it does not claim.** That the readings are genuine, which is a property of the driver's
/// sampling path. It claims exactly that these readings, whatever their provenance, fail the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(crate) struct FailedEnvironmentGate {
    evidence: EnvironmentGateEvidence,
}

impl FailedEnvironmentGate {
    /// The refusal these readings justify, or `None` if they in fact pass the gate.
    ///
    /// Total, and the sole constructor: there is no way to assert a refusal rather than derive one.
    pub(crate) fn refused(evidence: EnvironmentGateEvidence) -> Option<Self> {
        (!evidence.passes()).then_some(Self { evidence })
    }

    /// The two readings behind the refusal, so a reader can recompute which clause failed.
    pub(crate) fn evidence(self) -> EnvironmentGateEvidence {
        self.evidence
    }
}
