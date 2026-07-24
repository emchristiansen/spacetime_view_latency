//! The read-set invalidation granularity v2.6.1 records for a view.

/// The recorded invalidation granularity (spec: `RecordedReadSetClass = TableScoped | KeyScoped`),
/// separate from [`AccessPath`](super::access_path::AccessPath): this is what v2.6.1 *records* as a
/// consequence of the access path, not the access path itself. Any non-point access path — including
/// Arm A's genuine `NonPointRange` — is conservatively recorded `TableScoped`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum RecordedReadSetClass {
    /// Unrelated writes to the table invalidate the view.
    TableScoped,
    /// Unrelated writes to disjoint growth keys do not invalidate the view.
    KeyScoped,
}
