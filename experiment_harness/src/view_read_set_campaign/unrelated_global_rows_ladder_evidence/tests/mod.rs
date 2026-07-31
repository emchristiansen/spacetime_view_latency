//! Focused tests for the ladder rule. One test entity per file.
//!
//! Each case moves exactly one coordinate of an otherwise whole ladder, so it proves which clause
//! rejected it. Candidate, axis and version disagreement are not exercised — each of those
//! vocabularies exposes one value today, so a disagreeing key cannot be built and no enum is widened
//! to fabricate one; those clauses are covered by inspection.

mod fixture;

mod a_repeated_rung_with_another_missing_is_not_a_whole_ladder;
mod a_rung_from_another_block_is_not_this_ladder;
mod a_rung_under_the_matched_role_is_not_this_ladder;
mod six_frozen_rungs_in_any_order_at_either_ordinal_are_one_ladder;
