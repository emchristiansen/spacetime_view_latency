//! Which kind of record a durable sink line carries.

use serde::Serialize;

use crate::dataset::dose_index::DoseIndex;

/// The kind of a durable sink record: the once-per-run immutable manifest, or one cumulative dose's
/// observation identified by its ladder index. Recorded in a [`RecordId`](super::record_id::RecordId)
/// so an ambiguous in-flight write can be named precisely in a frontier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum RecordKind {
    /// The immutable run manifest, written once before this run's observations.
    Manifest,
    /// One cumulative dose observation at the given ladder index.
    Dose(DoseIndex),
}
