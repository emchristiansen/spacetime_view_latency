//! Focused tests for the discovery-screen record state machine. One test entity per file.
//!
//! Every rule is exercised through the **real record constructors**, not through the validators they
//! call. Asserting only that `AttemptStage::ensure_admits` rejects the wrong stage would leave the
//! coupling untested — a constructor that stopped calling it would still pass. The three
//! post-publication shapes take provenance by value, so they are reached with
//! [`AttemptProvenance::fixture`](crate::entity_owner_pilot::attempt_provenance::AttemptProvenance::fixture),
//! a `#[cfg(test)]` constructor that does not exist in a production build and leaves the real
//! `observed` path — and its requirement of live provisioning capabilities — untouched.

mod a_retry_identity_requires_an_earlier_superseded_ordinal;
mod an_after_observation_failure_splits_on_the_timed_apply;
mod disposition_must_match_acquisition_depth;
mod every_failure_kind_maps_to_one_record_shape;
mod evidence_requires_a_complete_host_bracket;
mod fixture;
mod partial_provision_prefixes_are_monotone;
mod post_apply_failures_retain_raw_nanoseconds;
mod stage_and_sampling_progress_cannot_disagree;
