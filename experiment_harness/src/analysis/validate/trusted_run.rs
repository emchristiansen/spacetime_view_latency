//! One validated run: a role-tagged run coordinate and its complete ten-dose ladder.

use std::marker::PhantomData;

use crate::analysis::validate::integrity_error::IntegrityError;
use crate::analysis::validate::run_kind::RunKind;
use crate::analysis::validate::trusted_dose::TrustedDose;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::params::NUM_DOSES_USIZE;

/// One run of a matched block, its role lifted into the type parameter `R` (an
/// [`ArmRun`](super::arm_run::ArmRun) or a [`ControlRun`](super::control_run::ControlRun)). A
/// `TrustedRun<ArmRun>` and a `TrustedRun<ControlRun>` are distinct types, so a
/// [`MatchedBlock`](super::matched_block::MatchedBlock) cannot hold two arms or two controls — the
/// arm/control pairing is structural, not a naming convention.
///
/// The run is proven to carry exactly the preregistered dose ladder: `NUM_DOSES` (10) doses, each
/// ladder index `1..=NUM_DOSES` present exactly once, in monotonic order. The fixed array makes the
/// ten-dose cardinality a property of the type. The [`RunCoordinate`] is the production trusted identity
/// carrier (cell, role, repetition block).
///
/// Fields are private with no defaults; [`Self::mint`] is the sole `pub(super)` role-specific validator,
/// so a `TrustedRun<R>` can be assembled only from within the `validate` subtree and only when the
/// coordinate's recorded role matches `R`.
pub(crate) struct TrustedRun<R> {
    /// This run's schedule coordinate, proven to agree with its manifest reference and its doses.
    coordinate: RunCoordinate,
    /// The complete monotonic dose ladder, doses `1..=NUM_DOSES` each present exactly once, in order.
    /// Heap-owned as a boxed fixed array so the ten-dose cardinality remains a property of the type while
    /// the run's by-value footprint is one pointer — keeping the fold's stack frame bounded regardless of
    /// [`NUM_DOSES`](crate::params::NUM_DOSES).
    doses: Box<[TrustedDose; NUM_DOSES_USIZE]>,
    /// The compile-time role marker; carries no data, only the type-level arm/control distinction.
    role: PhantomData<R>,
}

impl<R: RunKind> TrustedRun<R> {
    /// The sole role-specific minting path: assemble a trusted run from a coordinate and its
    /// already-validated complete dose ladder, verifying that the coordinate's recorded role equals the
    /// type-level role `R::ROLE`. This is why the type-level role can never disagree with the recorded
    /// one — a mismatch is a typed [`IntegrityError::run_role_mismatch`], not a silently mislabelled run.
    /// The caller (the validation pass) owns proving dose completeness, uniqueness, and ladder
    /// monotonicity.
    pub(super) fn mint(
        coordinate: RunCoordinate,
        doses: Box<[TrustedDose; NUM_DOSES_USIZE]>,
    ) -> Result<Self, IntegrityError> {
        if coordinate.role() != R::ROLE {
            let diagnostic = format!(
                "run coordinate role {:?} does not match the {:?} position it was matched into",
                coordinate.role(),
                R::ROLE,
            );
            return Err(IntegrityError::run_role_mismatch(coordinate, R::ROLE, diagnostic));
        }
        Ok(Self {
            coordinate,
            doses,
            role: PhantomData,
        })
    }
}

/// Test-only read accessors for asserting the minted graph shape. `#[cfg(test)]` so they never widen
/// the production API — and role-agnostic (`impl<R>`, no [`RunKind`] bound) since reading the coordinate
/// and doses does not depend on the role marker.
#[cfg(test)]
impl<R> TrustedRun<R> {
    /// The run's schedule coordinate.
    pub(super) fn coordinate(&self) -> &RunCoordinate {
        &self.coordinate
    }

    /// The run's complete dose ladder.
    pub(super) fn doses(&self) -> &[TrustedDose; NUM_DOSES_USIZE] {
        &self.doses
    }
}
