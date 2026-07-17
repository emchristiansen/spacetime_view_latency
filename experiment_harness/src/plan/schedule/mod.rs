//! The schedulable plan (hide-impl subtree).
//!
//! `BlockRun`'s constructor is `pub(super)`, confined to this `schedule` subtree, so
//! [`schedule::Schedule`] is its sole constructor — no other `plan` child can build
//! one. This entry file is declarative re-exports only.

mod block_coordinate;
mod block_run;
mod schedule;

pub(crate) use block_coordinate::BlockCoordinate;
pub use block_run::BlockRun;
pub use schedule::Schedule;

#[cfg(test)]
mod tests;
