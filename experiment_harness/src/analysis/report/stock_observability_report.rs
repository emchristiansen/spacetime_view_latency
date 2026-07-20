//! The report projection of the explicitly-stated stock-server observability limits.

use serde::Serialize;

use crate::analysis::report::stock_observability_limit::{
    STOCK_OBSERVABILITY_LIMIT_COUNT, StockObservabilityLimit,
};

/// The explicitly-reported stock observability limits (spec: "explicitly report stock observability
/// limits"). A boxed fixed [`STOCK_OBSERVABILITY_LIMIT_COUNT`] array of the closed typed
/// [`StockObservabilityLimit`], so the stated set is fixed and auditable and serde supports its length
/// directly. These are protocol constants, not campaign-derived data, so this projection takes no input.
#[derive(Debug, Serialize)]
pub(crate) struct StockObservabilityReport {
    /// The fixed set of what the stock server cannot directly prove, stated explicitly.
    limits: Box<[StockObservabilityLimit; STOCK_OBSERVABILITY_LIMIT_COUNT]>,
}

impl StockObservabilityReport {
    /// Build the fixed observability-limit set from the closed
    /// [`StockObservabilityLimit::ALL`](super::stock_observability_limit::StockObservabilityLimit::ALL)
    /// identities. Takes no input: the limits are protocol constants.
    pub(crate) fn all() -> Self {
        // The closed `ALL` set is the exact stated limit set; box it into the fixed array so the stated
        // cardinality stays a property of the type. The set is tiny (two protocol constants), so a direct
        // `Box::new` of the `Copy` array is faithful with no heap-first staging.
        Self {
            limits: Box::new(StockObservabilityLimit::ALL),
        }
    }
}

#[cfg(test)]
mod tests;
