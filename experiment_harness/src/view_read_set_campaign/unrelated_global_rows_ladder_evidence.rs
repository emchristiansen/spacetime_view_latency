//! One `(candidate, block, role, version)`'s complete unrelated-global-rows ladder.
//!
//! The type lives in the private, *childless* inline module [`sealed`] because the array type carries
//! only the *count*; that the six are one ladder with each rung present exactly once is carried by
//! [`UnrelatedGlobalRowsLadderEvidence::sealed`] alone. A private field is visible to its declaring
//! module **and every descendant**, so a `#[cfg(test)] mod tests` child — or any child added later —
//! could write the struct literal around six selections of the same rung, or of six different
//! ladders, and record it as a whole ladder. `sealed` has no children, so that constructor really is
//! the only door.

mod sealed {
    use anyhow::Result;
    use serde::Serialize;

    use crate::view_read_set_campaign::campaign_params::UNRELATED_GLOBAL_ROWS_LADDER_LEN;
    use crate::view_read_set_campaign::reconciled_campaign::SelectedCompleteAttempt;

    /// The six selected attempts that make up one whole unrelated-global-rows ladder.
    ///
    /// **Why a ladder is a separate claim.** Every scale point is its own fresh server, so an
    /// attempt's evidence is one scale point and nothing more. But the estimand is the endpoint
    /// factor `T = S_last / S_first`, which reads two rungs of *one* ladder — so "these six belong
    /// together and are the whole of it" is a claim no per-attempt type makes, and it is this type's
    /// entire content.
    ///
    /// **Why exactly `[SelectedCompleteAttempt; 6]` and never a slice.** Two properties come free
    /// from the array that a slice would leave to a runtime check somebody must remember: the
    /// cardinality is the ladder's, fixed at compile time from
    /// [`UNRELATED_GLOBAL_ROWS_LADDER_LEN`] rather than written as a literal six, so a change to the
    /// frozen ladder moves this signature; and there is no partial ladder to represent. A
    /// `&[SelectedCompleteAttempt]` parameter would accept five, or seven, or six copies of one rung,
    /// and the sealing check would be the only thing standing between that and a recorded ladder.
    ///
    /// **Why the element type matters more than the array.** [`SelectedCompleteAttempt`] can only be
    /// minted by
    /// `ReconciledCampaign::selections`,
    /// so every element here is already known to be a complete, non-superseded, lowest-ordinal,
    /// provenance-admitted attempt. This type therefore does not re-derive any of that — it could not
    /// — and adds exactly one thing: that the six are one ladder, each rung once. A ladder assembled
    /// from hand-built evidence is not merely rejected here; it cannot be expressed, because its
    /// elements cannot be built.
    ///
    /// **All six are retained, not just the endpoints.** The classifier reads the first and last
    /// rungs, but the spec also requires curves and detection bounds to be shown, and each element
    /// carries its own
    /// [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance) —
    /// so keeping the whole ladder is what lets a reader see which runtime and instance produced
    /// every point without looking anything back up in the campaign this came from.
    ///
    /// **Phase 1 boundary.** The sealing checks are an explicit `todo!()`; what is fixed is the
    /// shape.
    #[derive(Debug, Clone, Serialize)]
    pub(crate) struct UnrelatedGlobalRowsLadderEvidence {
        rungs: [SelectedCompleteAttempt; UNRELATED_GLOBAL_ROWS_LADDER_LEN],
    }

    impl UnrelatedGlobalRowsLadderEvidence {
        /// Seal six selections into one ladder, failing loud unless they are one ladder with each
        /// rung present exactly once.
        ///
        /// **Phase 1 boundary.** The checks land in Phase 2 and are frozen in the `todo!()` below.
        pub(crate) fn sealed(
            rungs: [SelectedCompleteAttempt; UNRELATED_GLOBAL_ROWS_LADDER_LEN],
        ) -> Result<Self> {
            let _ = rungs;
            todo!(
                "Phase 2: require every selection to satisfy AttemptKey::same_ladder against the \
                 first — which is exactly the candidate, stage block, role, axis, and candidate \
                 version agreeing, and is reused rather than restated so the grouping the endpoint \
                 factor is computed over is defined in one place; require that shared axis to be \
                 ExperimentAxis::UnrelatedGlobalRows, since a ladder of another axis would be \
                 interpreted against the wrong endpoint factor F; and require the six \
                 ScalePoint::rung values to be the six frozen rungs with each occurring exactly \
                 once, so a repeated rung with another missing cannot pass on count alone. Retry \
                 ordinals may differ between rungs: each is its own slot's lowest, and requiring \
                 them equal would reject a legitimate ladder"
            )
        }

        /// Every rung of the ladder, retained in full so provenance and intermediate points travel
        /// with the endpoints.
        pub(crate) fn rungs(&self) -> &[SelectedCompleteAttempt; UNRELATED_GLOBAL_ROWS_LADDER_LEN] {
            &self.rungs
        }
    }
}

// This type is intended crate-visible Phase-1 surface — it is the input to the endpoint-factor
// classifier — but that later analysis layer does not exist, so nothing names it yet. It is exposed
// as a type alias rather than a `use` re-export for that reason: an unused `use` is an
// `unused_imports` warning, which must never be silenced, whereas an alias preserves the type and
// its associated functions identically and an unexercised one is ordinary dead code, already
// governed crate-wide by the skeleton's `#![allow(dead_code)]`.
pub(crate) type UnrelatedGlobalRowsLadderEvidence = sealed::UnrelatedGlobalRowsLadderEvidence;
