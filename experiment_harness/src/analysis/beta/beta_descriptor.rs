//! The complete per-cell secondary descriptor, the exhaustive primary→stored projection that gates it,
//! and the module-private raw fit it guards.

use crate::analysis::beta::beta_population::BetaPopulation;
use crate::analysis::beta::block_fit::BlockFit;
use crate::analysis::beta::block_point::BlockPoint;
use crate::analysis::beta::block_search_outcome::BlockSearchOutcome;
use crate::analysis::beta::fit_block::fit_block;
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
/// ([`Self::from_ladders`]) are module-private; this module's child `tests` submodule drives the fit over
/// synthetic ladders through Rust's child privacy, so no sibling of `beta_descriptor` can mint one.
///
/// It stores **only** the 30 block outcomes. The population β interval is *derived* on demand by
/// [`Self::population`] (`Some` exactly when every block is identifiable), so it can never disagree with
/// the outcomes it summarizes. The blocks are ordered by the same
/// [`collection_order_key`](crate::analysis::validate::matched_block::MatchedBlock::collection_order_key)
/// as the primary evidence arrays, so block index `i` denotes the same repetition block across the
/// primary and secondary results.
#[derive(Debug)]
pub(crate) struct BetaDescriptor {
    /// The 30 per-block exponent-fit outcomes, in schedule-proven collection order. Heap-owned as a
    /// boxed fixed array, mirroring the trusted graph's heap-first fixed arrays.
    blocks: Box<[BlockFit; N_BLOCKS]>,
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
        let mut keyed: Vec<(RecordSeq, [BlockPoint; NUM_DOSES_USIZE])> = dataset
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
        keyed.sort_by_key(|(key, _)| *key);

        let ladders: Vec<[BlockPoint; NUM_DOSES_USIZE]> =
            keyed.into_iter().map(|(_, ladder)| ladder).collect();
        let ladders: [[BlockPoint; NUM_DOSES_USIZE]; N_BLOCKS] = ladders
            .try_into()
            .ok()
            .expect("a cell dataset carries exactly REPETITION_BLOCKS arm blocks");
        Self::from_ladders(&ladders)
    }

    /// Fit each of the 30 already-ordered ten-dose ladders. Module-private (no visibility modifier), so it
    /// is reachable only from within this module — in production solely through [`Self::fit`], and in
    /// tests through this module's child [`tests`] submodule, which Rust's child privacy lets read its
    /// parent's private items. No sibling under `analysis::beta` can mint a descriptor off a bare ladder.
    fn from_ladders(ladders: &[[BlockPoint; NUM_DOSES_USIZE]; N_BLOCKS]) -> Self {
        let blocks: Vec<BlockFit> = ladders.iter().map(fit_block).collect();
        let blocks: Box<[BlockFit; N_BLOCKS]> = blocks
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly REPETITION_BLOCKS ladders fit into exactly REPETITION_BLOCKS outcomes");
        Self { blocks }
    }

    /// The 30 per-block exponent-fit outcomes, in schedule-proven collection order.
    pub(crate) fn blocks(&self) -> &[BlockFit; N_BLOCKS] {
        &self.blocks
    }

    /// The 30 per-block *search* outcomes — each block's authoritative [`BlockFit`] bound to the complete
    /// [`BlockConvergence`](super::block_convergence::BlockConvergence) record of the search that produced
    /// it — in schedule-proven collection order. The report projects these into its typed all-basin
    /// convergence section (spec: "retain a typed per-block search summary ... Do not report only the
    /// winning basin").
    ///
    /// Phase-1 additive accessor: a `todo!()` signature with no backing field yet. The intended Phase-2
    /// migration stores `[BlockSearchOutcome; 30]` as the *sole* per-block field (each outcome carrying
    /// its authoritative [`BlockFit`] bound to its [`BlockConvergence`](super::block_convergence::BlockConvergence)
    /// by [`BlockSearchOutcome::mint`], so a fit can never be paired with a different block's convergence).
    /// [`Self::blocks`] then cannot keep returning `&[BlockFit; 30]` — that borrowed array cannot be
    /// synthesized from an `[BlockSearchOutcome; 30]` without duplicate storage or self-referential
    /// caching — so `blocks()` becomes an indexed/iterator accessor
    /// (`fn block_fits(&self) -> impl Iterator<Item = BlockFit>`, [`BlockFit`] being `Copy`) and its
    /// callers migrate to it. Outcomes-as-sole-storage with an iterator is preferred over parallel
    /// `[BlockFit; 30]` + `[BlockConvergence; 30]` arrays because it makes a fit/convergence mismatch
    /// structurally unrepresentable rather than merely constructor-checked. Wiring it now would require
    /// [`fit_block`](super::fit_block) to emit the convergence record, a behavior change deferred out of
    /// this compile-green skeleton.
    pub(crate) fn block_search_outcomes(&self) -> &[BlockSearchOutcome; N_BLOCKS] {
        todo!("Phase 2: store [BlockSearchOutcome; 30] as the sole per-block field; blocks() becomes an iterator")
    }

    /// The population β interval over all 30 blocks, derived from the block outcomes: `Some` exactly when
    /// every block is identifiable (spec: "may emit a population interval ... only when every block is
    /// identifiable"), `None` otherwise. Derived, not stored, so it cannot drift from the outcomes.
    pub(crate) fn population(&self) -> Option<BetaPopulation> {
        if !self.blocks.iter().all(|fit| fit.is_identifiable()) {
            return None;
        }
        let betas: Vec<f64> = self
            .blocks
            .iter()
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
