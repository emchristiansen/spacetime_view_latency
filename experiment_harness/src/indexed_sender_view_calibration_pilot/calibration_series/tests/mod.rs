//! Focused tests for the sealed calibration series. One test entity per file.
//!
//! The fixture is private to this module and reached as `super::fixture`, so nothing test-only
//! widens the module's real API.

mod a_complete_series_must_match_its_verified_appends;
mod a_nonpositive_sample_invalidates_the_series;
mod a_replicate_is_exactly_the_frozen_sample_count;
mod fixture;
