//! A preflight gate reading that cleared its attempt to launch.

use serde::Serialize;

use crate::view_read_set_campaign::environment_gate_evidence::EnvironmentGateEvidence;

/// Gate evidence that provably passes the frozen mechanical gate.
///
/// **Why a clearance is recorded at all.** A refusal already reaches the ledger inside
/// [`AttemptOutcome::PreflightRejected`](super::attempt_outcome::AttemptOutcome::PreflightRejected).
/// Without this, a *launched* attempt would carry no gate readings whatsoever, and "launch only if
/// …" would be unauditable for exactly the attempts whose evidence is used. So the ledger records
/// one clearance per launched attempt, and the reconciliation invariant that follows is checkable:
/// an attempt with a measured terminal outcome has exactly one preflight clearance recorded before
/// it.
///
/// **Why not one type covering both verdicts.** A single record carrying raw
/// [`EnvironmentGateEvidence`] plus the attempt's disposition would state the refusal twice — here
/// and in the terminal outcome — and two copies of a derived verdict can disagree. Splitting the
/// pair means each reading appears in exactly one place, under the verdict it actually supports.
///
/// **Who can construct it.** Its field is private and [`Self::cleared`] is the only constructor,
/// declared in this file, so no other module — sibling, parent, or elsewhere in the crate — can mint
/// one. And `cleared` is not a labelling function: it recomputes
/// [`EnvironmentGateEvidence::passes`] and returns `None` when the gate refused. A *failing* pair
/// therefore cannot be stored under a clearance, exactly as a passing pair cannot be stored under
/// [`FailedEnvironmentGate`](super::failed_environment_gate::FailedEnvironmentGate).
///
/// **What it does not claim.** That the readings are genuine, which is a property of the driver's
/// sampling path; nor that the attempt actually launched, which is the terminal record's business.
/// It claims exactly that these readings, whatever their provenance, pass the gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(crate) struct PassedEnvironmentGate {
    evidence: EnvironmentGateEvidence,
}

impl PassedEnvironmentGate {
    /// The clearance these readings justify, or `None` if they in fact fail the gate.
    ///
    /// Total, and the sole constructor: there is no way to assert a clearance rather than derive one.
    pub(crate) fn cleared(evidence: EnvironmentGateEvidence) -> Option<Self> {
        evidence.passes().then_some(Self { evidence })
    }

    /// The two readings behind the clearance, so a reader can recompute every clause.
    pub(crate) fn evidence(self) -> EnvironmentGateEvidence {
        self.evidence
    }
}
