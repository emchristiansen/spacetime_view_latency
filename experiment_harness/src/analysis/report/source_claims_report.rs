//! The report projection of the six release-qualified v2.6.1 mechanism citations.

use serde::Serialize;

use crate::analysis::report::mechanism_citation_report::MechanismCitationReport;
use crate::analysis::report::mechanism_kind::MECHANISM_CLAIM_COUNT;

/// The complete set of six release-qualified v2.6.1 mechanism citations (spec: "report the six
/// release-qualified v2.6.1 mechanism citations, including the corrected delete/reinsert coalescing
/// location in `mut_tx.rs` plus `delete_table.rs`"). A boxed fixed [`MECHANISM_CLAIM_COUNT`] array, so
/// *exactly six citations* is a property of the report type; serde supports this length directly (≤ 32),
/// so no manual `Serialize` is needed. The citations are release constants, not campaign-derived data, so
/// this projection takes no campaign input.
#[derive(Debug, Serialize)]
pub(crate) struct SourceClaimsReport {
    /// The six mechanism citations, one per [`MechanismKind`](super::mechanism_kind::MechanismKind), in
    /// spec order.
    citations: Box<[MechanismCitationReport; MECHANISM_CLAIM_COUNT]>,
}

impl SourceClaimsReport {
    /// Build the fixed six-citation set from the closed
    /// [`MechanismKind::ALL`](super::mechanism_kind::MechanismKind::ALL) identities. Takes no input: the
    /// citations are release-qualified constants.
    pub(crate) fn all() -> Self {
        todo!("Phase 2: project MechanismKind::ALL into the six release-qualified citations")
    }
}
