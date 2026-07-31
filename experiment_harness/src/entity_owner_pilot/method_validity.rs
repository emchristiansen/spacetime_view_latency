//! Whether a completed attempt's method is admissible as evidence.

use serde::Serialize;

/// Whether a completed attempt was produced by a method still considered sound.
///
/// The spec retains failed-method material in the ledger rather than deleting it, yet analysis must
/// select one *valid* attempt per logical slot. Those coexist only if validity is a recorded
/// property of the attempt rather than a judgement made by whoever reads the ledger.
///
/// A newly recorded attempt is [`Self::Valid`]; it becomes [`Self::FailedMethod`] only by a
/// deliberate later supersession, never by rewriting or deleting the original record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum MethodValidity {
    /// Admissible as its logical slot's evidence.
    Valid,
    /// Method later found unsound. Retained as provenance; never selected as current evidence.
    FailedMethod,
}
