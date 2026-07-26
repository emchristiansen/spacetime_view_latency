//! Focused tests for whole-ledger accounting and the selection fold over it. One test entity per
//! file.
//!
//! **What this suite proves, and where it stops.** Two tests read the healthy baseline directly: a
//! real sixty-slot campaign ledger is accepted, with its accounted attempts in terminal-line order
//! and the provisioning disposition each outcome implies; and selecting over that same campaign
//! yields nothing, because no attempt in it completed. Every other test takes that baseline and
//! makes exactly one change — augmenting it with a retry or a supersession it does not
//! otherwise contain, or perturbing one line — so that what it reports is the consequence of that
//! single difference. Between them they cover every rule reachable without a live server: sequence
//! contiguity, the opening inventory, one terminal per identity, every predeclared original
//! present, retries only off eligible originals, preflight-clearance cardinality and position,
//! post-attempt readings for attempts that measured nothing, orphan auxiliary lines, supersession
//! reach, and the one path through selection that a ledger with no complete attempt can take.
//!
//! Four rules are **compile-checked and directly inspected only**, because each needs an
//! [`AttemptProvenance`](crate::view_read_set_campaign::attempt_provenance::AttemptProvenance),
//! whose sole constructor requires a verified distribution, a running server, and a published
//! module:
//!
//! - provisioned-line cardinality and position, in both directions;
//! - the successful path of provenance admission, and its all-or-nothing refusal;
//! - post-attempt accounting for an attempt that *did* measure — including the **adjacency** half of
//!   the rule, that the reading sits at exactly the sequence after its terminal line with nothing in
//!   between. Only an outcome requiring one reading reaches that clause at all, since for every
//!   reachable outcome the required count is zero and cardinality is refused first;
//! - acceptance of any `Complete` attempt at all, and therefore every *nonempty* selection: minting
//!   a selection, carrying its admitted provenance onto it, excluding a superseded record before the
//!   ordinal minimum, and competing two ordinals. The empty-result path is reachable and is tested;
//!   nothing past it is. Lowest-ordinal competition is doubly unreachable — reconciliation admits a
//!   retry only off a non-complete original — so a provisioning-capable fixture alone would not
//!   close it.
//!
//! A fifth rule is unreachable for a different reason, and will not stay so: "a terminal identity
//! addressing no frozen logical slot" cannot be constructed today, because every component of an
//! [`AttemptKey`](crate::view_read_set_campaign::attempt_key::AttemptKey) is closed to the values
//! this one campaign uses, making the frozen inventory the complete cross product of every
//! representable slot. It is implemented as a runtime rejection rather than an `expect` precisely
//! because a later candidate, axis, block, or candidate version makes it reachable without changing
//! a line of the reconciler.
//!
//! None of these gaps is closed by widening provenance construction. A test-only constructor would
//! make the admission gate forgeable, and every guarantee that rests on it would become a
//! convention.

mod a_campaign_with_no_complete_attempt_selects_nothing;
mod a_fully_accounted_campaign_ledger_is_accepted;
mod a_ledger_whose_sequences_are_not_contiguous_from_zero_is_refused;
mod a_ledger_without_exactly_one_opening_inventory_is_refused;
mod a_post_attempt_reading_for_an_attempt_that_measured_nothing_is_refused;
mod a_predeclared_slot_with_no_terminal_record_is_refused;
mod a_retry_of_an_ineligible_original_is_refused;
mod a_supersession_reaching_a_recorded_attempt_is_kept;
mod a_supersession_reaching_no_recorded_attempt_is_refused;
mod an_attempts_preflight_clearances_must_match_its_outcome;
mod an_auxiliary_line_for_an_attempt_with_no_terminal_record_is_refused;
mod an_eligible_original_may_be_retried_once;
mod an_identity_with_more_than_one_terminal_record_is_refused;
mod fixture;
