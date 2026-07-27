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
//!
//! The fresh-server campaign's two write-issuing channels add barriers of the same shape, covered
//! the same way: the saturated barrier seals scrambled confirmations in issue order, rejects a
//! duplicate, and classifies a reducer error as an application failure; the paced barrier stops on
//! its own target key and no other, reports that key's *own* observer instant as the sample's
//! endpoint, and surfaces a failed write instead of letting it expire.
//!
//! The campaign's seeding step adds the one-completion barrier ([`await_reducer_completion`]),
//! covered for both callback-delivered failures: a reducer refusal and an SDK internal error are
//! each the application's answer, and each keeps its own wording.
//!
//! Out of reach without a server, and covered by inspection instead: anything holding a
//! `DbConnection` — observer registration and removal, delivery-callback registration, the cache
//! read-back, and both apply channels, which are network events with no pure factor to extract.
//! With them the awaited reducer's remaining branches: a failed issue, an elapsed wait, and a
//! sender dropped without a callback, none of which a hand-delivered message can produce.

mod a_duplicate_measured_confirmation_is_rejected;
mod a_duplicate_prerequisite_confirmation_is_rejected;
mod a_duplicate_saturated_confirmation_is_rejected;
mod a_failed_paced_write_is_rejected_as_an_application_failure;
mod a_failed_saturated_write_is_rejected_as_an_application_failure;
mod a_measured_failure_is_rejected;
mod a_missing_measured_confirmation_times_out_at_the_supplied_deadline;
mod a_missing_prerequisite_times_out_at_the_supplied_deadline;
mod a_paced_sample_reports_its_own_observer_instant;
mod a_paced_sample_stops_only_on_its_own_target_key;
mod a_prerequisite_failure_is_rejected;
mod a_seeding_internal_error_is_an_application_failure;
mod a_seeding_reducer_error_is_an_application_failure;
mod measured_confirmations_seal_in_issue_order;
mod prerequisites_confirm_regardless_of_order;
mod saturated_confirmations_seal_in_issue_order;
