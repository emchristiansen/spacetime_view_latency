//! Whether a terminated attempt's logical slot may be retried.

use serde::Serialize;

/// Whether the spec's retry rule permits one more attempt at a logical slot.
///
/// Deliberately carries no decision procedure of its own. The rule reads an attempt's terminal
/// outcome *together with* its identity's retry ordinal, and those two are only safely paired inside
/// [`TerminalAttemptRecord`](super::terminal_attempt_record::TerminalAttemptRecord) — so the decision
/// is a method on that record, not a free function over this type. A constructor here would invite
/// exactly the detached pairing the record exists to prevent.
///
/// **Three states, not two.** "The rule refused this slot a retry" and "this outcome poses no retry
/// question at all" are different facts, and collapsing them would make a completed attempt report
/// the same disposition as a security failure. They also behave differently under the ordinal cap:
/// capping applies to a slot that would otherwise be retried, and there is nothing to cap where no
/// retry was ever in question. Keeping [`Self::None`] distinct means the cap is expressed over the
/// state it actually governs rather than over a boolean that has already lost the distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum RetryEligibility {
    /// The slot may be retried once, under a fresh identity at
    /// [`RetryOrdinal::RETRY`](super::retry_ordinal::RetryOrdinal::RETRY).
    Eligible,
    /// The slot may not be retried. If it is not complete, it stays incomplete.
    Ineligible,
    /// The outcome raises no retry question. The attempt either completed — so its slot is already
    /// satisfied — or never executed, so there is no measured slot to reopen: a campaign stopped
    /// short resumes by planning, not by retrying its unplayed remainder.
    None,
}
