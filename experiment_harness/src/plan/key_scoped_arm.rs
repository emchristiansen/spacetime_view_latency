//! Arms predicted to record key-scoped (point) dependencies.

use crate::plan::access_path::AccessPath;
use crate::plan::recorded_read_set_class::RecordedReadSetClass;

/// Arms predicted to record key-scoped (point) dependencies.
///
/// These run under **both** growth regimes: flat under other identities' growth,
/// O(own slice) when the measured identity's own slice grows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum KeyScopedArm {
    /// F — procedural bounded point-filter over the single-column `sender` btree.
    PointFilter,
    /// F′ — procedural hand-written semijoin: viewer point-filter, then one PK
    /// `.find` per visibility row.
    PointSemijoin,
}

impl KeyScopedArm {
    /// This arm's genuine execution access path (spec: "F/F′ are `FullKeyPoint + KeyScoped`"). Both
    /// F and F′ reach every row through a full-key equality point seek.
    pub fn access_path(self) -> AccessPath {
        AccessPath::FullKeyPoint
    }

    /// The read-set class v2.6.1 records for this arm. Both key-scoped arms record `KeyScoped`.
    pub fn recorded_read_set_class(self) -> RecordedReadSetClass {
        RecordedReadSetClass::KeyScoped
    }
}
