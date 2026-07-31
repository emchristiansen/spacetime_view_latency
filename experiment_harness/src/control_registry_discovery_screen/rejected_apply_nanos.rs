//! A completed timed apply's raw duration, retained but never readable back as a number.

use serde::Serialize;

/// The raw nanoseconds of a timed apply that completed but was not admitted as evidence.
///
/// **This type exists solely to be a visibility boundary.** Putting the `u128` directly in a
/// [`PartialEvidence`](super::partial_evidence::PartialEvidence) variant would not have retained
/// anything privately: Rust enum variant fields inherit the enum's visibility and cannot be made
/// independently private, so any crate code could have destructured
/// `RejectedSample { apply_nanos }` and recovered the value. A tuple struct with a private field in
/// its **own module** is the construct that actually enforces the boundary — `partial_evidence` is a
/// sibling module, so it can hold this value and serialize it while being unable to look inside it.
///
/// **What is and is not guaranteed.** No accessor exists and no field access compiles outside this
/// module, so there is *no typed path* by which a rejected duration reaches
/// [`ColdApplyEvidence::sealed`](super::cold_apply_evidence::ColdApplyEvidence::sealed), which takes
/// a bare `u128`. The guarantee is scoped to that typed API and no further: `Serialize` necessarily
/// renders the value, because retaining it in the ledger is the whole requirement, so it is exposed
/// externally by design.
///
/// **Neither `Debug` nor `PartialEq` is derived**, deliberately rather than by oversight.
/// Serialization is required; a `Debug` rendering is not, and deriving one would add an in-process
/// textual recovery path for nothing the ledger does not already provide. Withholding `PartialEq`
/// likewise denies recovery by comparison search. The absence of these impls *is* the enforcement —
/// documenting an escape hatch would not have closed it.
///
/// Construction is deliberately unrestricted: minting one is harmless, because a
/// `RejectedApplyNanos` can do nothing except be recorded. It is reading that is closed.
#[derive(Clone, Copy, Serialize)]
#[serde(transparent)]
pub(crate) struct RejectedApplyNanos(u128);

impl RejectedApplyNanos {
    /// Retain a completed apply's raw duration as non-evidence.
    ///
    /// Accepts zero without complaint, unlike
    /// [`ColdApplyEvidence::sealed`](super::cold_apply_evidence::ColdApplyEvidence::sealed): a
    /// nonpositive statistic invalidates its cell rather than vanishing, so the rejected zero is
    /// exactly the thing the ledger must show.
    pub(crate) fn of(apply_nanos: u128) -> Self {
        Self(apply_nanos)
    }
}
