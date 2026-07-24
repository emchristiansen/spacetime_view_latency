//! Focused deterministic tests for the slot-indexed measured-latency accumulator: an out-of-range or
//! duplicate confirmation is loud, a missing confirmation cannot seal, and a full batch seals in
//! issue order regardless of record order.

mod a_duplicate_confirmation_is_rejected;
mod a_full_batch_seals_in_issue_order;
mod an_out_of_range_index_is_rejected;
mod sealing_with_a_missing_confirmation_fails;
