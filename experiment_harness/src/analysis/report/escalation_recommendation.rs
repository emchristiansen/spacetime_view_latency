//! The closed stock-to-patch escalation recommendation.

use serde::Serialize;

use crate::analysis::report::non_empty_cells::NonEmptyCells;
use crate::analysis::report::non_empty_unassessable_cells::NonEmptyUnassessableCells;
use crate::analysis::report::unassessable_cell::UnassessableCell;

/// The typed outcome of the stock-to-patch escalation gate (spec § "Stock-to-patched-server escalation
/// gate"): whether the stock evidence answers the core question or a new scope decision is warranted. A
/// closed enum, never a `String`, and it never patches or authorizes patching — it only records the
/// recommendation. Exactly the spec's total fold over every cell:
///
/// - Every valid cell `Confirmed` and none `InvalidControl`/`Inconclusive` → [`RemainStock`](Self::RemainStock).
/// - At least one cell `Contradicted` → [`EscalateToPatch`](Self::EscalateToPatch), retaining every
///   co-occurring invalid-control/inconclusive cell rather than discarding it.
/// - No cell contradicted but at least one invalid/inconclusive → [`Unassessable`](Self::Unassessable).
#[derive(Debug, Serialize)]
pub(crate) enum EscalationRecommendation {
    /// The stock outcome answers the core question (spec: `{A–E}` Increasing and `{F, F′}` Flat-equivalent
    /// under unrelated growth), so no patch is needed and the experiment remains on the stock server.
    RemainStock,
    /// At least one cell is `Contradicted`. Recording this never authorizes the patch; it means return to
    /// Eric for a new scope decision.
    EscalateToPatch {
        /// The non-empty set of contradicted cells.
        contradicted_cells: NonEmptyCells,
        /// Every co-occurring invalid-control or inconclusive cell, retained rather than discarded.
        unassessable_cells: Box<[UnassessableCell]>,
    },
    /// No cell is contradicted, but at least one is invalid-control or inconclusive, so the gate cannot
    /// certify `RemainStock`.
    Unassessable {
        /// The non-empty set of unassessable cells.
        cells: NonEmptyUnassessableCells,
    },
}
