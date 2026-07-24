//! One arm block's search outcome bound to its schedule-proven collection-order key.

use crate::analysis::beta::block_search_outcome::BlockSearchOutcome;
use crate::observation::record_seq::RecordSeq;

/// A [`BlockSearchOutcome`] tagged with the collection-order key of the block that produced it —
/// [`BetaDescriptor`](super::beta_descriptor::BetaDescriptor)'s sole per-block storage element. Binding
/// the key into the value makes it non-separable from its outcome *after* minting, so the secondary
/// descriptor joins to the primary evidence by the explicit shared key rather than by array position.
/// Correct *initial* pairing is not enforced by this type — [`Self::new`] accepts any key/outcome pair —
/// but rests on the single fit-minting site in
/// [`BetaDescriptor::fit`](super::beta_descriptor::BetaDescriptor::fit) plus the adversarial key-equality
/// proof.
///
/// [`Self::new`] is `pub(super)`, so a keyed outcome is minted only within the `beta` module. Not
/// `Serialize`: the wrapped outcome holds the non-`Serialize`
/// [`BetaCandidate`](super::beta_candidate::BetaCandidate) boundary; the report renders the key alongside
/// the projected fit.
#[derive(Debug, Clone)]
pub(crate) struct KeyedBlockOutcome {
    /// The collection-order key of the block that produced `outcome`.
    collection_order_key: RecordSeq,
    /// The block's complete search outcome (fit bound to its convergence record).
    outcome: BlockSearchOutcome,
}

impl KeyedBlockOutcome {
    /// Bind a block's search outcome to its collection-order key. `pub(super)` so only the `beta` module
    /// mints one.
    pub(super) fn new(collection_order_key: RecordSeq, outcome: BlockSearchOutcome) -> Self {
        Self {
            collection_order_key,
            outcome,
        }
    }

    /// The block's collection-order key — the explicit join key the report serializes.
    pub(crate) fn collection_order_key(&self) -> RecordSeq {
        self.collection_order_key
    }

    /// The block's complete search outcome.
    pub(crate) fn outcome(&self) -> &BlockSearchOutcome {
        &self.outcome
    }
}
