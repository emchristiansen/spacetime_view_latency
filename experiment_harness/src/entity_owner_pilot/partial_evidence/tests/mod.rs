//! Focused tests for the strict-prefix sealing of a failed attempt's evidence. One test entity per
//! file; `rungs` is the shared fixture helper.

mod a_complete_ladder_is_rejected;
mod a_gapped_prefix_is_rejected;
mod every_strict_ascending_prefix_seals;
mod rungs;
