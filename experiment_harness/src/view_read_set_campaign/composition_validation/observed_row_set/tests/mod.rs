//! Focused tests for the canonical encoding and the retained, content-addressed artifact.
//!
//! The encoding tests call [`canonical_encoding`](super::canonical_encoding) directly, with no
//! filesystem at all — which is what splitting it out of the constructor was for. The rest exercise
//! exactly what persistence adds: rejection *before* anything is written, exclusive creation, and a
//! digest that addresses the bytes on disk.
//!
//! One test entity per file, with the filesystem and attempt-directory fixtures shared between them.

mod a_duplicate_entity_uuid_is_rejected;
mod a_record_containing_a_line_break_is_rejected;
mod attempt_directory;
mod each_phase_writes_its_own_artifact_exactly_once;
mod row;
mod rows_are_ordered_by_key_and_every_line_is_terminated;
mod scratch_dir;
mod the_digest_is_sha256_of_the_written_bytes;
mod zero_rows_produce_an_empty_file;
