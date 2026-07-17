//! Focused deterministic tests for the slot-indexed prerequisite confirmation set: an empty set is
//! vacuously complete, an out-of-range or duplicate confirmation is loud, and a set completes only
//! once every expected slot is recorded.

mod a_duplicate_confirmation_is_rejected;
mod a_full_set_completes_only_when_every_slot_is_recorded;
mod an_empty_set_is_immediately_complete;
mod an_out_of_range_index_is_rejected;
