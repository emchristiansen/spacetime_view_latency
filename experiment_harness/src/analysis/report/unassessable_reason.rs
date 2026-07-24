//! The closed reason one cell contributes to the escalation gate without a comparison.

use serde::Serialize;

/// Why a cell is unassessable by the stock-to-patch escalation gate (spec: "`UnassessableReason` is
/// closed over `InvalidControl | InconclusiveComparison`"). A closed enum, never a `String`, so an
/// unassessable cell always carries one of exactly these two typed reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum UnassessableReason {
    /// The cell's direct control failed the flat-equivalence gate, so it carries no arm claim.
    InvalidControl,
    /// The cell's control is valid but its observed response is Inconclusive, so it confirms neither
    /// prediction.
    InconclusiveComparison,
}
