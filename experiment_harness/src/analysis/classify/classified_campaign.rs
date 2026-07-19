//! The classified campaign: the nine per-cell classifications of a validated campaign.

use crate::analysis::classify::cell_classification::CellClassification;
use crate::analysis::classify::cell_evidence::CellEvidence;
use crate::analysis::classify::equivalence_margin::{CONTROL_DOSE_MEDIAN_COUNT, EquivalenceMargin};
use crate::analysis::stats::rational::Rational;
use crate::analysis::stats::theil_sen::theil_sen_slope;
use crate::analysis::stats::total_change::total_change;
use crate::analysis::validate::arm_run::ArmRun;
use crate::analysis::validate::cell_dataset::CellDataset;
use crate::analysis::validate::control_run::ControlRun;
use crate::analysis::validate::trusted_run::TrustedRun;
use crate::analysis::validate::validated_campaign::ValidatedCampaign;
use crate::observation::record_seq::RecordSeq;
use crate::params::{NUM_DOSES_USIZE, REPETITION_BLOCKS};

/// The number of preregistered arm/regime cells (the length of
/// [`Cell::all`](crate::plan::cell::Cell::all)): five table-scoped arms under unrelated growth plus the
/// two key-scoped arms under each of the two growth regimes. It fixes the classification array length.
///
/// That these nine slots equal the trusted campaign's nine cells is proven by the validation pass that
/// produced the [`ValidatedCampaign`], not by this constant; the classifier maps each already-proven
/// cell to exactly one classification.
const CELL_COUNT: usize = 9;

/// The fixed 30-block per-cell sample size as an array length, guarded by a compile-time round-trip
/// assertion rather than a bare `as` cast — mirroring
/// [`CellDataset`](crate::analysis::validate::cell_dataset::CellDataset)'s `BLOCKS_PER_CELL` — so a
/// platform on which [`REPETITION_BLOCKS`] does not fit a `usize` fails to compile instead of silently
/// truncating the array length.
const BLOCKS_PER_CELL: usize = {
    let as_usize = REPETITION_BLOCKS as usize;
    assert!(
        as_usize as u32 == REPETITION_BLOCKS,
        "REPETITION_BLOCKS does not fit in usize on this platform"
    );
    as_usize
};

/// The complete classification of a validated campaign: exactly one [`CellClassification`] per
/// preregistered cell. Heap-owned as a boxed fixed array so the nine-cell cardinality is a property of
/// the type while the campaign's by-value footprint stays pointer-sized (mirroring the trusted graph's
/// heap-first fixed arrays).
#[derive(Debug)]
pub(crate) struct ClassifiedCampaign {
    cells: Box<[CellClassification; CELL_COUNT]>,
}

impl ClassifiedCampaign {
    /// Classify a validated campaign cell by cell: for each of the nine trusted [`CellDataset`]s,
    /// aggregate its fixed per-block evidence, freeze the margin, and apply the control-validity gate.
    ///
    /// [`CellDataset`]: crate::analysis::validate::cell_dataset::CellDataset
    pub(crate) fn classify(campaign: &ValidatedCampaign) -> Self {
        let cells: Vec<CellClassification> = campaign.cells().iter().map(classify_cell).collect();
        // Heap-first: seal the per-cell classification Vec into a boxed fixed array by the same fallible
        // conversion the trusted graph uses, so the nine-cell cardinality stays a property of the type
        // and no by-value classification array is materialized.
        let cells: Box<[CellClassification; CELL_COUNT]> = cells
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly CELL_COUNT cells classify into exactly CELL_COUNT classifications");
        Self { cells }
    }

    /// The nine per-cell classifications.
    pub(crate) fn cells(&self) -> &[CellClassification; CELL_COUNT] {
        &self.cells
    }
}

/// Aggregate one validated cell dataset into its gated classification: fold each of the 30 matched
/// blocks into its arm-minus-control and direct-control per-block total changes, freeze the margin over
/// the cell's 300 matched-control dose medians, order the block estimates by durable collection order,
/// and apply the control-validity gate.
fn classify_cell(dataset: &CellDataset) -> CellClassification {
    let blocks = dataset.blocks();

    // Freeze δ over the cell's 300 matched-control dose medians (30 blocks × 10 doses), each an exact
    // rational nanosecond count; order is irrelevant to the median.
    let control_dose_medians: Vec<Rational> = blocks
        .iter()
        .flat_map(|block| {
            block
                .control()
                .doses()
                .iter()
                .map(|dose| Rational::from_int(median_nanos_i128(dose.median_nanos())))
        })
        .collect();
    let control_dose_medians: [Rational; CONTROL_DOSE_MEDIAN_COUNT] = control_dose_medians
        .try_into()
        .ok()
        .expect("30 blocks × 10 control doses yield exactly CONTROL_DOSE_MEDIAN_COUNT medians");
    let margin = EquivalenceMargin::freeze(&control_dose_medians);

    // Per-block estimates, then ordered by the durable collection-order key so the evidence arrays are
    // in schedule-proven collection order rather than the canonical cell→role→block traversal order.
    let mut estimates: Vec<(RecordSeq, Rational, Rational)> = blocks
        .iter()
        .map(|block| {
            let key = block.collection_order_key();
            let arm_total = arm_minus_control_total_change(block.arm(), block.control());
            let control_total = control_total_change(block.control());
            (key, arm_total, control_total)
        })
        .collect();
    estimates.sort_by_key(|(key, _, _)| *key);

    let collection_order_keys: [RecordSeq; BLOCKS_PER_CELL] = estimates
        .iter()
        .map(|(key, _, _)| *key)
        .collect::<Vec<_>>()
        .try_into()
        .ok()
        .expect("exactly BLOCKS_PER_CELL block collection-order keys");
    let arm_minus_control_totals: [Rational; BLOCKS_PER_CELL] = estimates
        .iter()
        .map(|(_, arm_total, _)| *arm_total)
        .collect::<Vec<_>>()
        .try_into()
        .ok()
        .expect("exactly BLOCKS_PER_CELL arm-minus-control total changes");
    let control_totals: [Rational; BLOCKS_PER_CELL] = estimates
        .iter()
        .map(|(_, _, control_total)| *control_total)
        .collect::<Vec<_>>()
        .try_into()
        .ok()
        .expect("exactly BLOCKS_PER_CELL control total changes");

    let evidence = CellEvidence::new(
        dataset.cell(),
        arm_minus_control_totals,
        control_totals,
        collection_order_keys,
        margin,
    );
    CellClassification::classify(evidence)
}

/// The exact per-block total fitted change of the arm-minus-control paired difference `D(N) = L_arm −
/// L_control`: the Theil–Sen slope of the ten `(logical_n, D)` points over the block's shared dose
/// ladder, scaled to the full ladder span (spec: `T_block = slope_block(D) · (N_max − N_min)`). The arm
/// and control ladders were proven equal dose-by-dose by validation, so the shared x-axis is the arm's
/// logical-`n` and the paired difference is taken dose-index-aligned.
fn arm_minus_control_total_change(
    arm: &TrustedRun<ArmRun>,
    control: &TrustedRun<ControlRun>,
) -> Rational {
    let arm_doses = arm.doses();
    let control_doses = control.doses();
    let points: Vec<(i128, i128)> = (0..NUM_DOSES_USIZE)
        .map(|dose_index| {
            let logical_n = i128::from(arm_doses[dose_index].logical_n());
            let l_arm = median_nanos_i128(arm_doses[dose_index].median_nanos());
            let l_control = median_nanos_i128(control_doses[dose_index].median_nanos());
            let paired_difference = l_arm
                .checked_sub(l_control)
                .expect("a paired difference of two nanosecond medians fits i128");
            (logical_n, paired_difference)
        })
        .collect();
    total_change_over_ladder(&points)
}

/// The exact per-block total fitted change of the direct control's own response `L_control(N)`: the
/// Theil–Sen slope of the ten `(logical_n, L_control)` points scaled to the full ladder span — the
/// independent input to the control-validity gate.
fn control_total_change(control: &TrustedRun<ControlRun>) -> Rational {
    let doses = control.doses();
    let points: Vec<(i128, i128)> = (0..NUM_DOSES_USIZE)
        .map(|dose_index| {
            let logical_n = i128::from(doses[dose_index].logical_n());
            let l_control = median_nanos_i128(doses[dose_index].median_nanos());
            (logical_n, l_control)
        })
        .collect();
    total_change_over_ladder(&points)
}

/// The exact total fitted change of a per-block regression over the shared dose ladder: the Theil–Sen
/// slope of the points scaled by the ladder span `N_max − N_min`, taking the span from the ladder's
/// endpoints. The doses are in strict monotonic ladder order, so the first and last points carry the
/// minimum and maximum x; `total_change` asserts the span is strictly positive.
fn total_change_over_ladder(points: &[(i128, i128)]) -> Rational {
    let n_min = points
        .first()
        .expect("a trusted run carries the complete ten-dose ladder")
        .0;
    let n_max = points
        .last()
        .expect("a trusted run carries the complete ten-dose ladder")
        .0;
    total_change(theil_sen_slope(points), n_min, n_max)
}

/// A trusted nanosecond median as an `i128` for exact rational arithmetic. The campaign's nanosecond
/// magnitudes (order `1e7`–`1e10`) leave vast `i128` headroom; a `u128` above `i128::MAX` is an
/// impossible latency and fails loud rather than wrapping.
fn median_nanos_i128(median_nanos: u128) -> i128 {
    i128::try_from(median_nanos).expect("a nanosecond latency median fits i128")
}
