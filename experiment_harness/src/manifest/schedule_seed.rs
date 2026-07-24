//! The explicit seed driving deterministic scheduling.

use serde::Serialize;

/// The explicit seed that deterministically drives global block-order randomization. A
/// newtype (nominal domain separation) so a schedule seed is never silently interchanged
/// with another `u64`; recorded in the run manifest so every randomized decision is
/// reproducible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(crate) struct ScheduleSeed(u64);

impl ScheduleSeed {
    pub(crate) const fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The raw seed.
    pub(crate) fn get(&self) -> u64 {
        self.0
    }
}
