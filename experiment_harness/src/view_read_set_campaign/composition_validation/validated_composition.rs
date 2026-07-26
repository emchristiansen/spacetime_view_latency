//! The artifact of having actually checked one attempt's observed result set.

use anyhow::Result;
use serde::Serialize;

use crate::view_read_set_campaign::composition_validation::expected_composition::ExpectedComposition;
use crate::view_read_set_campaign::composition_validation::observed_row_set::ObservedRowSet;

/// The record of a composition check that was performed, pointing at the rows it checked.
///
/// **Who can construct it.** Every field is private and [`Self::validate`] is the only constructor,
/// declared in this same file. No other module — sibling, parent, or elsewhere in the crate — can
/// build one, by struct literal or otherwise, so a composition finding cannot be asserted without
/// going through the comparison. Field privacy is what enforces this; a `pub(super)` constructor
/// would have been reachable from every sibling of this module.
///
/// **What it validates.** [`Self::validate`] receives the whole observed row set before and after
/// the measured mutation and checks: every row falls in one of the two preregistered key ranges;
/// each carries that range's expected owner and the fixed payload; the owned count is exact; the
/// foreign count matches
/// [`ExpectedForeignVisibility`](super::expected_foreign_visibility::ExpectedForeignVisibility),
/// which for the Arm is zero and is the candidate's security gate; and the mutated row changed
/// payload between the two observations, since the pinned SpacetimeDB source elides a byte-identical
/// update outright.
///
/// **What makes it auditable.** The two [`ObservedRowSet`]s are content-addressed artifacts on disk,
/// not summaries, so a reader can fetch the exact rows and re-run every comparison rather than
/// trusting that a validator once returned `Ok`. The embedded expectation, the per-range counts, and
/// the mutated row's key and both payloads are recorded so the check can be reproduced without
/// reading this module.
///
/// **What it does not claim.** It does not establish that the observations came from a real server;
/// that is a property of the driver's measurement path. It claims only that these specific retained
/// rows satisfy this specific recorded expectation.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ValidatedComposition {
    expected: ExpectedComposition,
    before: ObservedRowSet,
    after: ObservedRowSet,
    observed_owned_rows: u64,
    observed_foreign_rows: u64,
    mutation: MutationEvidence,
    delivered_rows: u64,
    client_cache_rows: u64,
    subscription_handles: u32,
}

/// The concrete before/after state of the row the measured mutation targeted.
///
/// Private to this file, so it cannot be assembled independently of the check that produced it. It
/// records the key and both payloads rather than asserting that a change was seen, so "the mutation
/// was visible" is something a reader verifies against two values instead of a flag they must
/// believe — and the two values are also in the retained row sets, so the record can be cross-checked.
#[derive(Debug, Clone, Serialize)]
struct MutationEvidence {
    entity_key: u64,
    payload_before: String,
    payload_after: String,
}

impl ValidatedComposition {
    /// Check the observed result sets against their expectation, minting the artifact only if every
    /// applicable gate holds.
    ///
    /// **Phase 1 boundary.** The comparison lands in Phase 2. The signature is what fixes which
    /// observations a finding must derive from: it takes the retained row sets rather than counts,
    /// so no caller can pre-reduce the evidence into something this function cannot contradict.
    pub(crate) fn validate(
        expected: ExpectedComposition,
        before: ObservedRowSet,
        after: ObservedRowSet,
        mutation_entity_key: u64,
        delivered_rows: u64,
        client_cache_rows: u64,
        subscription_handles: u32,
    ) -> Result<Self> {
        let _ = (
            expected,
            before,
            after,
            mutation_entity_key,
            delivered_rows,
            client_cache_rows,
            subscription_handles,
        );
        todo!(
            "Phase 2: census both retained row sets by preregistered key range, rejecting any row \
             outside both ranges, any row whose owner is not its range's expected identity, and any \
             row whose payload is not the fixed one; require the owned count to be exact and the \
             foreign count to match ExpectedForeignVisibility (zero for the Arm — the security \
             gate); and require the mutated row to be present in both observations with a changed \
             payload, since a byte-identical update is elided at the pinned commit"
        )
    }

    /// The expectation this finding was checked against, embedded so the artifact is self-contained.
    pub(crate) fn expected(&self) -> &ExpectedComposition {
        &self.expected
    }

    /// The content-addressed row sets this finding was derived from — where a reader goes to re-run
    /// the comparison.
    pub(crate) fn observations(&self) -> (&ObservedRowSet, &ObservedRowSet) {
        (&self.before, &self.after)
    }

    /// How many of the measured identity's own rows were observed.
    pub(crate) fn observed_owned_rows(&self) -> u64 {
        self.observed_owned_rows
    }

    /// How many foreign rows were observed. Zero is the Arm's passing security gate.
    pub(crate) fn observed_foreign_rows(&self) -> u64 {
        self.observed_foreign_rows
    }

    /// The mutated row's key and its payload before and after the measured write — the concrete
    /// values behind the visibility claim.
    pub(crate) fn mutation(&self) -> (u64, &str, &str) {
        (
            self.mutation.entity_key,
            &self.mutation.payload_before,
            &self.mutation.payload_after,
        )
    }

    /// Rows delivered to this subscriber, recorded as supporting evidence rather than a trend
    /// estimand.
    pub(crate) fn delivered_rows(&self) -> u64 {
        self.delivered_rows
    }

    /// Rows resident in the client cache, recorded as supporting evidence.
    pub(crate) fn client_cache_rows(&self) -> u64 {
        self.client_cache_rows
    }

    /// Live subscription handles, recorded as supporting evidence.
    pub(crate) fn subscription_handles(&self) -> u32 {
        self.subscription_handles
    }
}
