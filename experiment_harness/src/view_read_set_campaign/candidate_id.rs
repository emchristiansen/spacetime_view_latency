//! Which deployable candidate an attempt exercises.

use serde::Serialize;

/// Which candidate an attempt's evidence belongs to.
///
/// Named for the spec's "Minimal type design" `CandidateId`, which enumerates eight; only the one
/// this stage can execute is declared. A variant added later must not reinterpret evidence already
/// written under this vocabulary, and arrives together with its executable candidate.
///
/// The field is retained on [`AttemptKey`](super::attempt_key::AttemptKey) even though it admits one
/// value: a ledger line that does not name its candidate cannot be disambiguated once a second
/// candidate exists, and lines cannot be re-tagged retroactively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum CandidateId {
    /// Sender-scoped entity-ownership view; an analogue of Muninn's real `entity_owner_view`.
    EntityOwnerSenderView,
}
