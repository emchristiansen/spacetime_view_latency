//! Focused tests for the frozen mechanical environment gate. One test entity per file.
//!
//! Every sample here is built through [`EnvironmentSample::observed`] and every pair through
//! `EnvironmentGateEvidence::paired`, so these prove the real constructor and the real decision.

mod fixture;

mod a_host_reporting_no_cpus_cannot_decide_the_gate;
mod a_pair_taken_too_close_together_cannot_decide_the_gate;
mod every_clause_of_the_frozen_gate_must_hold;
mod the_first_sample_is_read_only_where_the_rule_names_it;
