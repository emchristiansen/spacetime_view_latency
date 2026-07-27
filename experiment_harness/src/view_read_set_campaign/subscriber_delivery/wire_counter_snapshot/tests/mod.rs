//! Focused tests for the received-traffic window arithmetic. One test entity per file.
//!
//! Every case here is pure: two snapshots in, one reduction out. That is the whole of what can be
//! tested without a server, and deliberately the whole of what can go wrong in this file — taking
//! the readings is the meter's job, and its coherence comes from Prometheus's own shard protocol
//! rather than from anything these tests could exercise.

mod a_byte_sum_that_fell_over_the_window_is_refused;
mod a_clean_window_converts_both_counters_exactly;
mod a_frame_count_that_ran_backwards_is_refused;
mod an_absolute_byte_sum_outside_the_exact_integer_range_is_refused;
