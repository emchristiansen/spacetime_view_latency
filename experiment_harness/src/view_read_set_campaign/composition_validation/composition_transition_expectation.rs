//! The paired before/after expectation one attempt's final composition check is judged against.

use serde::Serialize;
use spacetimedb_sdk::Identity;

use crate::plan::run_role::RunRole;
use crate::view_read_set_campaign::composition_validation::expected_composition::ExpectedComposition;
use crate::view_read_set_campaign::scale_point::ScalePoint;

/// The two phase-matched expectations spanning an attempt's whole measured schedule: seeded before
/// any measured write, and the saturated channel's derived final state after the last one.
///
/// **Why the pair is a type.** A single [`ExpectedComposition`] names one phase, so handing one to a
/// validator alongside two differently-phased observations would apply that phase's rule to both —
/// judging the after-set as if nothing had been written, or the before-set as if everything had. Two
/// loose expectations would fix that but permit a different error: pairing the after-**E2**
/// expectation with the final after-**E1** row set. Both batches walk the same ten owned keys with
/// the same write indices and differ only in their channel tag, so that mismatch produces payloads
/// of exactly the right shape and the wrong tag — the kind of near-miss a hand-assembled pair
/// invites.
///
/// So the transition is minted as a whole. [`Self::required_final`] derives both sides from one attempt's
/// identity, role, and scale point, fixing `before` to
/// [`Seeded`](super::expected_payload_state::ExpectedPayloadState::Seeded) and `after` to the
/// **saturated** channel's schedule — which is the final state because E1 runs last in the frozen
/// execution order.
///
/// **Who can construct it.** Both fields are private and `required` is the only constructor,
/// declared in this file, so no other module can assemble a mismatched pair.
///
/// **What it does not cover.** E2's per-sample visibility. The paced channel stops each sample when
/// its change is observable in the subscriber cache, and that evidence lives in the paced channel's
/// own samples — not here. This transition is the *composition* claim spanning the attempt, not a
/// record of every intermediate state.
///
/// **This is validator input, not a finding.** It asserts nothing about what any server returned;
/// only [`ValidatedComposition`](super::validated_composition::ValidatedComposition) does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CompositionTransitionExpectation {
    before: ExpectedComposition,
    after: ExpectedComposition,
}

impl CompositionTransitionExpectation {
    /// Derive both phases of one attempt's composition expectation from its own identity.
    ///
    /// Named `required_final` rather than `required` because the *final* transition is the only one
    /// it mints: before any measured write, and after the last one. An intermediate transition would
    /// be a different constructor with a different name, so neither can be mistaken for the other at
    /// a call site.
    ///
    /// **Phase 1 boundary.** The derivation lands in Phase 2 together with
    /// [`ExpectedComposition::required`], which it calls twice.
    pub(crate) fn required_final(
        scale: ScalePoint,
        role: RunRole,
        owned_owner: Identity,
        foreign_owner: Identity,
    ) -> Self {
        let _ = (scale, role, owned_owner, foreign_owner);
        todo!(
            "Phase 2: call ExpectedComposition::required twice for this same scale, role, and pair \
             of identities — once with ExpectedPayloadState::Seeded for the before phase, and once \
             with AfterMeasuredBatch of MutationSchedule::of(SaturatedQueueGrowthPerWrite), which \
             is the final state because E1 runs last in the frozen execution order"
        )
    }

    /// What must be observed before any measured write: the seeded payload on every row.
    pub(crate) fn before(&self) -> &ExpectedComposition {
        &self.before
    }

    /// What must be observed after the saturated batch confirms: the schedule's derived final
    /// payload on each owned key, the seeded payload still on every foreign row.
    pub(crate) fn after(&self) -> &ExpectedComposition {
        &self.after
    }

    /// The scale point both phases describe. `required` derives them from one scale point, so this
    /// is unambiguous.
    pub(crate) fn scale(&self) -> ScalePoint {
        self.before.scale()
    }
}
