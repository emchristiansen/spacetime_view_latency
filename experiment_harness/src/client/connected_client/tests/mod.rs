//! Focused deterministic tests for the two measured-dose phase barriers — the phase-A prerequisite
//! barrier ([`collect_prerequisite_batch`]) and the phase-B measured barrier
//! ([`collect_measured_batch`]) — each driven entirely by hand-delivered messages over an [`mpsc`]
//! channel, with no live server.
//!
//! Phase B: measured confirmations delivered out of order still seal in issue order; a duplicate
//! measured confirmation is rejected; a measured failure is rejected; a batch missing one measured
//! confirmation times out once the supplied deadline has passed. Phase A: prerequisites confirm
//! regardless of delivery order; a duplicate prerequisite is rejected; a prerequisite failure is
//! rejected; a batch missing one prerequisite times out once the supplied deadline has passed.
//!
//! (That the *production* deadlines are one whole-batch budget each, and that phase A fully precedes
//! phase B, are structural properties of [`ConnectedClient::measure_dose`] and its phase methods,
//! inspected there, not asserted by these pure-barrier tests.)

mod a_duplicate_measured_confirmation_is_rejected;
mod a_duplicate_prerequisite_confirmation_is_rejected;
mod a_measured_failure_is_rejected;
mod a_missing_measured_confirmation_times_out_at_the_supplied_deadline;
mod a_missing_prerequisite_times_out_at_the_supplied_deadline;
mod a_prerequisite_failure_is_rejected;
mod measured_confirmations_seal_in_issue_order;
mod prerequisites_confirm_regardless_of_order;
