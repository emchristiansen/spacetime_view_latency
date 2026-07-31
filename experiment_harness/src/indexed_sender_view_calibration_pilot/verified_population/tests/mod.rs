//! Focused tests for the row-by-row composition verifier. One test entity per file.
//!
//! Every case is exercised through the **real** `VerifiedPopulation::verify`, over rows built by the
//! shared fixture and mutated one property at a time. The fixture is private to this module and
//! reached as `super::fixture`, so nothing test-only widens the module's real API.

mod an_unrelated_row_in_the_arm_is_a_leak;
mod exact_populations_verify;
mod fixture;
mod same_cardinality_substitutions_are_rejected;
mod witness_substitutions_are_rejected;
