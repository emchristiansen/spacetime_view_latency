//! The report projection of a control-valid non-Increasing cell's arm response.

use serde::Serialize;

use crate::analysis::classify::non_increasing_response::NonIncreasingResponse;

/// The report projection of the analysis-domain [`NonIncreasingResponse`], which is intentionally
/// non-`Serialize`. A closed report enum (never a `String`) that mirrors the domain type's three variants
/// exactly and, like it, *has no Increasing variant* — so an Increasing cell's response can never be
/// rendered here; that cell is the [`Increasing`](super::classified_cell_report::ClassifiedCellReport::Increasing)
/// classification branch instead.
#[derive(Debug, Serialize)]
pub(crate) enum NonIncreasingResponseReport {
    /// The arm's paired-difference interval lies wholly within `[-δ, +δ]`.
    FlatEquivalent,
    /// The arm's paired-difference interval lies wholly below `-δ`.
    Decreasing,
    /// The arm's paired-difference interval straddles a band bound.
    Inconclusive,
}

impl NonIncreasingResponseReport {
    /// Project the analysis-domain response. Takes the `Copy` [`NonIncreasingResponse`] by value — one
    /// input.
    pub(crate) fn of(response: NonIncreasingResponse) -> Self {
        let _ = response;
        todo!("Phase 2: map each NonIncreasingResponse variant to its report variant")
    }
}
