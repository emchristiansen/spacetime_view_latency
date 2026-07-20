//! The classified campaign: the nine per-cell classifications of a validated campaign.

use crate::analysis::beta::beta_descriptor::BetaDescriptor;
use crate::analysis::classify::classified_cell::ClassifiedCell;
use crate::analysis::validate::validated_campaign::ValidatedCampaign;

/// The number of preregistered arm/regime cells (the length of
/// [`Cell::all`](crate::plan::cell::Cell::all)): five table-scoped arms under unrelated growth plus the
/// two key-scoped arms under each of the two growth regimes. It fixes the classification array length.
///
/// That these nine slots equal the trusted campaign's nine cells is proven by the validation pass that
/// produced the [`ValidatedCampaign`], not by this constant; the classifier maps each already-proven
/// cell to exactly one classification.
const CELL_COUNT: usize = 9;

/// The complete classification of a validated campaign: exactly one stored [`ClassifiedCell`] per
/// preregistered cell. Heap-owned as a boxed fixed array so the nine-cell cardinality is a property of
/// the type while the campaign's by-value footprint stays pointer-sized (mirroring the trusted graph's
/// heap-first fixed arrays).
#[derive(Debug)]
pub(crate) struct ClassifiedCampaign {
    cells: Box<[ClassifiedCell; CELL_COUNT]>,
}

impl ClassifiedCampaign {
    /// Classify a validated campaign cell by cell: project each of the nine trusted
    /// [`CellDataset`](crate::analysis::validate::cell_dataset::CellDataset)s through
    /// [`BetaDescriptor::project`], which computes the exact primary classification from that same
    /// dataset and fits the secondary β descriptor only when the primary result is Increasing. The
    /// projection takes only the dataset, so a cell's primary result is never paired with a foreign
    /// dataset.
    pub(crate) fn classify(campaign: &ValidatedCampaign) -> Self {
        let cells: Vec<ClassifiedCell> = campaign
            .cells()
            .iter()
            .map(BetaDescriptor::project)
            .collect();
        // Heap-first: seal the per-cell result Vec into a boxed fixed array by the same fallible
        // conversion the trusted graph uses, so the nine-cell cardinality stays a property of the type
        // and no by-value classification array is materialized.
        let cells: Box<[ClassifiedCell; CELL_COUNT]> = cells
            .into_boxed_slice()
            .try_into()
            .ok()
            .expect("exactly CELL_COUNT cells classify into exactly CELL_COUNT results");
        Self { cells }
    }

    /// The nine per-cell stored classifications.
    pub(crate) fn cells(&self) -> &[ClassifiedCell; CELL_COUNT] {
        &self.cells
    }
}

#[cfg(test)]
mod tests;
