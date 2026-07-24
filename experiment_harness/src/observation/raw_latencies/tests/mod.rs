//! Tests for the exact-cardinality sealing of a dose's raw latency vector. One test entity per
//! file; `samples` is the shared fixture helper.

mod empty_is_rejected;
mod exactly_batch_size_seals;
mod one_too_few_is_rejected;
mod one_too_many_is_rejected;
mod samples;
