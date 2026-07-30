//! The measured-subscriber client: connect, seed, subscribe, read back.
//!
//! One public entity per file; this entry file is declarative re-exports only.

pub mod connected_client;
pub mod measured_step_failure;
pub mod paced_append;
pub mod paced_append_failure;
pub mod reconnect_failure;
pub mod view_event_shape;
