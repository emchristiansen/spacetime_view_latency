//! The report projection of a cell's typed identity: its arm/regime pairing and derived predictions.

use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::plan::cell::Cell;

/// The typed identity of one arm/regime [`Cell`] in the report. It stores **only** the canonical [`Cell`]
/// — the single source of truth — and derives the growth regime, predicted response, and canonical tag
/// from it *at serialization time*, since all three are pure functions of the cell
/// ([`Cell::growth_regime`], [`Cell::predicted_response`], [`Cell::canonical_tag`]). Storing only the cell
/// makes a derived fact disagreeing with the cell **unrepresentable** — not merely discouraged by private
/// fields and one constructor — because there is no second field to disagree.
///
/// The manual [`Serialize`] reproduces the identical JSON object (`cell`, `canonical_tag`, `growth_regime`,
/// `predicted_response`) that independent stored fields would emit, but with no duplicated derived state in
/// memory. [`Cell`], [`GrowthRegime`](crate::plan::growth_regime::GrowthRegime), and
/// [`PredictedResponse`](crate::plan::predicted_response::PredictedResponse) are closed `Serialize` domain
/// enums, so an invalid cell/regime/prediction is unrepresentable; the canonical tag is a stable
/// `&'static str` contract fixed independently of the Rust variant names.
#[derive(Debug)]
pub(crate) struct CellIdentityReport {
    /// The valid `(arm, growth-regime)` pairing this report cell covers — the sole stored identity, from
    /// which every serialized derived fact is computed.
    cell: Cell,
}

impl CellIdentityReport {
    /// Bind a cell's typed identity. One input — the `Copy` [`Cell`] — stored as the sole source of truth;
    /// the tag, regime, and prediction are derived from it at serialization, so none can disagree with the
    /// cell. A trivial canonical wrapper (it introduces no evidence of its own), so it is not a `todo!()`.
    pub(crate) fn of(cell: Cell) -> Self {
        Self { cell }
    }

    /// The canonical [`Cell`] this identity covers — the report's typed cell key, so a consumer (e.g. the
    /// stock-to-patch escalation gate) identifies a cell canonically without parsing its tag string.
    pub(crate) fn cell(&self) -> Cell {
        self.cell
    }
}

impl Serialize for CellIdentityReport {
    /// Serialize the canonical cell plus its three derived facts, each computed from the one stored [`Cell`]
    /// at serialization time — the same JSON object independent fields would emit, but with no duplicated
    /// derived state stored, so the serialized regime/prediction/tag cannot drift from the cell.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("CellIdentityReport", 4)?;
        state.serialize_field("cell", &self.cell)?;
        state.serialize_field("canonical_tag", self.cell.canonical_tag())?;
        state.serialize_field("growth_regime", &self.cell.growth_regime())?;
        state.serialize_field("predicted_response", &self.cell.predicted_response())?;
        state.end()
    }
}
