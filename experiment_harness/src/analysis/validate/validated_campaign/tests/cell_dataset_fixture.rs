//! A validation-owned single-[`CellDataset`] fixture for the dataset-driven projection gate proofs.
//!
//! It assembles one cell's complete 30-block matched sample directly through the same `pub(super)` mint
//! constructors the validation fold uses — [`TrustedDose::new`], [`TrustedRun::mint`],
//! [`MatchedBlock::new`], [`CellDataset::new`] — so a projection proof can drive
//! [`BetaDescriptor::project`](crate::analysis::beta::beta_descriptor::BetaDescriptor::project) over a
//! cell exhibiting a chosen arm response and control validity *without* threading a full 5940-record wire
//! campaign through the fold. This is the same fixture-building pattern as
//! [`StagedLadder`](super::staged_ladder) (which likewise mints trusted runs and doses from `pub(super)`
//! constructors); it widens no production constructor.
//!
//! Only the per-dose R-1 median latency varies between scenarios — a proof supplies it as a function of
//! `(block, role, dose)`. Every other trusted-content part (physical cardinalities, event evidence,
//! `logical_n` ladder, manifest collection-order sequence) is the deterministic value the classifier
//! either does not read or reads only as the fixed preregistered ladder x-axis, so a scenario's arm/control
//! response is a pure function of the medians it chooses.

use super::super::{NUM_DOSES_USIZE, REPETITION_BLOCKS_USIZE};
use crate::analysis::validate::arm_run::ArmRun;
use crate::analysis::validate::cell_dataset::CellDataset;
use crate::analysis::validate::control_run::ControlRun;
use crate::analysis::validate::matched_block::MatchedBlock;
use crate::analysis::validate::run_kind::RunKind;
use crate::analysis::validate::trusted_dose::TrustedDose;
use crate::analysis::validate::trusted_run::TrustedRun;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::manifest::repetition_block_index::RepetitionBlockIndex;
use crate::manifest::run_coordinate::RunCoordinate;
use crate::manifest::schedule_seed::ScheduleSeed;
use crate::manifest::validated_run_manifest::ValidatedRunManifest;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_sample::LatencySample;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;
use crate::observation::record_seq::RecordSeq;
use crate::params::BATCH_SIZE_USIZE;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// The sole handle the projection gate proofs use: a stateless assembler for one cell's dataset.
pub(crate) struct CellDatasetFixture;

impl CellDatasetFixture {
    /// The default arm/regime [`Cell`] the projection proofs classify — the canonical first cell. A gate
    /// proof cares only about the *response* it shapes, not which cell carries it, so a single canonical
    /// choice keeps every scenario comparable.
    pub(crate) fn cell() -> Cell {
        Cell::all()[0]
    }

    /// Assemble one cell's complete 30-block dataset in which block `b` (0-based), role `r`, dose `d`
    /// (0-based) carries R-1 median latency `median_of(b, r, d)` nanoseconds, in canonical collection order
    /// (construction block `b` sits at collection rank `b`). Each dose's `logical_n` is the canonical
    /// cumulative ladder value `(d + 1) · BATCH_SIZE`, so the classifier regresses each per-dose response
    /// against the preregistered strictly-increasing x-axis; the median is realized by a `BATCH_SIZE`-long
    /// constant latency vector, whose exact R-1 median is that constant.
    ///
    /// A gate proof that does not exercise collection ordering uses this canonical form, where evidence
    /// block index `i` is construction block `i`. A proof that *does* exercise ordering uses
    /// [`Self::build_with_collection_ranks`] to place blocks at a deliberately noncanonical rank.
    pub(crate) fn build(
        cell: Cell,
        median_of: impl Fn(usize, RunRole, usize) -> u128,
    ) -> CellDataset {
        Self::build_with_collection_ranks(cell, |block| block, median_of)
    }

    /// Assemble one cell's 30-block dataset exactly as [`Self::build`], but placing construction block `b`
    /// at collection rank `collection_rank_of(b)` rather than at rank `b`. The rank function must be a
    /// permutation of `0..REPETITION_BLOCKS`; block `b`'s arm manifest sequence becomes `2·rank` and its
    /// control's `2·rank + 1`, so the block's collection-order key (the earlier of the two) is `2·rank`,
    /// distinct across blocks. Construction (array) order stays `0..30` and each run coordinate keeps its
    /// construction block index; only the durable collection order is permuted — so a proof can drive a
    /// deliberately noncanonical collection order and check that both the primary evidence and the secondary
    /// descriptor independently sort by that same key.
    pub(crate) fn build_with_collection_ranks(
        cell: Cell,
        collection_rank_of: impl Fn(usize) -> usize,
        median_of: impl Fn(usize, RunRole, usize) -> u128,
    ) -> CellDataset {
        // Enforce the permutation precondition: every rank is in range and used exactly once, so the
        // block collection keys `2·rank` are genuinely distinct. A duplicate or out-of-range rank would
        // silently collapse two blocks onto one key and void the distinct-key alignment proof.
        let mut rank_seen = [false; REPETITION_BLOCKS_USIZE];
        let mut blocks: Vec<MatchedBlock> = Vec::with_capacity(REPETITION_BLOCKS_USIZE);
        for block in 0..REPETITION_BLOCKS_USIZE {
            let rank = collection_rank_of(block);
            assert!(rank < REPETITION_BLOCKS_USIZE, "collection rank {rank} is out of range");
            assert!(!rank_seen[rank], "collection rank {rank} is assigned to two blocks");
            rank_seen[rank] = true;
            let arm = mint_run::<ArmRun>(cell, RunRole::Arm, block, rank, &median_of);
            let control = mint_run::<ControlRun>(cell, RunRole::Control, block, rank, &median_of);
            blocks.push(MatchedBlock::new(arm, control));
        }
        assert!(
            rank_seen.into_iter().all(|seen| seen),
            "collection ranks must cover 0..REPETITION_BLOCKS exactly once"
        );
        let blocks: Box<[MatchedBlock; REPETITION_BLOCKS_USIZE]> = blocks
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly REPETITION_BLOCKS matched blocks were assembled for the cell");
        CellDataset::new(cell, blocks)
    }
}

/// Mint one role-typed trusted run for construction block `block` placed at collection `rank`: its ten
/// canonical-ladder doses, each carrying the scenario's chosen median `median_of(block, role, dose)`, under
/// a role-matched coordinate carrying the *construction* block index so [`TrustedRun::mint`]'s role check
/// passes. The manifest sequence is the run's collection-order anchor: `2·rank` for the arm and `2·rank + 1`
/// for the control, so the arm always precedes its control and the block sits at collection key `2·rank`.
fn mint_run<R: RunKind>(
    cell: Cell,
    role: RunRole,
    block: usize,
    rank: usize,
    median_of: &impl Fn(usize, RunRole, usize) -> u128,
) -> TrustedRun<R> {
    let coordinate = RunCoordinate::from_parts(
        cell,
        role,
        RepetitionBlockIndex::try_new(block as u32)
            .expect("a fixture block index is in 0..REPETITION_BLOCKS"),
    );
    let doses: Vec<TrustedDose> = DoseIndex::ALL
        .into_iter()
        .enumerate()
        .map(|(dose_zero_based, dose)| {
            let latencies = constant_latencies(median_of(block, role, dose_zero_based));
            let summary = LatencySummary::from_raw(&latencies);
            TrustedDose::new(
                dose,
                dose.cumulative_driving_rows(),
                latencies,
                summary,
                PhysicalCardinalities::expected(cell, dose),
                filler_events(),
            )
        })
        .collect();
    let doses: Box<[TrustedDose; NUM_DOSES_USIZE]> = doses
        .into_boxed_slice()
        .try_into()
        .ok()
        .expect("DoseIndex::ALL supplies exactly NUM_DOSES trusted doses");
    let manifest_seq = RecordSeq::new(2 * rank as u64 + role_offset(role));
    // `mint` derives the run coordinate from the manifest it is handed; a fixture manifest built *for* this
    // role-matched coordinate reports the matching role, so the role check passes. The manifest sequence is
    // still supplied separately as the run's collection-order anchor.
    let manifest = ValidatedRunManifest::fixture_for(coordinate.clone(), ScheduleSeed::new(0));
    TrustedRun::<R>::mint(&manifest, doses, manifest_seq)
        .expect("the coordinate role matches the type-level role R")
}

/// The within-block collection-order offset: the arm's manifest precedes its control's.
fn role_offset(role: RunRole) -> u64 {
    match role {
        RunRole::Arm => 0,
        RunRole::Control => 1,
    }
}

/// A full `BATCH_SIZE`-long latency vector of one repeated nanosecond value, so the dose's exact R-1
/// median equals that value. `BATCH_SIZE` is even, so the R-1 median is the mean of the two central
/// order statistics — both this constant — hence exactly it.
fn constant_latencies(median_nanos: u128) -> RawLatencies {
    let samples = (0..BATCH_SIZE_USIZE)
        .map(|_| LatencySample::from_nanos(median_nanos))
        .collect();
    RawLatencies::sealed(samples).expect("a constant dose carries exactly BATCH_SIZE samples")
}

/// An identity-consistent event stream: inserts − deletes equals the net delta. Deterministic filler the
/// classifier never reads, mirroring [`StagedLadder`](super::staged_ladder)'s.
fn filler_events() -> EventEvidence {
    EventEvidence::checked(3, 0, 1, 3)
        .expect("filler event counts satisfy the insert/delete/net-delta identity")
}
