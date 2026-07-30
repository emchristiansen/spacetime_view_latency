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
//! The calibration pilot's paced **append** barrier is covered the same way again, with one addition
//! its two-producer channel makes load-bearing: every message is keyed by activity id, failures
//! included, so a stale contradictory report about an already-sealed append cannot end the sample
//! that happens to be waiting. Its three tests cover the target insert, foreign-id skipping and the
//! endpoint instant; a stale failure skipped and a matching one classified; and a channel with no
//! senders left, which is the harness losing its own channel rather than an elapsed bound. All
//! instant arithmetic in them goes through [`paced_append_fixture::checked_after`], because the
//! fail-fast policy is not relaxed for test code.
//!
//! The campaign's seeding step adds the one-completion barrier ([`await_reducer_completion`]),
//! covered for both callback-delivered failures: a reducer refusal and an SDK internal error are
//! each the application's answer, and each keeps its own wording.
//!
//! The two timed cold-subscription adapters contribute no pure factor at all. Where each builds its
//! query is not asserted here because it is not assertable here: `subscribe_retained_from` takes a
//! `MeasuredTarget` and so can only build its query inside its own interval, while
//! `subscribe_cold_target` takes an already-built `String` and so cannot build one there. The
//! signatures carry that distinction, and a test restating it would only re-check the compiler.
//!
//! Out of reach without a server, and covered by inspection instead: anything holding a
//! `DbConnection` — observer registration and removal, delivery-callback registration, the cache
//! read-back, and both apply channels, which are network events with no pure factor to extract.
//! That includes the callback-first endpoint itself: `origin.elapsed()` being the applied callback's
//! first statement is read from the source and from the pinned SDK's documented apply order, not
//! from a hand-delivered message, because nothing pure can drive an SDK callback. With them the
//! awaited reducer's remaining branches: a failed issue, an elapsed wait, and a sender dropped
//! without a callback, none of which a hand-delivered message can produce.

mod a_dropped_paced_append_channel_is_an_infrastructure_failure;
mod a_duplicate_measured_confirmation_is_rejected;
mod a_duplicate_prerequisite_confirmation_is_rejected;
mod a_duplicate_saturated_confirmation_is_rejected;
mod a_failed_paced_write_is_rejected_as_an_application_failure;
mod a_paced_append_failure_is_matched_to_its_own_append;
mod a_paced_append_stops_only_on_its_own_target_id;
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
mod paced_append_fixture;
mod prerequisites_confirm_regardless_of_order;
mod saturated_confirmations_seal_in_issue_order;
