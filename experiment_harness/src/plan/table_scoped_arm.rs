//! Arms predicted to record table-scoped (O(N)) dependencies.

use crate::plan::access_path::AccessPath;
use crate::plan::recorded_read_set_class::RecordedReadSetClass;

/// Arms predicted to record table-scoped (O(N)) dependencies under unrelated growth.
///
/// These run **only** under `UnrelatedGrowth`; own-slice growth is meaningless for a
/// full-table-scoped read set. Keeping them in a separate enum from the key-scoped
/// arms is what makes "table-scoped arm under own-slice growth" unrepresentable (see
/// [`crate::plan::cell::Cell`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum TableScopedArm {
    /// A — procedural `Vec<Row>` full-domain unbounded range over the `sender` btree.
    ProceduralRange,
    /// B — query-builder `impl Query<Row>` full pass-through (Anton's current form).
    QueryFull,
    /// C — query-builder Papaya-shaped viewer-equality plus message-UUID semijoin.
    QuerySemijoin,
    /// D — B plus an explicit custom view primary key.
    QueryFullPk,
    /// E — C plus an explicit custom view primary key; identical query body to C.
    QuerySemijoinPk,
}

impl TableScopedArm {
    /// This arm's genuine execution access path (spec: "Arm A is `NonPointRange + TableScoped`;
    /// ... B/C/D/E are `QueryPlan + TableScoped`"). Only A executes a non-point range; B–E execute
    /// through the query-builder plan.
    pub fn access_path(self) -> AccessPath {
        match self {
            TableScopedArm::ProceduralRange => AccessPath::NonPointRange,
            TableScopedArm::QueryFull
            | TableScopedArm::QuerySemijoin
            | TableScopedArm::QueryFullPk
            | TableScopedArm::QuerySemijoinPk => AccessPath::QueryPlan,
        }
    }

    /// The read-set class v2.6.1 records for this arm. Every table-scoped arm — including A's genuine
    /// non-point range — is conservatively recorded `TableScoped`.
    pub fn recorded_read_set_class(self) -> RecordedReadSetClass {
        RecordedReadSetClass::TableScoped
    }
}
