//! The complete per-cell secondary descriptor, the exhaustive primary→stored projection that gates it,
//! and the module-private raw fit it guards.

use crate::analysis::beta::beta_population::BetaPopulation;
use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::block_point::BlockPoint;
use crate::analysis::beta::fit_block::fit_block;
use crate::analysis::beta::keyed_block_outcome::KeyedBlockOutcome;
use crate::analysis::classify::cell_classification::CellClassification;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::classify::classify_cell::classify_cell;
use crate::analysis::classify::non_increasing_response::NonIncreasingResponse;
use crate::analysis::classify::response_class::ResponseClass;
use crate::analysis::validate::cell_dataset::CellDataset;
use crate::observation::record_seq::RecordSeq;
use crate::params::{NUM_DOSES_USIZE, REPETITION_BLOCKS};

/// The fixed 30-block per-cell sample size as an array length, so the "report all 30 block outcomes"
/// obligation is compiler-visible rather than a runtime count.
const N_BLOCKS: usize = REPETITION_BLOCKS as usize;

/// The secondary `L(N) = a + bN^β` descriptor for one Increasing cell: the exponent fit outcome of each
/// of its 30 arm blocks, in schedule-proven collection order (spec: "report all 30 block outcomes and
/// preserve non-identifiable failures").
///
/// β is *computed* only for a gated Increasing cell, not merely *stored* only there. The raw fit over a
/// dataset ([`Self::fit`]) is module-private, and no crate-visible API takes a bare `&CellDataset` and
/// returns a `BetaDescriptor`. The only crate-visible entry is [`Self::project`], the exhaustive
/// projection of the exact primary [`CellClassification`], which invokes the raw fit solely in its
/// matched [`ResponseClass::Increasing`] arm (spec: "implement and invoke the ... descriptor only for
/// cells whose gated primary result is Increasing"). Both the raw fit and the ladder fit it delegates to
/// ([`Self::from_keyed_ladders`]) are module-private; this module's child `tests` submodule drives the fit over
/// synthetic ladders through Rust's child privacy, so no sibling of `beta_descriptor` can mint one.
///
/// It stores **only** the 30 per-block [`KeyedBlockOutcome`]s — each an authoritative [`BlockFit`] bound
/// to the complete [`BlockConvergence`](super::block_convergence::BlockConvergence) record of the search
/// that produced it, tagged with that block's
/// [`collection_order_key`](crate::analysis::validate::matched_block::MatchedBlock::collection_order_key).
/// This single field is the sole per-block storage: the authoritative fits are *derived* from it by
/// [`Self::block_fits`] rather than duplicated, so a fit can never disagree with its convergence record,
/// and the population β interval is *derived* on demand by [`Self::population`] (`Some` exactly when every
/// block is identifiable), so it can never disagree with the outcomes it summarizes. The blocks are
/// ordered by the same collection-order key as the primary evidence arrays, and each outcome carries that
/// key explicitly, so block index `i` denotes the same repetition block across the primary and secondary
/// results and the report join is by the serialized key rather than by array position.
#[derive(Debug)]
pub(crate) struct BetaDescriptor {
    /// The 30 per-block key-tagged search outcomes (collection-order key + fit + convergence record), in
    /// schedule-proven collection order — the sole per-block storage. Heap-owned as a boxed fixed array,
    /// mirroring the trusted graph's heap-first fixed arrays.
    blocks: Box<[KeyedBlockOutcome; N_BLOCKS]>,
}

impl BetaDescriptor {
    /// Classify one cell's dataset and exhaustively project the exact primary [`CellClassification`] into
    /// the stored [`ClassifiedCell`]. This is the sole crate-visible path that can mint a descriptor, and
    /// it takes **only** the `&CellDataset`: it computes the primary classification from that same dataset
    /// via [`classify_cell`], so a cell's primary result can never be paired with a foreign dataset. The
    /// non-Increasing and invalid arms construct their stored variants without fitting, and only the
    /// [`ResponseClass::Increasing`] arm invokes the module-private raw fit over the same dataset. So the
    /// secondary descriptor is computed only for a gated Increasing cell, and the primary-to-secondary
    /// gate is unrepresentable in the stored result.
    pub(crate) fn project(dataset: &CellDataset) -> ClassifiedCell {
        match classify_cell(dataset) {
            CellClassification::InvalidControl { evidence } => {
                ClassifiedCell::InvalidControl { evidence }
            }
            CellClassification::Valid {
                evidence,
                response,
                comparison,
            } => match response {
                ResponseClass::Increasing => ClassifiedCell::Increasing {
                    evidence,
                    comparison,
                    beta: Self::fit(dataset),
                },
                ResponseClass::FlatEquivalent => ClassifiedCell::NonIncreasing {
                    evidence,
                    response: NonIncreasingResponse::FlatEquivalent,
                    comparison,
                },
                ResponseClass::Decreasing => ClassifiedCell::NonIncreasing {
                    evidence,
                    response: NonIncreasingResponse::Decreasing,
                    comparison,
                },
                ResponseClass::Inconclusive => ClassifiedCell::NonIncreasing {
                    evidence,
                    response: NonIncreasingResponse::Inconclusive,
                    comparison,
                },
            },
        }
    }

    /// Fit every arm block of a cell's dataset. Module-private (no visibility modifier), so it is
    /// reachable only from within this module — in production solely through [`Self::project`]'s
    /// Increasing arm. Each block's ten-dose ladder is the arm run's `(logical_n, median_nanos)` points
    /// (spec: the descriptor fits the arm's raw per-dose latency, its free `a ≥ 0` absorbing the
    /// confirmed-read floor); blocks are ordered by the collection-order key so this descriptor's block
    /// index aligns with the primary evidence's.
    fn fit(dataset: &CellDataset) -> Self {
        let keyed: Vec<(RecordSeq, [BlockPoint; NUM_DOSES_USIZE])> = dataset
            .blocks()
            .iter()
            .map(|block| {
                let doses = block.arm().doses();
                let ladder: [BlockPoint; NUM_DOSES_USIZE] = std::array::from_fn(|dose_index| {
                    let dose = &doses[dose_index];
                    BlockPoint::from_measurements(dose.logical_n(), dose.median_nanos())
                });
                (block.collection_order_key(), ladder)
            })
            .collect();
        // No sort here: `from_keyed_ladders` centralizes the collection-order ordering invariant for every
        // constructor path, so this only has to derive each block's key from its matched block.
        let keyed: [(RecordSeq, [BlockPoint; NUM_DOSES_USIZE]); N_BLOCKS] = keyed
            .try_into()
            .ok()
            .expect("a cell dataset carries exactly REPETITION_BLOCKS arm blocks");
        Self::from_keyed_ladders(&keyed)
    }

    /// Fit 30 key-tagged ten-dose ladders into the sole per-block storage — the single site that mints a
    /// [`KeyedBlockOutcome`], and the one place the collection-order ordering invariant is enforced. It
    /// copies the already-keyed pairs, sorts them by collection-order key *without separating payloads*
    /// (so each fit stays bound to its own key), and requires strictly increasing adjacent keys — a
    /// duplicate key, which would collapse two blocks onto one identity, fails loudly here rather than
    /// silently. It then fits and mints in that proven order, so every stored descriptor is in ascending
    /// collection order regardless of the caller's array order.
    ///
    /// Module-private (no visibility modifier): reachable only within this module — in production through
    /// [`Self::fit`], which derives every key from a matched block's `collection_order_key`, and in tests
    /// through this module's child [`tests`] submodule, whose callers supply intentional keys. The helper
    /// never fabricates a key, so a block's identity always originates with its caller.
    fn from_keyed_ladders(
        keyed: &[(RecordSeq, [BlockPoint; NUM_DOSES_USIZE]); N_BLOCKS],
    ) -> Self {
        let mut ordered: Vec<(RecordSeq, [BlockPoint; NUM_DOSES_USIZE])> = keyed.to_vec();
        ordered.sort_by_key(|(key, _)| *key);
        for adjacent in ordered.windows(2) {
            assert!(
                adjacent[0].0 < adjacent[1].0,
                "block collection-order keys must be strictly increasing; a duplicate key ({:?}) \
                 would collapse two blocks onto one identity",
                adjacent[0].0
            );
        }
        let blocks: Vec<KeyedBlockOutcome> = ordered
            .into_iter()
            .map(|(key, ladder)| KeyedBlockOutcome::new(key, fit_block(&ladder)))
            .collect();
        let blocks: Box<[KeyedBlockOutcome; N_BLOCKS]> = blocks
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect(
                "exactly REPETITION_BLOCKS keyed ladders fit into exactly REPETITION_BLOCKS outcomes",
            );
        Self { blocks }
    }

    /// The 30 per-block key-tagged search outcomes — each block's authoritative [`BlockFit`] bound to the
    /// complete [`BlockConvergence`](super::block_convergence::BlockConvergence) record of the search that
    /// produced it and tagged with the block's collection-order key — in schedule-proven collection order.
    /// This is the descriptor's sole per-block storage; the report projects these into its typed all-basin
    /// convergence section together with the explicit key (spec: "retain a typed per-block search summary
    /// ... Do not report only the winning basin").
    pub(crate) fn block_search_outcomes(&self) -> &[KeyedBlockOutcome; N_BLOCKS] {
        &self.blocks
    }

    /// The 30 per-block authoritative exponent-fit outcomes, in schedule-proven collection order, derived
    /// from the sole [`Self::block_search_outcomes`] storage rather than stored separately — so a fit can
    /// never disagree with its convergence record. [`BlockFit`] is `Copy`, so this yields values, not
    /// borrows; a consumer needing indexed access reads `block_search_outcomes()[i].outcome().fit()`.
    pub(crate) fn block_fits(&self) -> impl Iterator<Item = BlockFit> + '_ {
        self.blocks.iter().map(|keyed| keyed.outcome().fit())
    }

    /// The population β interval over all 30 blocks, derived from the block outcomes: `Some` exactly when
    /// every block is identifiable (spec: "may emit a population interval ... only when every block is
    /// identifiable"), `None` otherwise. Derived, not stored, so it cannot drift from the outcomes.
    pub(crate) fn population(&self) -> Option<BetaPopulation> {
        if !self.block_fits().all(|fit| fit.is_identifiable()) {
            return None;
        }
        let betas: Vec<f64> = self
            .block_fits()
            .map(|fit| {
                fit.identified_beta()
                    .expect("every block is identifiable in this branch")
            })
            .collect();
        let betas: [f64; N_BLOCKS] = betas
            .try_into()
            .ok()
            .expect("exactly REPETITION_BLOCKS identified exponents");
        Some(BetaPopulation::from_identified(&betas))
    }
}

#[cfg(test)]
mod tests;
