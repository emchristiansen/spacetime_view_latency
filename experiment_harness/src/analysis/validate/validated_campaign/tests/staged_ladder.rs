//! The single shared helper for the direct category proofs: a full canonical dose-slot grid for the two
//! ladder proofs (which drive the private `validate_ladders`) plus a complete trusted-dose ladder for the
//! run-role proof (which drives `TrustedRun::mint`).
//!
//! `validate_ladders` reads only each staged dose's `logical_n`, and `mint` checks the coordinate role
//! before its doses matter, so the non-`logical_n` content is deterministic-but-irrelevant filler whose
//! sole job is to satisfy the trusted content types' constructors.

use super::super::{
    dose_slot_index, role_index, run_slot_index, StagedDose, CELL_COUNT, DOSE_SLOTS, NUM_DOSES_USIZE,
    REPETITION_BLOCKS_USIZE, ROLE_COUNT,
};
use crate::analysis::validate::trusted_dose::TrustedDose;
use crate::dataset::dose_index::DoseIndex;
use crate::dataset::physical_cardinalities::PhysicalCardinalities;
use crate::observation::event_evidence::EventEvidence;
use crate::observation::latency_sample::LatencySample;
use crate::observation::latency_summary::LatencySummary;
use crate::observation::raw_latencies::RawLatencies;
use crate::params::BATCH_SIZE;
use crate::plan::cell::Cell;
use crate::plan::run_role::RunRole;

/// Owns the complete canonical staging grid and is the sole helper handle the direct proofs use: the
/// ladder proofs perturb one slot and surrender the slots to `validate_ladders`; the run-role proof takes
/// a complete trusted-dose ladder for `TrustedRun::mint`.
pub(super) struct StagedLadder {
    slots: Vec<Vec<StagedDose>>,
}

impl StagedLadder {
    /// The complete canonical grid: every run's ten doses strictly monotonic and every block's arm and
    /// control ladders equal, so `validate_ladders` passes until a proof perturbs one slot.
    pub(super) fn canonical() -> Self {
        let mut slots: Vec<Vec<StagedDose>> = (0..DOSE_SLOTS).map(|_| Vec::new()).collect();
        for cell_idx in 0..CELL_COUNT {
            for role_idx in 0..ROLE_COUNT {
                for block in 0..REPETITION_BLOCKS_USIZE {
                    let run_slot = run_slot_index(cell_idx, role_idx, block);
                    for dose_zero_based in 0..NUM_DOSES_USIZE {
                        let logical_n = DoseIndex::ALL[dose_zero_based].cumulative_driving_rows();
                        slots[dose_slot_index(run_slot, dose_zero_based)].push(staged_dose(logical_n));
                    }
                }
            }
        }
        Self { slots }
    }

    /// Replace the sole staged dose at a canonical `(cell, role, block, dose)` slot with one carrying
    /// `logical_n`, so a proof can break exactly one ladder obligation.
    pub(super) fn set_logical_n(
        &mut self,
        cell_idx: usize,
        role: RunRole,
        block: usize,
        dose_zero_based: usize,
        logical_n: u64,
    ) {
        let run_slot = run_slot_index(cell_idx, role_index(role), block);
        self.slots[dose_slot_index(run_slot, dose_zero_based)] = vec![staged_dose(logical_n)];
    }

    /// The staged slots, as `validate_ladders` consumes them.
    pub(super) fn slots(&self) -> &[Vec<StagedDose>] {
        &self.slots
    }

    /// A complete monotonic dose ladder for one run — the `Box<[TrustedDose; NUM_DOSES]>` a
    /// `TrustedRun::mint` call requires — each dose carrying its canonical cumulative count and filler
    /// content. Built heap-first (Vec → boxed slice → boxed fixed array) so no by-value dose array is
    /// materialized, mirroring the production mint path.
    pub(super) fn trusted_doses(cell: Cell) -> Box<[TrustedDose; NUM_DOSES_USIZE]> {
        let doses: Vec<TrustedDose> = DoseIndex::ALL
            .into_iter()
            .map(|dose| {
                let latencies = filler_latencies();
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
        doses
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("DoseIndex::ALL supplies exactly NUM_DOSES trusted doses")
    }
}

/// A single staged dose carrying `logical_n` and otherwise-irrelevant filler content. The lossless
/// latency vector is built once and its summary derived from it before it is moved into the dose, so the
/// BATCH_SIZE sample vector is materialized only once.
fn staged_dose(logical_n: u64) -> StagedDose {
    let latencies = filler_latencies();
    let summary = LatencySummary::from_raw(&latencies);
    StagedDose {
        logical_n,
        latencies,
        summary,
        cardinalities: PhysicalCardinalities::expected(Cell::all()[0], DoseIndex::ALL[0]),
        events: filler_events(),
    }
}

/// A full, seal-valid latency vector of exactly BATCH_SIZE strictly-positive samples.
fn filler_latencies() -> RawLatencies {
    let samples = (1..=BATCH_SIZE)
        .map(|nanos| LatencySample::from_nanos(u128::from(nanos)))
        .collect();
    RawLatencies::sealed(samples).expect("a filler dose carries exactly BATCH_SIZE samples")
}

/// An identity-consistent event stream: inserts − deletes equals the net delta.
fn filler_events() -> EventEvidence {
    EventEvidence::checked(3, 0, 1, 3)
        .expect("filler event counts satisfy the insert/delete/net-delta identity")
}
