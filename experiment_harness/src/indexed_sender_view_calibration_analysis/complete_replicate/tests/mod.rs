//! Focused tests for §569 admission. One test entity per file.
//!
//! All pure and server-free: every input is a hand-built wire line from
//! [`LedgerFixture`](crate::indexed_sender_view_calibration_analysis::ledger_fixture::LedgerFixture).

mod a_complete_replicate_is_exactly_the_frozen_series;
mod a_forged_key_component_is_refused;
mod a_nonpositive_sample_refuses_admission;
mod a_partial_population_proof_refuses_admission;
mod a_valid_but_nonfrozen_method_fact_is_refused;
mod an_unrecorded_outcome_parses_but_is_inadmissible;
