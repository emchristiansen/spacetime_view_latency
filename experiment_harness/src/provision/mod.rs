//! Official 2.6.1 server provisioning capabilities and the run driver.
//!
//! Binary-internal: entities are crate-visible (`pub(crate)`). Each provisioning capability is
//! an owning type that gates a live resource and requires explicit consuming cleanup, with a
//! loud [`Drop`] guard. This entry file is declarative re-exports only.

pub(crate) mod fresh_data_dir;
pub(crate) mod provision;
pub(crate) mod running_pinned_server;
pub(crate) mod staged_module_wasm;
pub(crate) mod teardown;
pub(crate) mod verified_distribution;
