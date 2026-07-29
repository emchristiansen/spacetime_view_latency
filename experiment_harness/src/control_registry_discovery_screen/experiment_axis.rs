//! The axis this screen sweeps.

use serde::Serialize;

/// The single axis the discovery screen moves: append-only `control_activity` history depth, with
/// the registry's own cardinality `K` held fixed.
///
/// Named for the existing frozen unrelated/global range whose endpoints it uses, because that is
/// what the rows are with respect to the *deployable* arm: Arm A's read set is the K-row registry at
/// both rungs, so the history is backing growth it must stay flat against. For the diagnostic
/// comparator the same rows are its read set — which is the contrast the screen exists to observe,
/// not a second axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum ExperimentAxis {
    UnrelatedGlobalRows,
}
