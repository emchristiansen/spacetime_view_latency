//! Tests for the sink's create/finalize lifecycle and its terminal-on-failure (poisoning) behavior.
//! One test entity per file.
//!
//! These prove *state transitions* only — `O_EXCL` exclusivity, sequence advance on a successful
//! writer return, poisoning after a persist failure, refusal of later writes, and the finalize
//! outcome's aggregation/classification. They do not — and a unit test cannot — prove crash
//! durability, which depends on the platform honoring the fsync contract the production writer
//! exercises.
//!
//! The poisoning/finalize tests inject a scripted `DurableLineWriter` that performs no I/O: its
//! `write_line` and `finalize` merely *return* success or failure per script. A successful return
//! therefore proves
//! only that the sink treats a successful seam return as advancing its sequence/marker — never that
//! bytes were written or synced. A failing return is the sink's seam-failure input; the sink
//! conservatively classifies a line-seam failure as durability-ambiguous because the *production*
//! seam may fail at write, flush, or `sync_data`. This distinction between scripted return behavior
//! and production I/O is stated here once for the whole subtree.
//!
//! Separately, `written_line_carries_the_assigned_record_identity` drives the real
//! `write_manifest`/`write_observation` paths against a `capturing_writer` that records each line's
//! bytes, asserting the serialized `record` identity (and the manifest receipt's) — record content,
//! not a state transition.

mod ambiguous_write_poisons_the_sink;
mod capturing_writer;
mod create_is_exclusive;
mod create_then_finalize;
mod failing_serialize;
mod prior_failure_and_finalize_sync_failure_retain_both;
mod scratch_dir;
mod scripted_writer;
mod serialization_failure_is_definitely_not_written;
mod successful_line_returns_then_finalizer_failure;
mod successful_writer_returns_advance_the_sequence;
mod written_line_carries_the_assigned_record_identity;
