//! The report projection of a cell's population β interval and consistency label.

use serde::Serialize;

use crate::analysis::beta::beta_population::BetaPopulation;
use crate::analysis::finite_f64::FiniteF64;
use crate::analysis::report::beta_label_report::BetaLabelReport;

/// The report projection of a [`BetaPopulation`]: the `[X_(10), X_(21)]` order-statistic β interval, its
/// coverage, and its linear/sublinear consistency label. Present only when every one of a cell's 30 blocks
/// is identifiable, so this DTO is carried in an `Option` by its parent
/// [`SecondaryDescriptorReport`](super::secondary_descriptor_report::SecondaryDescriptorReport). Interval
/// endpoints and coverage are finite [`FiniteF64`]; the label is the closed typed
/// [`BetaLabelReport`] — never a `String` — so an invalid label state is unrepresentable.
#[derive(Debug, Serialize)]
pub(crate) struct BetaPopulationReport {
    /// The lower endpoint of the population β interval.
    interval_lo: FiniteF64,
    /// The upper endpoint of the population β interval.
    interval_hi: FiniteF64,
    /// The interval's order-statistic coverage.
    coverage: FiniteF64,
    /// The linear/sublinear consistency label, projected to a closed report enum.
    label: BetaLabelReport,
}

impl BetaPopulationReport {
    /// Project a cell's population β interval and label. One input — the `Copy` [`BetaPopulation`], whose
    /// endpoints/coverage cross the lossy boundary to [`FiniteF64`] and whose label projects to
    /// [`BetaLabelReport`].
    pub(crate) fn of(population: &BetaPopulation) -> Self {
        // The population endpoints and coverage are already finite by construction in the source
        // descriptor, and the label projects to the closed report enum.
        Self {
            interval_lo: FiniteF64::new(population.interval_lo()),
            interval_hi: FiniteF64::new(population.interval_hi()),
            coverage: FiniteF64::new(population.coverage()),
            label: BetaLabelReport::of(population.label()),
        }
    }
}
