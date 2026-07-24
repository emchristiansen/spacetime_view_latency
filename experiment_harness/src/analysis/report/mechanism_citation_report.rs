//! The report projection of one release-qualified v2.6.1 mechanism citation.

use serde::Serialize;

use crate::analysis::report::mechanism_kind::MechanismKind;

/// One release-qualified mechanism citation: its typed [`MechanismKind`] identity, the claim it asserts,
/// and the exact v2.6.1 source location backing it (spec: "cite exact `v2.6.1` source paths/lines or
/// immutable commit links"). The identity is the closed typed enum, never a `String`; the claim text and
/// the source location are genuinely open text (prose and a `path:line`/immutable-link), so they are
/// `String`.
#[derive(Debug, Serialize)]
pub(crate) struct MechanismCitationReport {
    /// The closed typed identity of the cited mechanism.
    kind: MechanismKind,
    /// The claim asserted about the mechanism — open prose.
    claim: String,
    /// The exact `v2.6.1` source `path:line`(s) or immutable commit link backing the claim — open text.
    source_location: String,
}

impl MechanismCitationReport {
    /// Build one mechanism citation. One input — the typed [`MechanismKind`] — from which the fixed claim
    /// text and its release-qualified source location are looked up, so neither can be paired with the
    /// wrong mechanism.
    pub(crate) fn of(kind: MechanismKind) -> Self {
        let _ = kind;
        todo!("Phase 2: bind the fixed claim text and v2.6.1 source location for this mechanism kind")
    }
}
