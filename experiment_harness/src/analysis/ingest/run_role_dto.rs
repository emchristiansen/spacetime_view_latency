//! Untrusted mirror of [`RunRole`](crate::plan::run_role::RunRole).

use serde::Deserialize;

/// The wire form of a run's role: the externally-tagged unit variants `"Arm"` / `"Control"`. An
/// unknown role string is rejected by serde as it deserializes, closing the tag set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub(crate) enum RunRoleDto {
    Arm,
    Control,
}
