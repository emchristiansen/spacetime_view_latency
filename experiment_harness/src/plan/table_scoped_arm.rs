//! Arms predicted to record table-scoped (O(N)) dependencies.

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
