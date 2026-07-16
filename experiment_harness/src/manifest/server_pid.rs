//! The OS process id of the started standalone server.

use std::num::NonZeroU32;

use serde::Serialize;

/// PID of the isolated standalone server process. A `NonZeroU32` newtype so a raw OS
/// integer is never confused with another numeric field and pid `0` is unrepresentable;
/// used to resolve `/proc/<pid>/exe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(crate) struct ServerPid(NonZeroU32);

impl ServerPid {
    pub(crate) fn new(pid: NonZeroU32) -> Self {
        Self(pid)
    }

    /// The raw process id.
    pub(crate) fn get(&self) -> u32 {
        self.0.get()
    }
}
