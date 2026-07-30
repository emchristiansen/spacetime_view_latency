//! The candidate version every attempt identity is pinned to.

use serde::Serialize;

/// A monotone version stamp for the candidate implementation an attempt exercised.
///
/// Part of attempt identity so series produced against different candidate code can never be read
/// as replicates of one another. Minted here rather than shared with the Pilot's or the discovery
/// screen's stamp: the three count different implementations, and a shared counter would make one
/// candidate's revision look like another's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct CandidateVersion(u32);

/// The indexed sender-view arm as it stands in the accepted `ControlRegistry` Step 1 artifacts —
/// module WASM SHA-256 `bb82c59e…`, generated-tree digest `cc5fdfcb…`.
///
/// Version one: this pilot exercises the first accepted implementation, and reuses those artifacts
/// unchanged because it adds no module change.
pub(crate) const INDEXED_SENDER_VIEW_CALIBRATION_VERSION: CandidateVersion = CandidateVersion(1);
