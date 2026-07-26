//! Whether a completed attempt's method was considered sound when it terminated.

use serde::Serialize;

/// The method validity of a completed attempt **as known at the moment it terminated**.
///
/// **The environment gate is deliberately absent from this type.** The gate is *prospective*: its
/// two-sample pair is a preflight ending immediately before launch, so an attempt that got as far as
/// a completed outcome has already passed it. Gate invalidation is a condition before the first
/// measured sample, not a verdict on a completed attempt, and it lives on the refusal side as
/// [`AttemptOutcome::PreflightRejected`](super::attempt_outcome::AttemptOutcome::PreflightRejected),
/// which carries no evidence at all. Modelling it here would create a "complete but environmentally
/// invalid" outcome, which the contract forbids twice over: gates are prospective only, and no
/// criterion may reference a measured outcome.
///
/// The *post-attempt* host reading is likewise absent, and deliberately unreachable from here: it is
/// supporting diagnostics that "never invalidates evidence", so it is recorded as its own ledger
/// line rather than as a field of any evidence or validity type. Nothing that could consult it in a
/// selection decision can reach it.
///
/// **This is not the source of truth for supersession.** The ledger is append-only, so a terminal
/// record cannot later "become" invalid: doing so would mean rewriting a line that is already on
/// disk, which is exactly what the spec forbids when it requires historical material be imported
/// "without rewriting it". A method found unsound afterwards must be recorded as its own appended
/// supersession record, keyed to the attempt identity and candidate version — and the selection rule
/// must *fold those records over* the terminal outcomes rather than reading this field alone. That
/// record is [`MethodSupersession`](super::method_supersession::MethodSupersession), and the fold is
/// [`ReconciledCampaign::selections`](super::reconciled_campaign::ReconciledCampaign::selections).
///
/// So this field answers one narrow question — did the campaign already know, when it wrote this
/// line, that the method was unsound? — and a reader who treats it as the final word will select
/// superseded evidence.
///
/// Only [`Self::Valid`] is declared, because it is the only value this stage can produce: an attempt
/// whose method was already known unsound would not have been executed. The variant is retained as a
/// field rather than left implicit for the same reason
/// [`CandidateId`](super::candidate_id::CandidateId) is: a line that does not state the fact cannot
/// be distinguished later from one written before the fact was recorded at all. The supersession
/// record type, and the validated-ledger selection that folds it, arrive with the ledger-reading
/// layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum MethodValidity {
    /// No supersession was known when this attempt terminated.
    Valid,
}
