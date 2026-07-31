//! Focused tests for the exclusively-created per-attempt artifact directory. One test entity per
//! file, with the filesystem fixture and the attempt-identity builder shared between them.

mod a_missing_root_is_rejected;
mod a_repeated_attempt_identity_fails_exclusively;
mod pilot_attempt;
mod scratch_dir;
mod the_directory_is_one_named_component_inside_the_root;
