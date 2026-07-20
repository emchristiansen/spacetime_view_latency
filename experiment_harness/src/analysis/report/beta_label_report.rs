//! The report projection of a population β interval's linear/sublinear consistency label.

use serde::Serialize;

use crate::analysis::beta::beta_label::BetaLabel;

/// The report projection of the analysis-domain [`BetaLabel`], which is intentionally non-`Serialize` so
/// it cannot leak into JSON unprojected. A closed report enum (never a `String`), so an invalid or
/// misspelled label is unrepresentable in the serialized output; its three variants mirror
/// [`BetaLabel`]'s exactly.
#[derive(Debug, Serialize)]
pub(crate) enum BetaLabelReport {
    /// The whole interval lies within `[0.8, 1.2]`: consistent with a linear `L(N) ∝ N`.
    LinearConsistent,
    /// The whole interval lies within `(0.2, 0.8)`: consistent with a sublinear `L(N) ∝ N^β`, `β < 1`.
    SublinearConsistent,
    /// The interval fits neither band wholly: no consistency label is claimed.
    Unlabeled,
}

impl BetaLabelReport {
    /// Project the analysis-domain label. Takes the `Copy` [`BetaLabel`] by value — one input.
    pub(crate) fn of(label: BetaLabel) -> Self {
        let _ = label;
        todo!("Phase 2: map each BetaLabel variant to its report variant")
    }
}
