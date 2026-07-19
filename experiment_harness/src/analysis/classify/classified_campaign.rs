//! The classified campaign: the nine per-cell classifications of a validated campaign.

use crate::analysis::classify::cell_classification::CellClassification;
use crate::analysis::validate::validated_campaign::ValidatedCampaign;

/// The number of preregistered arm/regime cells (the length of
/// [`Cell::all`](crate::plan::cell::Cell::all)): five table-scoped arms under unrelated growth plus the
/// two key-scoped arms under each of the two growth regimes. It fixes the classification array length.
///
/// That these nine slots equal the trusted campaign's nine cells is proven by the validation pass that
/// produced the [`ValidatedCampaign`], not by this constant; the classifier maps each already-proven
/// cell to exactly one classification.
const CELL_COUNT: usize = 9;

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
    pub(crate) fn classify(_campaign: &ValidatedCampaign) -> Self {
        todo!("Phase 2: aggregate each trusted cell into CellEvidence and gate it into a CellClassification")
    }

    /// The nine per-cell classifications.
    pub(crate) fn cells(&self) -> &[CellClassification; CELL_COUNT] {
        &self.cells
    }
}
