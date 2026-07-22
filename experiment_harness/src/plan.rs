//! Typed experiment plan.
//!
//! The invariants Control requires are enforced structurally across this module:
//!
//! - Invalid `(arm, growth-regime)` products are unrepresentable: table-scoped arms
//!   (A–E) exist only under `UnrelatedGrowth`; own-slice growth is reachable only by
//!   the key-scoped arms F/F′. See [`cell::Cell`].
//! - The schedulable state ([`schedule::Schedule`]) stores only [`cell::Cell`]
//!   values. The growth regime and the predicted response are *derived* from a cell,
//!   never stored alongside it.
//! - The matched arm and direct-table control runs are derived privately via
//!   [`cell::Cell::matched_runs`]; [`run::Run`] has private fields and no public
//!   constructor, so there is no `Cell × RunRole` product that could pair an arm
//!   with the wrong control table.
//! - The direct-table control is a [`control_table::ControlTable`], never an eighth
//!   view arm.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub mod access_path;
pub mod cell;
pub mod control_table;
pub mod growth_regime;
pub mod key_scoped_arm;
pub mod predicted_response;
pub mod recorded_read_set_class;
pub mod run;
pub mod run_role;
pub mod schedule;
pub mod table_scoped_arm;
