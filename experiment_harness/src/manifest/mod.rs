//! Immutable, typed run manifest and its capability-specific validation proofs.
//!
//! Binary-internal: entities are crate-visible (`pub(crate)`), not broadly public. This
//! entry file is declarative re-exports only.

pub(crate) mod database_identity;
pub(crate) mod listen_address;
pub(crate) mod preregistered_parameters;
pub(crate) mod release_commit;
pub(crate) mod run_coordinate;
pub(crate) mod schedule_seed;
pub(crate) mod server_pid;
pub(crate) mod strict_version_output;
pub(crate) mod tool_version_line;
pub(crate) mod validated_run_manifest;
pub(crate) mod verified_cli;
pub(crate) mod verified_module_artifact;
pub(crate) mod verified_server_process;
pub(crate) mod verified_standalone;
pub(crate) mod wasm_sha256;
