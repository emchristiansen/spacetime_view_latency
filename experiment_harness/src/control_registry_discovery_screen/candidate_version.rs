//! The candidate version every attempt identity is pinned to.

use serde::Serialize;

/// A monotone version stamp for the candidate implementation an attempt measured.
///
/// Part of attempt identity so results produced against different candidate code can never join into
/// one comparison. Minted here rather than shared with the Pilot's stamp: the two count different
/// implementations, and a shared counter would make one candidate's revision look like the other's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub(crate) struct CandidateVersion(u32);

/// The discovery candidate pair as accepted at `ControlRegistry` Step 1 — module WASM SHA-256
/// `bb82c59e…`, generated-tree digest `cc5fdfcb…`. Version one: this screen measures the first
/// accepted implementation, and the freeze reuses those artifacts unchanged because it adds no
/// module change.
pub(crate) const CONTROL_REGISTRY_DISCOVERY_VERSION: CandidateVersion = CandidateVersion(1);
