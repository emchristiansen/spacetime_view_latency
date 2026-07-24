//! The report projection of one refined basin's golden-section termination telemetry.

use serde::Serialize;

use crate::analysis::beta::basin_termination::BasinTermination;
use crate::analysis::finite_f64::FiniteF64;
use crate::analysis::report::golden_stop_report::GoldenStopReport;

/// The report projection of one basin's [`BasinTermination`] (spec: "termination telemetry for every
/// refined local basin (initial/final brackets, iteration count, and `BracketWidthReached` versus
/// `IterationCapReached`)"). Every bracket endpoint is a finite [`FiniteF64`] — the source brackets live
/// within the `[0.1, 4.0]` domain by construction, so no non-finite value can reach serialization.
#[derive(Debug, Serialize)]
pub(crate) struct BasinTerminationReport {
    /// The 0-based grid node this basin refined from.
    grid_node_index: usize,
    /// The lower endpoint of the refinement's initial bracket.
    initial_bracket_lo: FiniteF64,
    /// The upper endpoint of the refinement's initial bracket.
    initial_bracket_hi: FiniteF64,
    /// The lower endpoint of the bracket at termination.
    final_bracket_lo: FiniteF64,
    /// The upper endpoint of the bracket at termination.
    final_bracket_hi: FiniteF64,
    /// The number of golden-section iterations executed before termination.
    iterations: usize,
    /// Why the refinement stopped.
    stop: GoldenStopReport,
}

impl BasinTerminationReport {
    /// Project one basin's termination telemetry into its finite report shape.
    pub(crate) fn of(termination: &BasinTermination) -> Self {
        // Every bracket endpoint is finite by construction in the source (the brackets live within
        // `[0.1, 4.0]`), so each `FiniteF64::new` assertion can never fire here.
        Self {
            grid_node_index: termination.grid_node_index(),
            initial_bracket_lo: FiniteF64::new(termination.initial_bracket_lo()),
            initial_bracket_hi: FiniteF64::new(termination.initial_bracket_hi()),
            final_bracket_lo: FiniteF64::new(termination.final_bracket_lo()),
            final_bracket_hi: FiniteF64::new(termination.final_bracket_hi()),
            iterations: termination.iterations(),
            stop: GoldenStopReport::of(termination.stop()),
        }
    }
}
