//! Whether a completed attempt's method is admissible as evidence.

use serde::Serialize;

/// Whether a completed attempt was produced by a method still considered sound.
///
/// The spec keeps failed-method material in the ledger rather than deleting it: the historical
/// 540-run inverted-Chronicle campaign is "failed-method provenance only", imported "without
/// rewriting it", while analysis must still "select exactly one current *valid* complete attempt per
/// logical slot". Those two requirements only coexist if validity is a recorded property of an
/// attempt rather than a decision made by whoever is reading the ledger that day.
///
/// A newly recorded attempt is [`Self::Valid`]. An attempt becomes [`Self::FailedMethod`] only by a
/// later, deliberate supersession that says *why* — never by silent deletion, and never by
/// rewriting the original record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum MethodValidity {
    /// The method is sound; this attempt is admissible for its logical slot.
    Valid,
    /// The method was later found unsound. The record is retained as provenance and must never be
    /// selected as a slot's current evidence.
    FailedMethod,
}
