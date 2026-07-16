//! The schedulable unit: a valid `(arm, growth-regime)` pairing.
//!
//! [`Cell`] is the only state the schedule stores. Its three variants are the only
//! constructors, and there is no public `Arm × GrowthRegime` product, so impossible
//! pairings — e.g. a table-scoped arm under own-slice growth — cannot be built. The
//! matched arm/control runs, the growth regime, the predicted response, and the
//! control table are all derived from the cell, never supplied independently.

use crate::plan::control_table::ControlTable;
use crate::plan::growth_regime::GrowthRegime;
use crate::plan::key_scoped_arm::KeyScopedArm;
use crate::plan::predicted_response::PredictedResponse;
use crate::plan::run::Run;
use crate::plan::table_scoped_arm::TableScopedArm;

/// A valid arm/regime pairing eligible for scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Cell {
    /// A table-scoped arm (A–E) under unrelated-total growth — its only regime.
    TableScopedUnrelated(TableScopedArm),
    /// A key-scoped arm (F/F′) under unrelated-total growth.
    KeyScopedUnrelated(KeyScopedArm),
    /// A key-scoped arm (F/F′) under own-slice growth — reachable only by F/F′.
    KeyScopedOwnSlice(KeyScopedArm),
}

impl Cell {
    /// The complete preregistered set of valid cells: every table-scoped arm under
    /// unrelated growth, and every key-scoped arm under both growth regimes.
    pub fn all() -> Vec<Cell> {
        let table_scoped = [
            TableScopedArm::ProceduralRange,
            TableScopedArm::QueryFull,
            TableScopedArm::QuerySemijoin,
            TableScopedArm::QueryFullPk,
            TableScopedArm::QuerySemijoinPk,
        ];
        let key_scoped = [KeyScopedArm::PointFilter, KeyScopedArm::PointSemijoin];
        table_scoped
            .into_iter()
            .map(Cell::TableScopedUnrelated)
            .chain(key_scoped.into_iter().map(Cell::KeyScopedUnrelated))
            .chain(key_scoped.into_iter().map(Cell::KeyScopedOwnSlice))
            .collect()
    }

    /// The growth regime this cell runs in.
    pub fn growth_regime(self) -> GrowthRegime {
        match self {
            Cell::TableScopedUnrelated(_) | Cell::KeyScopedUnrelated(_) => {
                GrowthRegime::UnrelatedGrowth
            }
            Cell::KeyScopedOwnSlice(_) => GrowthRegime::OwnSliceGrowth,
        }
    }

    /// The preregistered prediction for this cell.
    pub fn predicted_response(self) -> PredictedResponse {
        match self {
            Cell::TableScopedUnrelated(_) => {
                PredictedResponse::IncreasingIn(GrowthRegime::UnrelatedGrowth)
            }
            Cell::KeyScopedUnrelated(_) => PredictedResponse::FlatIn(GrowthRegime::UnrelatedGrowth),
            Cell::KeyScopedOwnSlice(_) => {
                PredictedResponse::IncreasingIn(GrowthRegime::OwnSliceGrowth)
            }
        }
    }

    /// The matched direct-base-table control for this cell's arm (spec matrix:
    /// A/B/D and F use `message`; C/E/F′ use `chronicle_message`).
    pub fn control_table(self) -> ControlTable {
        match self {
            Cell::TableScopedUnrelated(arm) => match arm {
                TableScopedArm::ProceduralRange
                | TableScopedArm::QueryFull
                | TableScopedArm::QueryFullPk => ControlTable::Message,
                TableScopedArm::QuerySemijoin | TableScopedArm::QuerySemijoinPk => {
                    ControlTable::ChronicleMessage
                }
            },
            Cell::KeyScopedUnrelated(arm) | Cell::KeyScopedOwnSlice(arm) => match arm {
                KeyScopedArm::PointFilter => ControlTable::Message,
                KeyScopedArm::PointSemijoin => ControlTable::ChronicleMessage,
            },
        }
    }

    /// Derive this cell's two block runs — the arm run and its matched control-only
    /// run, in that order. This is the ONLY way to obtain a [`Run`]: an arm can
    /// never be paired with the wrong control table.
    pub fn matched_runs(self) -> [Run; 2] {
        Run::matched_pair(self)
    }

    /// A stable canonical tag uniquely identifying this cell, for deterministic identity
    /// derivation and machine-readable evidence.
    ///
    /// Deliberately an explicit `&'static str` contract rather than `Debug`/variant-name
    /// formatting: the growth identity is derived from this tag via `Identity::from_claims`,
    /// so a `Debug`-derived subject would silently change every growth identity if a Rust
    /// variant were ever renamed, breaking reproducibility against a recorded seed. The tag
    /// is fixed independently of the variant names, so refactors cannot move it.
    pub fn canonical_tag(self) -> &'static str {
        match self {
            Cell::TableScopedUnrelated(TableScopedArm::ProceduralRange) => "arm-a-unrelated",
            Cell::TableScopedUnrelated(TableScopedArm::QueryFull) => "arm-b-unrelated",
            Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoin) => "arm-c-unrelated",
            Cell::TableScopedUnrelated(TableScopedArm::QueryFullPk) => "arm-d-unrelated",
            Cell::TableScopedUnrelated(TableScopedArm::QuerySemijoinPk) => "arm-e-unrelated",
            Cell::KeyScopedUnrelated(KeyScopedArm::PointFilter) => "arm-f-unrelated",
            Cell::KeyScopedUnrelated(KeyScopedArm::PointSemijoin) => "arm-fprime-unrelated",
            Cell::KeyScopedOwnSlice(KeyScopedArm::PointFilter) => "arm-f-ownslice",
            Cell::KeyScopedOwnSlice(KeyScopedArm::PointSemijoin) => "arm-fprime-ownslice",
        }
    }
}
