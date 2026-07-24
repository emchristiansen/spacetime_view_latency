//! The recorded database access path an arm's module view executes through.

/// The access path a view body executes through (spec: `AccessPath = QueryPlan | NonPointRange |
/// FullKeyPoint`). Kept separate from [`RecordedReadSetClass`](super::recorded_read_set_class::RecordedReadSetClass):
/// the access path is what the view body does; the recorded read-set class is what v2.6.1 records as a
/// consequence. Arm A proves these are genuinely distinct — a `NonPointRange` access path that still
/// records `TableScoped`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum AccessPath {
    /// B–E: raw/query-builder plan execution over the table.
    QueryPlan,
    /// A: a full-domain unbounded range scan over a single-column btree index.
    NonPointRange,
    /// F/F′: a full-key equality point seek (index accessor or PK `.find`).
    FullKeyPoint,
}
