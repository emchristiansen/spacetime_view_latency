//! Focused tests for the endpoint rule. One test entity per file.
//!
//! These run against the pure siblings rather than `EndpointFactor::of`, because a ladder is
//! mintable only from a live campaign — `SelectedCompleteAttempt` comes from
//! `ReconciledCampaign::selections` alone. The projection `of` performs is one `each_ref().map(..)`
//! over typed accessors and is covered by inspection; the endpoint rule and the serialized content,
//! where a wrong answer would still look plausible, are exercised directly.

mod fixture;

mod a_flat_ladder_has_unit_factor;
mod only_the_endpoints_enter_the_factor;
mod rung_order_does_not_change_the_factor;
mod the_factor_is_the_last_rung_over_the_first;
mod the_serialized_pair_is_the_exact_reduced_ratio;
