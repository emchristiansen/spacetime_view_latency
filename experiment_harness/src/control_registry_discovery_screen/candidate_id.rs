//! The candidates this screen can produce evidence for.

use serde::Serialize;

/// The two candidates the discovery screen measures, and the reason the screen needs a candidate
/// dimension at all.
///
/// Arm A and Arm B are *composition-matched*, not variants of one candidate: Arm A is the
/// reducer-maintained O(K) registry proposed for deployment, Arm B is the O(N) procedural comparator
/// that may never itself support a production recommendation. They owe separate terminal candidate
/// dispositions, and attempt identity binds candidate — so minting Arm B's identities under
/// `ControlRegistry` would merge two dispositions that the spec keeps apart.
///
/// Both variants are producible here. The spec's wider `CandidateId` vocabulary is not restated:
/// this enum carries only what this module writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum CandidateId {
    /// The reducer-maintained bounded current-state registry — the deployable discovery candidate.
    ControlRegistry,
    /// The latest-per-control procedural view over append-only history. `DiagnosticOnly`: an
    /// exact-version-coupled O(N) comparator, never independently recommendable.
    ControlActivityLatestByControlView,
}
